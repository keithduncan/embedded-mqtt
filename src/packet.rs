use core::{cmp::min, convert::TryFrom, default::Default, result::Result};

use crate::{
    codec::{Decodable, Encodable},
    error::{DecodeError, EncodeError},
    fixed_header::{self, FixedHeader},
    payload::{self, Payload},
    qos,
    status::Status,
    variable_header::{self, VariableHeader},
};

#[derive(Debug)]
#[allow(dead_code)]
pub struct Packet<'a> {
    fixed_header: FixedHeader,
    variable_header: Option<VariableHeader<'a>>,
    payload: Payload<'a>,
}

/// A full MQTT packet with fixed header, variable header and payload.
///
/// Variable header and payload are optional for some packet types.
impl<'a> Packet<'a> {
    /// Create a CONNECT packet.
    pub fn connect(
        variable_header: variable_header::connect::Connect<'a>,
        payload: payload::connect::Connect<'a>,
    ) -> Result<Self, EncodeError> {
        Self::packet(
            fixed_header::PacketType::Connect,
            fixed_header::PacketFlags::CONNECT,
            Some(variable_header::VariableHeader::Connect(variable_header)),
            payload::Payload::Connect(payload),
        )
    }

    /// Create a SUBSCRIBE packet.
    pub fn subscribe(
        variable_header: variable_header::packet_identifier::PacketIdentifier,
        payload: payload::subscribe::Subscribe<'a>,
    ) -> Result<Self, EncodeError> {
        Self::packet(
            fixed_header::PacketType::Subscribe,
            fixed_header::PacketFlags::SUBSCRIBE,
            Some(variable_header::VariableHeader::Subscribe(variable_header)),
            payload::Payload::Subscribe(payload),
        )
    }

    /// Create a PUBLISH packet.
    pub fn publish(
        flags: fixed_header::PublishFlags,
        variable_header: variable_header::publish::Publish<'a>,
        payload: &'a [u8],
    ) -> Result<Self, EncodeError> {
        // TODO encode this using type states
        assert!(
            flags.qos().expect("valid qos") == qos::QoS::AtMostOnce
                || variable_header.packet_identifier().is_some()
        );

        Self::packet(
            fixed_header::PacketType::Publish,
            flags.into(),
            Some(variable_header::VariableHeader::Publish(variable_header)),
            payload::Payload::Bytes(payload),
        )
    }

    pub fn puback(
        variable_header: variable_header::packet_identifier::PacketIdentifier,
    ) -> Result<Self, EncodeError> {
        Self::packet(
            fixed_header::PacketType::Puback,
            fixed_header::PacketFlags::PUBACK,
            Some(variable_header::VariableHeader::Puback(variable_header)),
            Default::default(),
        )
    }

    /// Create a PINGREQ packet.
    pub fn pingreq() -> Self {
        Self {
            fixed_header: FixedHeader::new(
                fixed_header::PacketType::Pingreq,
                fixed_header::PacketFlags::PINGREQ,
                0,
            ),
            variable_header: None,
            payload: Default::default(),
        }
    }

    /// Create a PINGRESP packet.
    pub fn pingresp() -> Self {
        Self {
            fixed_header: FixedHeader::new(
                fixed_header::PacketType::Pingresp,
                fixed_header::PacketFlags::PINGRESP,
                0,
            ),
            variable_header: None,
            payload: Default::default(),
        }
    }

