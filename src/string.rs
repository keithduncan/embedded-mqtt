use core::str;

use crate::{
	status::Status,
	error::Error,
	result::Result,
};

use byteorder::{
    BigEndian,
    ByteOrder,
};

pub fn parse_string(bytes: &[u8]) -> Result<Status<(usize, &str)>> {
    // we need at least the 2 bytes to figure out length of the utf-8 encoded
    // string in bytes
    if bytes.len() < 2 {
        return Ok(Status::Partial);
    }

    let len = BigEndian::read_u16(bytes);
    if bytes.len() - 2 < len as usize {
        return Ok(Status::Partial);
    }

    let val = if len > 0 {
        // Rust string slices are never in the code point range 0xD800 and
        // 0xDFFF which takes care of requirement MQTT-1.5.3-1. str::from_utf8
        // will fail if those code points are found in "bytes".
        //
        // Rust utf-8 decoding also takes care of MQTT-1.5.3-3. U+FEFF does not
        // get ignored/stripped off.
        str::from_utf8(&bytes[2..(len + 2) as usize])?
    } else {
        ""
    };

    // Requirement MQTT-1.5.3-2 requires that there be no U+0000 code points
    // in the string.
    if val.chars().any(|ch| ch == '\u{0000}') {
        return Err(Error::Utf8)
    }
    
    Ok(Status::Complete(((len + 2) as usize, val)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        io::{Cursor, Write},
        vec::Vec,
        format,
    };

    use byteorder::WriteBytesExt;

    #[test]
    fn small_buffer() {
        assert_eq!(Status::Partial, parse_string(&[]).unwrap());
        assert_eq!(Status::Partial, parse_string(&[0]).unwrap());

        let mut buf = [0u8; 2];
        BigEndian::write_u16(&mut buf, 16);
        assert_eq!(Status::Partial, parse_string(&buf).unwrap());
    }

    #[test]
    fn empty_str() {
        let mut buf = [0u8; 2];
        BigEndian::write_u16(&mut buf, 0);
        assert_eq!(Status::Complete((2, "")), parse_string(&buf).unwrap());
    }

    #[test]
    fn parse_str() {
        let inp = "don't panic!";
        let mut buf = Cursor::new(Vec::new());
        buf.write_u16::<BigEndian>(inp.len() as u16).unwrap();
        buf.write(inp.as_bytes()).unwrap();
        assert_eq!(
            Status::Complete((14, inp)),
            parse_string(buf.get_ref().as_ref()).unwrap()
        );
    }
    
    #[test]
    fn invalid_utf8() {
        let inp = [0, 159, 146, 150];
        let mut buf = Cursor::new(Vec::new());
        buf.write_u16::<BigEndian>(inp.len() as u16).unwrap();
        buf.write(&inp).unwrap();
        assert_eq!(Err(Error::Utf8), parse_string(buf.get_ref().as_ref()));
    }

    #[test]
    fn null_utf8() {
        let inp = format!("don't {} panic!", '\u{0000}');
        let mut buf = Cursor::new(Vec::new());
        buf.write_u16::<BigEndian>(inp.len() as u16).unwrap();
        buf.write(inp.as_bytes()).unwrap();
        assert_eq!(Err(Error::Utf8), parse_string(buf.get_ref().as_ref()));
    }
}