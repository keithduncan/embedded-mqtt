use core::{
	result::Result,
};

use crate::{
	status::Status,
	codec::{Decodable, Encodable},
	error::{DecodeError, EncodeError},
};

#[derive(Debug)]
pub struct Publish<'a> {
	bytes: &'a [u8],
}

impl<'a> Publish<'a> {
	pub fn new(bytes: &'a [u8]) -> Self {
		Self {
			bytes,
		}
	}
}

impl<'a> Decodable<'a> for Publish<'a> {
	fn decode(bytes: &'a [u8]) -> Result<Status<(usize, Self)>, DecodeError> {
		let len = bytes.len();
		Ok(Status::Complete((len, Self {
			bytes: &bytes[..],
		})))
	}
}

impl<'a> Encodable for Publish<'a> {
	fn encoded_len(&self) -> usize {
		self.bytes.len()
	}

	fn encode(&self, bytes: &mut [u8]) -> Result<usize, EncodeError> {
		if bytes.len() < self.bytes.len() {
			return Err(EncodeError::OutOfSpace)
		}

		(&mut bytes[..self.bytes.len()]).copy_from_slice(self.bytes);

		return Ok(self.bytes.len())
	}
}

impl<'a> From<&'a [u8]> for Publish<'a> {
	fn from(bytes: &'a [u8]) -> Self {
		Self {
			bytes,
		}
	}
}
