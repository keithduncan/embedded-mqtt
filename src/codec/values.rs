use core::{
    cmp::min,
    convert::TryFrom,
    result::Result,
};

use crate::{
    status::Status,
    error::{DecodeError, EncodeError},
};

use byteorder::{
    BigEndian,
    ByteOrder,
};

pub fn parse_u8(bytes: &[u8]) -> Result<Status<(usize, u8)>, DecodeError> {
    if bytes.len() < 1 {
        return Ok(Status::Partial(1))
    }

    Ok(Status::Complete((1, bytes[0])))
}

#[allow(dead_code)]
pub fn encode_u8(value: u8, bytes: &mut [u8]) -> Result<usize, EncodeError> {
    if bytes.len() < 1 {
        return Err(EncodeError::OutOfSpace)
    }

    bytes[0] = value;
    Ok(1)
}

pub fn parse_u16(bytes: &[u8]) -> Result<Status<(usize, u16)>, DecodeError> {
    if bytes.len() < 2 {
        return Ok(Status::Partial(2 - bytes.len()))
    }

    Ok(Status::Complete((2, BigEndian::read_u16(&bytes[0..2]))))
}

pub fn encode_u16(value: u16, bytes: &mut [u8]) -> Result<usize, EncodeError> {
    if bytes.len() < 2 {
        return Err(EncodeError::OutOfSpace)
    }

    BigEndian::write_u16(&mut bytes[0..2], value);
    Ok(2)
}

pub fn parse_bytes(bytes: &[u8]) -> Result<Status<(usize, &[u8])>, DecodeError> {
    let offset = 0;
    let (offset, len) = read!(parse_u16, bytes, offset);

    let available = bytes.len() - offset;
    let needed = len as usize - min(available, len as usize);
    if needed > 0 {
        return Ok(Status::Partial(needed));
    }
    let payload = &bytes[offset..offset+len as usize];

    Ok(Status::Complete((offset + len as usize, payload)))
}

pub fn encode_bytes(value: &[u8], bytes: &mut [u8]) -> Result<usize, EncodeError> {
    let size = match u16::try_from(value.len()) {
        Err(_) => return Err(EncodeError::ValueTooBig),
        Ok(s) => s,
    };

    let offset = encode_u16(size, bytes)?;

    let payload_size = value.len();
    if offset + payload_size > bytes.len() {
        return Err(EncodeError::OutOfSpace)
    }

    (&mut bytes[offset..offset + payload_size as usize]).copy_from_slice(value);

    Ok(offset + payload_size)
}
