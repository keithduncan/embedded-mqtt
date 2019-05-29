use core::result::Result;

use crate::{
    codec::{self, Decodable},
    status::Status,
    error::DecodeError,
};

// VariableHeader for Connack packet
#[derive(PartialEq, Debug)]
pub struct Suback {
    packet_identifier: u16,
}

impl Suback {
    pub fn packet_identifier(&self) -> u16 {
        self.packet_identifier
    }
}

impl<'buf> Decodable<'buf> for Suback {
    fn from_bytes(bytes: &'buf [u8]) -> Result<Status<(usize, Self)>, DecodeError> {
        if bytes.len() < 2 {
            return Ok(Status::Partial(2 - bytes.len()));
        }

        let offset = 0;

        // read connack flags
        let (offset, packet_identifier) = read!(codec::values::parse_u16, bytes, offset);

        Ok(Status::Complete((offset, Suback {
            packet_identifier,
        })))
    }
}

#[cfg(test)]
mod tests {
    
}
