use core::cmp::min;

use crate::{
    fixed_header::FixedHeader,
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
        let (fixed_header_offset, fixed_header) = read!(FixedHeader::from_bytes, bytes, 0);

        // TODO this is only duplicated while not all types have their
        // variable header parsed.
        let (variable_header, payload) = if let Some(result) = VariableHeader::from_bytes(fixed_header.r#type(), &bytes[fixed_header_offset..]) {
            let (variable_header_offset, variable_header) = match result {
                Err(e) => return Err(e),
                Ok(Status::Partial(p)) => return Ok(Status::Partial(p)),
                Ok(Status::Complete(x)) => x,
            };
            let variable_header_consumed = variable_header_offset;
            #[cfg(feature = "std")]
            println!("variable_header_consumed {:?}", variable_header_consumed);

            let payload_len = fixed_header.len() as usize - variable_header_consumed;
            #[cfg(feature = "std")]
            println!("payload_len {:?}", payload_len);

            let available = bytes.len() - (fixed_header_offset + variable_header_offset);
            let needed = payload_len - min(available, payload_len);
            if needed > 0 {
                return Ok(Status::Partial(needed));
            }
            let payload = &bytes[fixed_header_offset+variable_header_offset..fixed_header_offset+variable_header_offset+payload_len];

            (Some(variable_header), payload)
        } else {
            let available = bytes.len() - fixed_header_offset;
            let needed = fixed_header.len() as usize - min(available, fixed_header.len() as usize);
            if needed > 0 {
                return Ok(Status::Partial(needed));
            }
            let payload = &bytes[fixed_header_offset..fixed_header_offset+fixed_header.len() as usize];

            (None, payload)
        };

        Ok(Status::Complete((fixed_header_offset + fixed_header.len() as usize, Packet {
            fixed_header,
            variable_header,
            payload,
        })))
    }
}
