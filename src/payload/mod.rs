use core::{
	result::Result,
	default::Default,
};

use crate::{
	codec::Encodable,
	error::EncodeError,
};

pub mod connect;
pub mod subscribe;

#[derive(Debug)]
pub enum Payload<'a> {
	Bytes(&'a [u8]),
	Connect(connect::Connect<'a>),
	Subscribe(subscribe::Subscribe<'a>),
}

impl<'a> Encodable for Payload<'a> {
	fn encoded_len(&self) -> usize {
		match self {
			&Payload::Bytes(ref c)     => c.len(),
			&Payload::Connect(ref c)   => c.encoded_len(),
			&Payload::Subscribe(ref c) => c.encoded_len(),
		}
	}

	fn encode(&self, bytes: &mut [u8]) -> Result<usize, EncodeError> {
		match self {
			&Payload::Bytes(ref c)   => {
				if bytes.len() < c.len() {
					return Err(EncodeError::OutOfSpace)
				}

				(&mut bytes[0..c.len()]).copy_from_slice(c);

				Ok(c.len())
			},
			&Payload::Connect(ref c)   => c.encode(bytes),
			&Payload::Subscribe(ref c) => c.encode(bytes),
		}
	}
}

impl<'a> Default for Payload<'a> {
	fn default() -> Self {
		Payload::Bytes(&[])
	}
}
