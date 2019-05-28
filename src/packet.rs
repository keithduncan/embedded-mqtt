use core::cmp::min;

use crate::{
    fixed_header::{
        PacketType,
        FixedHeader,
    },
    variable_header::VariableHeader,
    result::Result,
    status::Status,
};

pub type PacketId = u16;

#[derive(Debug)]
#[allow(dead_code)]
pub struct Packet<'a> {
    fixed_header: FixedHeader,
    variable_header: Option<VariableHeader<'a>>,
    payload: &'a [u8],
}

impl<'a> Packet<'a> {
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Status<(usize, Self)>> {
        let offset = 0;

        let (offset, fixed_header) = read!(FixedHeader::from_bytes, bytes, offset);

        // TODO this is only duplicated while not all types have their
        // variable header parsed.
        let (variable_header, payload) = if fixed_header.r#type() == PacketType::Connect {
            let (offset, variable_header) = read!(VariableHeader::connect, bytes, offset);

            let available = bytes.len() - offset;
            let needed = fixed_header.len() as usize - min(available, fixed_header.len() as usize);
            if needed > 0 {
                return Ok(Status::Partial(needed));
            }
            let payload = &bytes[offset..offset+fixed_header.len() as usize];

            (Some(variable_header), payload)
        } else {
            let available = bytes.len() - offset;
            let needed = fixed_header.len() as usize - min(available, fixed_header.len() as usize);
            if needed > 0 {
                return Ok(Status::Partial(needed));
            }
            let payload = &bytes[offset..offset+fixed_header.len() as usize];

            (None, payload)
        };

        Ok(Status::Complete((offset + fixed_header.len() as usize, Packet {
            fixed_header,
            variable_header,
            payload,
        })))
    }
}
