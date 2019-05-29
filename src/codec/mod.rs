use crate::{
	status::Status,
	error::{DecodeError, EncodeError},
};

pub mod string;
pub mod values;

pub trait Decodable<'a>
	where Self: core::marker::Sized {
	fn from_bytes(bytes: &'a [u8]) -> Result<Status<(usize, Self)>, DecodeError>;
}

pub trait Encodable {
	fn to_bytes(&self, bytes: &mut [u8]) -> Result<usize, EncodeError>;
}
