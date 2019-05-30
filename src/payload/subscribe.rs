use core::{
	fmt,
	result::Result,
	iter::Iterator,
	convert::{From, TryFrom},
};

use crate::{
	codec::{self, Decodable, Encodable},
	error::{DecodeError, EncodeError},
	qos,
	status::Status,
};

pub struct Iter<'a> {
	offset: usize,
	sub: &'a Subscribe<'a>,
}

impl<'a> Iter<'a> {
	fn new(sub: &'a Subscribe<'a>) -> Self {
		Iter {
			offset: 0,
			sub,
		}
	}
}

impl<'a> Iterator for Iter<'a> {
	type Item = (&'a str, qos::QoS);
	fn next(&mut self) -> Option<Self::Item> {
		match self.sub {
			&Subscribe::Encode(topics) => {
				// Offset is an index into the encode slice
				if self.offset >= topics.len() {
					return None
				}

				let item = topics[self.offset];
				self.offset += 1;

				Some(item)
			},
			&Subscribe::Decode(bytes) => {
				// Offset is a byte offset in the byte slice
				if self.offset >= bytes.len() {
					return None
				}

				// &bytes[offset..] points to a length, string and QoS
				let (o, item) = parse_subscription(&bytes[self.offset..]).expect("already validated").unwrap();
				self.offset += o;

				Some(item)
			}
		}
	}
}

pub enum Subscribe<'a> {
	Encode(&'a [(&'a str, qos::QoS)]),
	Decode(&'a [u8]),
}

impl<'a> Subscribe<'a> {
	pub fn new(topics: &'a [(&'a str, qos::QoS)]) -> Self {
		Subscribe::Encode(topics)
	}

	pub fn topics(&self) -> Iter {
		Iter::new(self)
	}
}

impl<'a> fmt::Debug for Subscribe<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "Subscribe {{")?;
		self.topics()
			.fold(Ok(()), |acc, (topic, qos)| {
				acc?;
				write!(f, "Topic {:#?}, QoS {:#?}", topic, qos)
			})?;
		write!(f, "}}")?;

		Ok(())
	}
}

fn parse_subscription<'a>(bytes: &'a [u8]) -> Result<Status<(usize, (&'a str, qos::QoS))>, DecodeError> {
	let offset = 0;

	let (offset, topic) = {
		let (o, topic) = read!(codec::string::parse_string, bytes, offset);
		(offset + o, topic)
	};

	let (offset, qos) = {
		let (o, qos) = read!(codec::values::parse_u8, bytes, offset);
		let qos = qos::QoS::try_from(qos)?;
		(offset + o, qos)
	};

	Ok(Status::Complete((offset, (topic, qos))))
}

impl<'a> Decodable<'a> for Subscribe<'a> {
	fn decode(bytes: &'a [u8]) -> Result<Status<(usize, Self)>, DecodeError> {
		let mut offset = 0;
		while offset < bytes.len() {
			let (o, _) = read!(parse_subscription, bytes, offset);
			offset += o;
		}

		Ok(Status::Complete((bytes.len(), Subscribe::Decode(bytes))))
	}
}

impl<'a> Encodable for Subscribe<'a> {
	fn encoded_len(&self) -> usize {
		self.topics()
			.map(|topic| {
				topic.0.encoded_len() + 1
			})
			.sum()
	}

	fn encode(&self, bytes: &mut [u8]) -> Result<usize, EncodeError> {
		self.topics()
			.fold(Ok(0), |acc, (topic, qos)| {
				let offset = acc?;
				let offset = {
					let o = codec::string::encode_string(topic, &mut bytes[offset..])?;
					offset + o
				};
				let offset = {
					let o = codec::values::encode_u8(u8::from(qos), &mut bytes[offset..])?;
					offset + o
				};
				Ok(offset)
			})
	}
}
