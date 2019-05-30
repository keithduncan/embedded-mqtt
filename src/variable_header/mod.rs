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
                match publish::Publish::decode(flags, bytes) {
                    Ok(Status::Partial(n)) => return Some(Ok(Status::Partial(n))),
                    Err(e) => return Some(Err(e)),
                    Ok(Status::Complete((offset, var_header))) => {
                        return Some(Ok(Status::Complete((offset, VariableHeader::Publish(var_header)))))
                    },
                };
            }
            _ => None,
        }
    }
}

macro_rules! encode {
    ($($enum:ident),+) => (
        fn encoded_len(&self) -> usize {
            match self {
                $( &VariableHeader::$enum(ref c) => c.encoded_len(), )+
            }
        }

        fn encode(&self, bytes: &mut [u8]) -> Result<usize, EncodeError> {
            match self {
                $( &VariableHeader::$enum(ref c) => c.encode(bytes), )+
            }
        }
    )
}

impl<'buf> Encodable for VariableHeader<'buf> {
    encode!(
        Connect,
        Connack,
        Subscribe,
        Suback,
        Publish
    );
}
