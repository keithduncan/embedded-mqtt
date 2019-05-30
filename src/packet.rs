use core::{
    convert::TryFrom,
    cmp::min,
    result::Result,
};

use crate::{
    fixed_header::{self, FixedHeader},
    variable_header::{self, HeaderDecode},
    payload,
    status::Status,
    error::{DecodeError, EncodeError},
    codec::{Decodable, Encodable},
    qos,
};

#[derive(Debug)]
#[allow(dead_code)]
pub enum Packet<'a> {
    Connect(FixedHeader, variable_header::Connect<'a>, payload::connect::Connect<'a>),
    Subscribe(FixedHeader, variable_header::PacketIdentifier, payload::subscribe::Subscribe<'a>),
    Publish(FixedHeader, variable_header::Publish<'a>, payload::Publish<'a>),
    Pingreq(FixedHeader),
    Pingresp(FixedHeader),
}

/// A full MQTT packet with fixed header, variable header and payload.
///
/// Variable header and payload are optional for some packet types.
impl<'a> Packet<'a> {
    /// Create a CONNECT packet.
    pub fn connect(variable_header: variable_header::Connect<'a>, payload: payload::Connect<'a>) -> Result<Self, EncodeError> {
        let fields = Self::packet(
            fixed_header::PacketType::Connect,
            fixed_header::PacketFlags::CONNECT,
            variable_header,
            payload
        )?;
        Ok(Packet::Connect(fields.0, fields.1, fields.2))
    }

    /// Create a SUBSCRIBE packet.
    pub fn subscribe(variable_header: variable_header::PacketIdentifier, payload: payload::Subscribe<'a>) -> Result<Self, EncodeError> {
        let fields = Self::packet(
            fixed_header::PacketType::Subscribe,
            fixed_header::PacketFlags::SUBSCRIBE,
            variable_header,
            payload,
        )?;
        Ok(Packet::Subscribe(fields.0, fields.1, fields.2))
    }

    /// Create a PUBLISH packet.
    pub fn publish(flags: fixed_header::PublishFlags, variable_header: variable_header::Publish<'a>, payload: payload::Publish<'a>) -> Result<Self, EncodeError> {
        // TODO encode this using type states
        assert!(flags.qos().expect("valid qos") == qos::QoS::AtMostOnce || variable_header.packet_identifier().is_some());

        let fields = Self::packet(
            fixed_header::PacketType::Publish,
            flags.into(),
            variable_header,
            payload
        )?;
        Ok(Packet::Publish(fields.0, fields.1, fields.2))
    }

    /// Create a PINGREQ packet.
    pub fn pingreq() -> Self {
        Packet::Pingreq(
            FixedHeader::new(
                fixed_header::PacketType::Pingreq,
                fixed_header::PacketFlags::PINGREQ,
                0,
            )
        )
    }

    /// Create a PINGRESP packet.
    pub fn pingresp() -> Self {
        Packet::Pingresp(
            FixedHeader::new(
                fixed_header::PacketType::Pingresp,
                fixed_header::PacketFlags::PINGRESP,
                0,
            )
        )
    }

    /// Create a packet with the given type, flags, variable header and payload.
    ///
    /// Constructs a fixed header with the appropriate `len` field for the given
    /// variable header and payload.
    fn packet<VH, P>(r#type: fixed_header::PacketType, flags: fixed_header::PacketFlags, variable_header: VH, payload: P) -> Result<(FixedHeader, VH, P), EncodeError>
        where VH: Encodable, P: Encodable, {
        let len = u32::try_from(
            variable_header.encoded_len() +
            payload.encoded_len()
        )?;

        Ok((
            FixedHeader::new(
                r#type,
                flags,
                len,
            ),
            variable_header,
            payload,
        ))
    }

    /// Return a reference to the fixed header of the packet.
    ///
    /// The len field of the returned header will be valid.
    pub fn fixed_header(&self) -> &FixedHeader {
        match self {
            &Packet::Connect(fh, ..) => &fh,
            &Packet::Subscribe(fh, ..) => &fh,
            &Packet::Publish(fh, ..) => &fh,
            &Packet::Pingreq(fh, ..) => &fh,
            &Packet::Pingresp(fh, ..) => &fh,
        }
    }

    fn variable_header<VH>(&self) -> Option<&VH> where VH: Encodable {
        match self {
            &Packet::Connect(_, vh, ..) => Some(&vh),
            &Packet::Subscribe(_, vh, ..) => Some(&vh),
            &Packet::Publish(_, vh, ..) => Some(&vh),
            &Packet::Pingreq(_) => None,
            &Packet::Pingresp(_) => None,
        }
    }

    fn payload<P>(&self) -> Option<P> where P: Encodable {
        match self {
            &Packet::Connect(_, _, p) => Some(&p),
            &Packet::Subscribe(_, _, p) => Some(&p),
            &Packet::Publish(_, _, p) => Some(&p),
            &Packet::Pingreq(_) => None,
            &Packet::Pingresp(_) => None,
        }
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

        // TODO this is only duplicated while not all types have their
        // variable header parsed.
        let (variable_header, payload) = if let Some(result) = VH::decode(fixed_header.flags(), &bytes[fixed_header_offset..]) {
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

        let payload = Some(payload);

        Ok(Status::Complete((fixed_header_offset + fixed_header.len() as usize, Self {
            fixed_header,
            variable_header,
            payload,
        })))
    }
}

impl<'a> Encodable for Packet<'a> {
    /// Calculate the exact length of the fully encoded packet.
    ///
    /// The encode buffer will need to hold at least this number of bytes.
    fn encoded_len(&self) -> usize {
        self.fixed_header().encoded_len() + self.fixed_header().len() as usize
    }

    /// Encode a packet for sending over a network connection.
    ///
    /// If encoding fails an `Err(x)` is returned.
    ///
    /// If encoding succeeds an `Ok(written)` is returned with the number of
    /// bytes written to the buffer.
    fn encode(&self, bytes: &mut [u8]) -> Result<usize, EncodeError> {
        let mut offset = 0;

        offset = {
            let o = self.fixed_header().encode(&mut bytes[offset..])?;
            offset + o
        };

        if let Some(ref variable_header) = self.variable_header() {
            offset = {
                let o = variable_header.encode(&mut bytes[offset..])?;
                offset + o
            };
        }

        let offset = if let Some(ref payload) = self.payload() {
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