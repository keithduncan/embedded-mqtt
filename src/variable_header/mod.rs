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
    ($($name:ident, $parser:path;)+) => (
        pub fn decode(r#type: PacketType, flags: PacketFlags, bytes: &'a [u8]) -> Option<Result<Status<(usize, Self)>, DecodeError>> {
            Some(match r#type {
                $(
                    PacketType::$name => $parser(flags, bytes).map(|s| {
                        match s {
                            Status::Complete((offset, var_header)) => {
                                Status::Complete((offset, VariableHeader::$name(var_header)))
                            },
                            Status::Partial(n) => Status::Partial(n),
                        }
                    }),
                )+
                _ => return None,
            })
        }
    )
}

impl<'a> VariableHeader<'a> {
    decode!(
        Connect,   connect::Connect::decode;
        Connack,   connack::Connack::decode;
        Subscribe, packet_identifier::PacketIdentifier::decode;
        Suback,    packet_identifier::PacketIdentifier::decode;
        Publish,   publish::Publish::decode;
        Puback,    packet_identifier::PacketIdentifier::decode;
    );
}

macro_rules! encode {
    ($($enum:ident;)+) => (
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
        Connect;
        Connack;
        Subscribe;
        Suback;
        Publish;
        Puback;
    );
}
