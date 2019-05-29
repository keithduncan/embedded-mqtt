use core::result::Result;

use crate::{
    status::Status,
    fixed_header::PacketType,
    error::{ParseError, EncodeError},
    codec::{Decodable, Encodable},
};

pub mod connect;
pub mod connack;
pub mod suback;

#[derive(Debug)]
pub enum VariableHeader<'a> {
	Connect(connect::Connect<'a>),
	Connack(connack::Connack),
	Suback(suback::Suback),
}

macro_rules! from_bytes {
	($fn:ident, $parser:path, $name:ident) => (
		pub fn $fn(bytes: &'a [u8]) -> Result<Status<(usize, Self)>, ParseError> {
			let (offset, var_header) = complete!($parser(bytes));
			Ok(Status::Complete((offset, VariableHeader::$name(var_header))))
		}
	)
}

impl<'a> VariableHeader<'a> {
	from_bytes!(connect, connect::Connect::from_bytes, Connect);
	from_bytes!(connack, connack::Connack::from_bytes, Connack);
	from_bytes!(suback,  suback::Suback::from_bytes,   Suback);

	pub fn from_bytes(r#type: PacketType, bytes: &'a [u8]) -> Option<Result<Status<(usize, Self)>, ParseError>> {
		match r#type {
			PacketType::Connect => Some(VariableHeader::connect(bytes)),
			PacketType::Connack => Some(VariableHeader::connack(bytes)),
			PacketType::Suback  => Some(VariableHeader::suback(bytes)),
			_ => None,
		}
	}
}

impl<'buf> Encodable for VariableHeader<'buf> {
	fn to_bytes(&self, bytes: &mut [u8]) -> Result<usize, EncodeError> {
		match self {
			&VariableHeader::Connect(ref conn) => conn.to_bytes(bytes),
			_ => unimplemented!(),
		}
	}
}
