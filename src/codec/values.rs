use core::result::Result;

use crate::{
    status::Status,
    error::ParseError,
};

use byteorder::{
    BigEndian,
    ByteOrder,
};

pub fn parse_u8(bytes: &[u8]) -> Result<Status<(usize, u8)>, ParseError> {
    if bytes.len() < 1 {
        return Ok(Status::Partial(1))
    }

    Ok(Status::Complete((1, bytes[0])))
}

pub fn parse_u16(bytes: &[u8]) -> Result<Status<(usize, u16)>, ParseError> {
    if bytes.len() < 2 {
        return Ok(Status::Partial(2 - bytes.len()))
    }

    Ok(Status::Complete((2, BigEndian::read_u16(&bytes[0..2]))))
}
