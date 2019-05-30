use core::result::Result;

use crate::{
    status::Status,
    fixed_header::{PacketType, PacketFlags},
    error::{DecodeError, EncodeError},
    codec::Encodable,
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
    Puback(packet_identifier::PacketIdentifier),
}

pub trait HeaderDecode<'a>
    where Self: core::marker::Sized {
    fn decode(flags: PacketFlags, bytes: &'a [u8]) -> Result<Status<(usize, Self)>, DecodeError>;
}

pub type PacketId = u16;

macro_rules! decode {
    ($fn:ident, $parser:path, $name:ident) => (
        fn $fn(flags: PacketFlags, bytes: &'a [u8]) -> Result<Status<(usize, Self)>, DecodeError> {
            let (offset, var_header) = complete!($parser(flags, bytes));
            Ok(Status::Complete((offset, VariableHeader::$name(var_header))))
        }
    )
}

impl<'a> VariableHeader<'a> {
    decode!(connect,   connect::Connect::decode,                    Connect);
    decode!(connack,   connack::Connack::decode,                    Connack);
    decode!(subscribe, packet_identifier::PacketIdentifier::decode, Subscribe);
    decode!(suback,    packet_identifier::PacketIdentifier::decode, Suback);
    decode!(publish,   publish::Publish::decode,                    Publish);
    decode!(puback,    packet_identifier::PacketIdentifier::decode, Puback);

    pub fn decode(r#type: PacketType, flags: PacketFlags, bytes: &'a [u8]) -> Option<Result<Status<(usize, Self)>, DecodeError>> {
        match r#type {
            PacketType::Connect   => Some(Self::connect(flags, bytes)),
            PacketType::Connack   => Some(Self::connack(flags, bytes)),
            PacketType::Subscribe => Some(Self::subscribe(flags, bytes)),
            PacketType::Suback    => Some(Self::suback(flags, bytes)),
            PacketType::Publish   => Some(Self::publish(flags, bytes)),
            PacketType::Puback    => Some(Self::puback(flags, bytes)),
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
        Publish,
        Puback
    );
}