    /// Create a packet with the given type, flags, variable header and payload.
    ///
    /// Constructs a fixed header with the appropriate `len` field for the given
    /// variable header and payload.
    fn packet(
        r#type: fixed_header::PacketType,
        flags: fixed_header::PacketFlags,
        variable_header: Option<VariableHeader<'a>>,
        payload: Payload<'a>,
    ) -> Result<Self, EncodeError> {
        let len = u32::try_from(
            variable_header
                .as_ref()
                .map(VariableHeader::encoded_len)
                .unwrap_or(0)
                + payload.encoded_len(),
        )?;

        Ok(Self {
            fixed_header: FixedHeader::new(r#type, flags, len),
            variable_header: variable_header,
            payload: payload,
        })
    }

    /// Return a reference to the fixed header of the packet.
    ///
    /// The len field of the returned header will be valid.
    pub fn fixed_header(&self) -> &FixedHeader {
        &self.fixed_header
    }

    /// Return a reference to the variable header of the packet.
    pub fn variable_header(&self) -> &Option<VariableHeader> {
        &self.variable_header
    }

    /// Return a reference to the payload of the packet.
    pub fn payload(&self) -> &Payload {
        &self.payload
    }
}

impl<'a> Decodable<'a> for Packet<'a> {
    /// Decode any MQTT packet from a pre-allocated buffer.
    ///
    /// If an unrecoverable error occurs an `Err(x)` is returned, the caller should
    /// disconnect and network connection and discard the contents of the connection
    /// receive buffer.
    ///
    /// Decoding may return an `Ok(Status::Partial(x))` in which case the caller
    /// should buffer at most `x` more bytes and then attempt decoding again.
    ///
    /// If decoding succeeds an `Ok(Status::Complete(x))` will be returned
    /// containing the number of bytes read from the buffer and the decoded packet.
    /// The lifetime of the decoded packet is tied to the input buffer.
    fn decode(bytes: &'a [u8]) -> Result<Status<(usize, Self)>, DecodeError> {
        let (fixed_header_offset, fixed_header) = read!(FixedHeader::decode, bytes, 0);

        let (variable_header_consumed, variable_header) = if let Some(result) =
            VariableHeader::decode(
                fixed_header.r#type(),
                fixed_header.flags(),
                &bytes[fixed_header_offset..],
            ) {
            let (variable_header_offset, variable_header) = complete!(result);
            (variable_header_offset, Some(variable_header))
        } else {
            (0, None)
        };

        let payload_len = fixed_header.len() as usize - variable_header_consumed;

        let available = bytes.len() - (fixed_header_offset + variable_header_consumed);
        let needed = payload_len - min(available, payload_len);
        if needed > 0 {
            return Ok(Status::Partial(needed));
        }

        let payload_bytes = &bytes[fixed_header_offset + variable_header_consumed
            ..fixed_header_offset + variable_header_consumed + payload_len];

        let payload = if let Some(result) = Payload::decode(fixed_header.r#type(), payload_bytes) {
            match result {
                Err(e) => return Err(e),
                Ok(Status::Partial(n)) => return Ok(Status::Partial(n)),
                Ok(Status::Complete((_, payload))) => payload,
            }
        } else {
            payload::Payload::Bytes(payload_bytes)
        };

        Ok(Status::Complete((
            fixed_header_offset + fixed_header.len() as usize,
            Self {
                fixed_header,
                variable_header,
                payload,
            },
        )))
    }
}

impl<'a> Encodable for Packet<'a> {
    /// Calculate the exact length of the fully encoded packet.
    ///
    /// The encode buffer will need to hold at least this number of bytes.
    fn encoded_len(&self) -> usize {
        self.fixed_header.encoded_len() + self.fixed_header.len() as usize
    }

    /// Encode a packet for sending over a network connection.
    ///
    /// If encoding fails an `Err(x)` is returned.
    ///
    /// If encoding succeeds an `Ok(written)` is returned with the number of
    /// bytes written to the buffer.
    fn encode(&self, bytes: &mut [u8]) -> Result<usize, EncodeError> {
        let mut offset = 0;

        offset += self.fixed_header.encode(&mut bytes[offset..])?;
        if let Some(ref variable_header) = self.variable_header {
            offset += variable_header.encode(&mut bytes[offset..])?;
        }
        offset += self.payload.encode(&mut bytes[offset..])?;

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
            variable_header::publish::Publish::new("a/b", Some(publish_id)),
            payload,
        )
        .expect("valid packet");

        assert_eq!(11, publish.encoded_len());
        assert_eq!(2, publish.fixed_header().encoded_len());
        assert_eq!(9, publish.fixed_header().len());
        assert_eq!(
            7,
            publish
                .variable_header()
                .as_ref()
                .expect("variable header")
                .encoded_len()
        );
        assert_eq!(2, publish.payload().encoded_len());
    }

    #[test]
    fn encode_subscribe() {
        let subscribe_id = 1;
        let sub = Packet::subscribe(
            variable_header::packet_identifier::PacketIdentifier::new(subscribe_id),
            payload::subscribe::Subscribe::new(&[
                ("c/a", qos::QoS::AtMostOnce),
                ("c/b", qos::QoS::AtLeastOnce),
                ("c/c", qos::QoS::ExactlyOnce),
            ]),
        )
        .expect("valid packet");

        assert_eq!(22, sub.encoded_len());
        assert_eq!(2, sub.fixed_header().encoded_len());
        assert_eq!(20, sub.fixed_header().len());
        assert_eq!(
            2,
            sub.variable_header()
                .as_ref()
                .expect("variable header")
                .encoded_len()
        );
        assert_eq!(18, sub.payload().encoded_len());
    }
}
