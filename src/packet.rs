use core::{
    convert::TryFrom,
    cmp::min,
    result::Result,
};

use crate::{
    fixed_header::{self, FixedHeader},
    variable_header::{self, VariableHeader},
    payload::{self, Payload},
    status::Status,
    error::{DecodeError, EncodeError},
    codec::{Decodable, Encodable},
    qos,
};

#[derive(Debug)]
#[allow(dead_code)]
pub struct Packet<'a> {
    fixed_header: FixedHeader,
    variable_header: Option<VariableHeader<'a>>,
    payload: Option<Payload<'a>>,
}

impl<'a> Packet<'a> {
    pub fn connect(variable_header: variable_header::connect::Connect<'a>, payload: payload::connect::Connect<'a>) -> Result<Self, EncodeError> {
        Self::packet(
            fixed_header::PacketType::Connect,
            fixed_header::PacketFlags::connect(),
            Some(variable_header::VariableHeader::Connect(variable_header)),
            Some(payload::Payload::Connect(payload))
        )
    }

    pub fn subscribe(variable_header: variable_header::packet_identifier::PacketIdentifier, payload: payload::subscribe::Subscribe<'a>) -> Result<Self, EncodeError> {
        Self::packet(
            fixed_header::PacketType::Subscribe,
            fixed_header::PacketFlags::subscribe(),
            Some(variable_header::VariableHeader::Subscribe(variable_header)),
            Some(payload::Payload::Subscribe(payload)),
        )
    }

    pub fn publish(flags: fixed_header::PublishFlags, variable_header: variable_header::publish::Publish<'a>, payload: &'a [u8]) -> Result<Self, EncodeError> {
        // TODO encode this using type states
        assert!(flags.qos().expect("valid qos") == qos::QoS::AtMostOnce || variable_header.packet_identifier().is_some());

        Self::packet(
            fixed_header::PacketType::Publish,
            flags.into(),
            Some(variable_header::VariableHeader::Publish(variable_header)),
            Some(payload::Payload::Bytes(payload))
        )
    }

    pub fn pingreq() -> Self {
        Self {
            fixed_header: FixedHeader::new(
                fixed_header::PacketType::Pingreq,
                fixed_header::PacketFlags::pingreq(),
                0,
            ),
            variable_header: None,
            payload: None,
        }
    }

    pub fn pingresp() -> Self {
        Self {
            fixed_header: FixedHeader::new(
                fixed_header::PacketType::Pingresp,
                fixed_header::PacketFlags::pingresp(),
                0,
            ),
            variable_header: None,
            payload: None,
        }
    }

    fn packet(r#type: fixed_header::PacketType, flags: fixed_header::PacketFlags, variable_header: Option<VariableHeader<'a>>, payload: Option<Payload<'a>>) -> Result<Self, EncodeError> {
        let len = u32::try_from(
            variable_header.as_ref().map(VariableHeader::encoded_len).unwrap_or(0) +
            payload.as_ref().map(Payload::encoded_len).unwrap_or(0)
        )?;

        Ok(Self {
            fixed_header: FixedHeader::new(
                r#type,
                flags,
                len,
            ),
            variable_header: variable_header,
            payload: payload,
        })
    }

    pub fn fixed_header(&self) -> &FixedHeader {
        &self.fixed_header
    }

    pub fn variable_header(&self) -> &Option<VariableHeader> {
        &self.variable_header
    }

    pub fn payload(&self) -> &Option<Payload> {
        &self.payload
    }
}

impl<'a> Decodable<'a> for Packet<'a> {
    fn decode(bytes: &'a [u8]) -> Result<Status<(usize, Self)>, DecodeError> {
        let (fixed_header_offset, fixed_header) = read!(FixedHeader::decode, bytes, 0);

        // TODO this is only duplicated while not all types have their
        // variable header parsed.
        let (variable_header, payload) = if let Some(result) = VariableHeader::decode(fixed_header.r#type(), fixed_header.flags(), &bytes[fixed_header_offset..]) {
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

        let payload = Some(payload::Payload::Bytes(payload));

        Ok(Status::Complete((fixed_header_offset + fixed_header.len() as usize, Self {
            fixed_header,
            variable_header,
            payload,
        })))
    }
}

impl<'a> Encodable for Packet<'a> {
    fn encoded_len(&self) -> usize {
        self.fixed_header.encoded_len() + self.fixed_header.len() as usize
    }

    fn encode(&self, bytes: &mut [u8]) -> Result<usize, EncodeError> {
        let mut offset = 0;

        offset = {
            let o = self.fixed_header.encode(&mut bytes[offset..])?;
            offset + o
        };

        if let Some(ref variable_header) = self.variable_header {
            offset = {
                let o = variable_header.encode(&mut bytes[offset..])?;
                offset + o
            };
        }

        let offset = if let Some(ref payload) = self.payload {
            let o = payload.encode(&mut bytes[offset..])?;
            offset + o
        } else {
            offset
        };

        Ok(offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_publish() {
        let payload = b"{}";
        assert_eq!(2, payload.len());

        let mut publish_flags = fixed_header::PublishFlags::default();
        publish_flags.set_qos(qos::QoS::AtLeastOnce);
        let publish_id = 2;
        let publish = Packet::publish(
            publish_flags,
            variable_header::publish::Publish::new(
                "a/b",
                Some(publish_id),
            ),
            payload
        ).expect("valid packet");

        println!("{:#?}", publish);

        assert_eq!(11, publish.encoded_len());
        assert_eq!(2, publish.fixed_header().encoded_len());
        assert_eq!(9, publish.fixed_header().len());
        assert_eq!(7, publish.variable_header().as_ref().expect("variable header").encoded_len());
        assert_eq!(2, publish.payload().as_ref().expect("payload").encoded_len());
    }
}