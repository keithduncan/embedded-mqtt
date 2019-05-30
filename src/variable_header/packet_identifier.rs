use core::result::Result;

use crate::{
    fixed_header::PacketFlags,
    codec::{self, Encodable},
    status::Status,
    error::{DecodeError, EncodeError},
};

use super::{HeaderDecode, PacketId};

// TODO make this a non-zero u16 when it is stable
#[derive(PartialEq, Debug)]
pub struct PacketIdentifier(PacketId);

impl PacketIdentifier {
    pub fn new(packet_identifier: PacketId) -> Self {
        Self(packet_identifier)
    }

    pub fn packet_identifier(&self) -> PacketId {
        self.0
    }
}

impl<'buf> HeaderDecode<'buf> for PacketIdentifier {
    fn decode(_flags: PacketFlags, bytes: &'buf [u8]) -> Result<Status<(usize, Self)>, DecodeError> {
        // read connack flags
        let (offset, packet_identifier) = read!(codec::values::parse_u16, bytes, 0);

        Ok(Status::Complete((offset, Self(packet_identifier))))
    }
}

impl Encodable for PacketIdentifier {
    fn encoded_len(&self) -> usize {
        2
    }

    fn encode(&self, bytes: &mut [u8]) -> Result<usize, EncodeError> {
        codec::values::encode_u16(self.0, bytes)
    }
}

#[cfg(test)]
mod tests {
    
}
