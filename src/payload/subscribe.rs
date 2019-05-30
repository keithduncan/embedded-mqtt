use core::result::Result;

use crate::{
	codec::{self, Encodable},
	error::EncodeError,
	qos::QoS,
};

#[derive(Debug)]
pub struct Subscribe<'a> {
	topics: &'a [(&'a str, QoS)]
}

impl<'a> Subscribe<'a> {
	pub fn new(topics: &'a [(&'a str, QoS)]) -> Self {
		Self {
			topics,
		}
	}
}

impl<'a> Encodable for Subscribe<'a> {
	fn encoded_len(&self) -> usize {
		self.topics
			.iter()
			.map(|topic| {
				topic.0.encoded_len() + 1
			})
			.sum()
	}

	fn encode(&self, bytes: &mut [u8]) -> Result<usize, EncodeError> {
		self.topics
			.iter()
			.fold(Ok(0), |acc, (topic, qos)| {
				let offset = acc?;
				let offset = {
					let o = codec::string::encode_string(topic, &mut bytes[offset..])?;
					offset + o
				};
				let offset = {
					let o = codec::values::encode_u8(u8::from(*qos), &mut bytes[offset..])?;
					offset + o
				};
				Ok(offset)
			})
	}
}
