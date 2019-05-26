use crate::{
    string,
    status::Status,
    result::Result,
};

use byteorder::{
    BigEndian,
    ByteOrder,
};

// VariableHeader for Connect packet
pub struct Connect<'buf> {
    name: &'buf str,
    level: u8,
    flags: u8,
    keep_alive: u16,
}

fn parse_byte(bytes: &[u8]) -> Result<Status<(usize, u8)>> {
    if bytes.len() < 1 {
        return Ok(Status::Partial)
    }

    Ok(Status::Complete((1, bytes[0])))
}

fn parse_length(bytes: &[u8]) -> Result<Status<(usize, u16)>> {
    if bytes.len() < 2 {
        return Ok(Status::Partial)
    }

    Ok(Status::Complete((2, BigEndian::read_u16(&bytes[0..2]))))
}

impl<'buf> Connect<'buf> {
    pub fn from_bytes(bytes: &[u8]) -> Result<Status<(usize, Connect)>> {
        let offset = 0;

        // read protocol name
        let (offset, name) = complete!(string::parse_string(&bytes[offset..]));

        // read protocol revision
        let (offset, level) = complete!(parse_byte(&bytes[offset..]));

        // read protocol flags
        let (offset, flags) = complete!(parse_byte(&bytes[offset..]));

        // read protocol keep alive
        let (offset, keep_alive) = complete!(parse_length(&bytes[offset..]));

        Ok(Status::Complete((offset, Connect {
            name,
            level,
            flags,
            keep_alive,
        })))
    }

    pub fn name(&self) -> &str {
        self.name
    }

    pub fn level(&self) -> &u8 {
        &self.level
    }

    pub fn flags(&self) -> &u8 {
        &self.flags
    }

    pub fn keep_alive(&self) -> &u16 {
        &self.keep_alive
    }
}
