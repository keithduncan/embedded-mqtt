use crate::{
    status::Status,
    result::Result,
    fixed_header::PacketType,
};

pub mod connect;
pub mod connack;

#[derive(Debug)]
pub enum VariableHeader<'a> {
	Connect(connect::Connect<'a>),
	Connack(connack::Connack),
}

macro_rules! from_bytes {
	($fn:ident, $parser:path, $name:ident) => (
		pub fn $fn(bytes: &'a [u8]) -> Result<Status<(usize, Self)>> {
			let (offset, connect) = complete!($parser(bytes));
			Ok(Status::Complete((offset, VariableHeader::$name(connect))))
		}
	)
}

impl<'a> VariableHeader<'a> {
	pub fn from_bytes(r#type: PacketType, bytes: &'a [u8]) -> Option<Result<Status<(usize, Self)>>> {
		match r#type {
			PacketType::Connect => Some(VariableHeader::connect(bytes)),
			PacketType::Connack => Some(VariableHeader::connack(bytes)),
			_ => None,
		}
	}

	from_bytes!(connect, connect::Connect::from_bytes, Connect);
	from_bytes!(connack, connack::Connack::from_bytes, Connack);
}
