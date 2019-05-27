use crate::{
    status::Status,
    result::Result,
};

use byteorder::{
    BigEndian,
    ByteOrder,
};

pub fn parse_u8(bytes: &[u8]) -> Result<Status<(usize, u8)>> {
    if bytes.len() < 1 {
        return Ok(Status::Partial(1))
    }

    Ok(Status::Complete((1, bytes[0])))
}

pub fn parse_u16(bytes: &[u8]) -> Result<Status<(usize, u16)>> {
    if bytes.len() < 2 {
        return Ok(Status::Partial(2))
    }

    Ok(Status::Complete((2, BigEndian::read_u16(&bytes[0..2]))))
}