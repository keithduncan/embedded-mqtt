use core::result::Result;

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