use core::result::Result;

use crate::{
	status::Status,
	fixed_header::PacketFlags,
	error::DecodeError,
};

pub use self::{
    connect::Connect,
    connack::Connack,
    packet_identifier::PacketIdentifier,
    publish::Publish,
};

pub mod connect;
pub mod connack;
pub mod packet_identifier;
pub mod publish;

pub type PacketId = u16;

pub trait HeaderDecode<'a>
	where Self: core::marker::Sized {
	fn decode(flags: PacketFlags, bytes: &'a [u8]) -> Result<Status<(usize, Self)>, DecodeError>;
}
