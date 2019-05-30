use core::result::Result;

use crate::{
    status::Status,
    fixed_header::{PacketType, PacketFlags},
    error::{DecodeError, EncodeError},
    codec::{Decodable, Encodable},
};

pub mod connect;
pub mod connack;
pub mod packet_identifier;
pub mod publish;

#[derive(Debug)]
pub enum VariableHeader<'a> {
    Connect(connect::Connect<'a>),
    Connack(connack::Connack),
    Subscribe(packet_identifier::PacketIdentifier),
    Suback(packet_identifier::PacketIdentifier),
    Publish(publish::Publish<'a>),
}

pub type PacketId = u16;

macro_rules! decode {
    ($fn:ident, $parser:path, $name:ident) => (
        fn $fn(bytes: &'a [u8]) -> Result<Status<(usize, Self)>, DecodeError> {
            let (offset, var_header) = complete!($parser(bytes));
            Ok(Status::Complete((offset, VariableHeader::$name(var_header))))
        }
    )
}

impl<'a> VariableHeader<'a> {
    decode!(connect,   connect::Connect::decode,                    Connect);
    decode!(connack,   connack::Connack::decode,                    Connack);
    decode!(subscribe, packet_identifier::PacketIdentifier::decode, Subscribe);
    decode!(suback,    packet_identifier::PacketIdentifier::decode, Suback);

    pub fn decode(r#type: PacketType, flags: PacketFlags, bytes: &'a [u8]) -> Option<Result<Status<(usize, Self)>, DecodeError>> {
        match r#type {
            PacketType::Connect   => Some(VariableHeader::connect(bytes)),
            PacketType::Connack   => Some(VariableHeader::connack(bytes)),
            PacketType::Subscribe => Some(VariableHeader::subscribe(bytes)),
            PacketType::Suback    => Some(VariableHeader::suback(bytes)),
            PacketType::Publish   => {
                let (offset, var_header) = match publish::Publish::decode(flags.into(), bytes) {
                    Ok(Status::Partial(n)) => return Some(Ok(Status::Partial(n))),
                    Err(e) => return Some(Err(e)),

                    Ok(Status::Complete(x)) => x,
                };
                Some(Ok(Status::Complete((offset, VariableHeader::Publish(var_header)))))
            }
            _ => None,
        }
    }
}

impl<'buf> Encodable for VariableHeader<'buf> {
    fn encoded_len(&self) -> usize {
        match self {
            &VariableHeader::Connect(ref c)   => c.encoded_len(),
            &VariableHeader::Subscribe(ref c) => c.encoded_len(),
            &VariableHeader::Publish(ref c)   => c.encoded_len(),
            _ => unimplemented!()
        }
    }

    fn to_bytes(&self, bytes: &mut [u8]) -> Result<usize, EncodeError> {
        match self {
            &VariableHeader::Connect(ref c)   => c.to_bytes(bytes),
            &VariableHeader::Subscribe(ref c) => c.to_bytes(bytes),
            &VariableHeader::Publish(ref c)   => c.to_bytes(bytes),
            _ => unimplemented!(),
        }
    }
}
