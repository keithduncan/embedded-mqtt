use core::{cmp::min, convert::TryFrom, result::Result, str};

use crate::{
    error::{DecodeError, EncodeError},
    status::Status,
};

use super::{values, Decodable, Encodable};

impl<'buf> Decodable<'buf> for &'buf str {
    fn decode(bytes: &'buf [u8]) -> Result<Status<(usize, &'buf str)>, DecodeError> {
        parse_string(bytes)
    }
}

impl Encodable for str {
    fn encoded_len(&self) -> usize {
        2 + self.len()
    }

    fn encode(&self, bytes: &mut [u8]) -> Result<usize, EncodeError> {
        encode_string(self, bytes)
    }
}

pub fn parse_string(bytes: &[u8]) -> Result<Status<(usize, &str)>, DecodeError> {
    let offset = 0;

    let (offset, string_len) = read!(values::parse_u16, bytes, offset);

    let available = bytes.len() - offset;

    let needed = string_len as usize - min(available, string_len as usize);
    if needed > 0 {
        return Ok(Status::Partial(needed));
    }

    let val = if string_len > 0 {
        // Rust string slices are never in the code point range 0xD800 and
        // 0xDFFF which takes care of requirement MQTT-1.5.3-1. str::from_utf8
        // will fail if those code points are found in "bytes".
        //
        // Rust utf-8 decoding also takes care of MQTT-1.5.3-3. U+FEFF does not
        // get ignored/stripped off.
        str::from_utf8(&bytes[2..(2 + string_len) as usize])?
    } else {
        ""
    };

    // Requirement MQTT-1.5.3-2 requires that there be no U+0000 code points
    // in the string.
    if val.chars().any(|ch| ch == '\u{0000}') {
        return Err(DecodeError::Utf8);
    }

    Ok(Status::Complete(((2 + string_len) as usize, val)))
}

pub fn encode_string(string: &str, bytes: &mut [u8]) -> Result<usize, EncodeError> {
    let size = match u16::try_from(string.len()) {
        Err(_) => return Err(EncodeError::ValueTooBig),
        Ok(s) => s,
    };

    if bytes.len() < (2 + size) as usize {
        return Err(EncodeError::OutOfSpace);
    }

    values::encode_u16(size, &mut bytes[0..2])?;
    (&mut bytes[2..2 + size as usize]).copy_from_slice(string.as_bytes());

    Ok(2 + size as usize)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        format,
        io::{Cursor, Write},
        vec::Vec,
    };

    use byteorder::{BigEndian, ByteOrder};

    use byteorder::WriteBytesExt;

    #[test]
    fn small_buffer() {
        assert_eq!(Ok(Status::Partial(2)), parse_string(&[]));
        assert_eq!(Ok(Status::Partial(1)), parse_string(&[0]));

        let mut buf = [0u8; 2];
        BigEndian::write_u16(&mut buf, 16);
        assert_eq!(Ok(Status::Partial(16)), parse_string(&buf));
    }

    #[test]
    fn empty_str() {
        let mut buf = [0u8; 2];
        BigEndian::write_u16(&mut buf, 0);
        assert_eq!(Ok(Status::Complete((2, ""))), parse_string(&buf));
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
        assert_eq!(Err(DecodeError::Utf8), parse_string(buf.get_ref().as_ref()));
    }

    #[test]
    fn null_utf8() {
        let inp = format!("don't {} panic!", '\u{0000}');
        let mut buf = Cursor::new(Vec::new());
        buf.write_u16::<BigEndian>(inp.len() as u16).unwrap();
        buf.write(inp.as_bytes()).unwrap();
        assert_eq!(Err(DecodeError::Utf8), parse_string(buf.get_ref().as_ref()));
    }

    #[test]
    fn encode() {
        let mut buf = [0u8; 3];
        let result = encode_string("a", &mut buf[0..3]);
        assert_eq!(result, Ok(3));
        assert_eq!(buf, [0b00000000, 0b00000001, 0x61]);
    }
}
