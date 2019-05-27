use core::{
    cmp::min,
};

use crate::{
    fixed_header::FixedHeader,
    result::Result,
    status::Status,
};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum PacketType {
    Connect,
    Connack,
    Publish,
    Puback,
    Pubrec,
    Pubrel,
    Pubcomp,
    Subscribe,
    Suback,
    Unsubscribe,
    Unsuback,
    Pingreq,
    Pingresp,
    Disconnect,
}

pub type PacketFlags = u8;
pub type PacketId = u16;

#[derive(Debug)]
#[allow(dead_code)]
pub struct Packet<'a> {
    fixed_header: FixedHeader,
    payload: &'a [u8],
}

impl<'a> Packet<'a> {
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Status<(usize, Self)>> {
        let offset = 0;

        let (offset, fixed_header) = read!(FixedHeader::from_bytes, bytes, offset);

        let available = bytes.len() - offset;

        let needed = fixed_header.len() as usize - min(available, fixed_header.len() as usize);
        if needed > 0 {
            return Ok(Status::Partial(needed));
        }

        let payload = &bytes[offset..offset+fixed_header.len() as usize];

        Ok(Status::Complete((offset + fixed_header.len() as usize, Packet {
            fixed_header,
            payload,
        })))
    }
}
