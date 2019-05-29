use core::{
    cmp::min,
    result::Result,
};

use crate::{
    fixed_header::FixedHeader,
    variable_header::VariableHeader,
    status::Status,
    error::{ParseError, EncodeError},
    codec::{Decodable, Encodable},
};

#[derive(Debug)]
#[allow(dead_code)]
pub struct Packet<'a> {
    pub fixed_header: FixedHeader,
    pub variable_header: Option<VariableHeader<'a>>,
    pub payload: &'a [u8],
}

impl<'a> Decodable<'a> for Packet<'a> {
    fn from_bytes(bytes: &'a [u8]) -> Result<Status<(usize, Self)>, ParseError> {
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

            let payload_len = fixed_header.len() as usize - variable_header_consumed;

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

impl<'a> Encodable for Packet<'a> {
    fn to_bytes(&self, bytes: &mut [u8]) -> Result<usize, EncodeError> {
        let mut offset = 0;

        offset = {
            let o = self.fixed_header.to_bytes(&mut bytes[offset..])?;
            offset + o
        };

        if let Some(ref variable_header) = self.variable_header {
            offset = {
                let o = variable_header.to_bytes(&mut bytes[offset..])?;
                offset + o
            };
        }

        let offset = {
            let payload_size = self.payload.len();
            if offset + payload_size > bytes.len() {
                return Err(EncodeError::OutOfSpace)
            }

            (&mut bytes[offset..offset + payload_size as usize]).copy_from_slice(self.payload);

            (offset + payload_size)
        };

        Ok(offset)
    }
}
