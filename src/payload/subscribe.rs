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
		write!(f, "Subscribe {{\n")?;
		self.topics()
			.fold(Ok(()), |acc, (topic, qos)| {
				acc?;
				write!(f, "    (\n        Topic: {:#?},\n        QoS: {:#?}\n    )\n", topic, qos)
			})?;
		write!(f, "}}")?;

		Ok(())
	}
}

fn parse_subscription<'a>(bytes: &'a [u8]) -> Result<Status<(usize, (&'a str, qos::QoS))>, DecodeError> {
	let offset = 0;

	let (offset, topic) = {
		let (o, topic) = complete!(codec::string::parse_string(&bytes[offset..]));
		(offset + o, topic)
	};

	let (offset, qos) = {
		let (o, qos) = complete!(codec::values::parse_u8(&bytes[offset..]));
		let qos = qos::QoS::try_from(qos)?;
		(offset + o, qos)
	};

	Ok(Status::Complete((offset, (topic, qos))))
}

impl<'a> Decodable<'a> for Subscribe<'a> {
	fn decode(bytes: &'a [u8]) -> Result<Status<(usize, Self)>, DecodeError> {
		let mut offset = 0;
		while offset < bytes.len() {
			let o = match parse_subscription(&bytes[offset..]) {
				Err(e) => return Err(e),
				Ok(Status::Partial(..)) => return Err(DecodeError::InvalidLength),
				Ok(Status::Complete((o, _))) => o,
			};
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

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn decode_literal() {
		let topics = [
			("a", qos::QoS::AtMostOnce),
			("b", qos::QoS::AtLeastOnce),
			("c", qos::QoS::ExactlyOnce),
		];

		let sub = Subscribe::new(&topics);

		let mut iter = sub.topics();

		let next = iter.next();
		assert_eq!(next, Some(("a", qos::QoS::AtMostOnce)));

		let next = iter.next();
		assert_eq!(next, Some(("b", qos::QoS::AtLeastOnce)));

		let next = iter.next();
		assert_eq!(next, Some(("c", qos::QoS::ExactlyOnce)));

		let next = iter.next();
		assert_eq!(next, None);
	}

	#[test]
	fn decode_bytes() {
		let bytes = [
			0b0000_0000, // 1
			0b0000_0001,
			0x61,        // 'a'
			0x0000_0000, // AtMostOnce

			0b0000_0000, // 1
			0b0000_0001,
			0x62,        // 'b'
			0b0000_0001, // AtLeastOnce

			0b0000_0000, // 1
			0b0000_0001,
			0x63,        // 'c'
			0b0000_0010, // ExactlyOnce
		];

		let (_, sub) = Subscribe::decode(&bytes).expect("valid").unwrap();

		let mut iter = sub.topics();

		let next = iter.next();
		assert_eq!(next, Some(("a", qos::QoS::AtMostOnce)));

		let next = iter.next();
		assert_eq!(next, Some(("b", qos::QoS::AtLeastOnce)));

		let next = iter.next();
		assert_eq!(next, Some(("c", qos::QoS::ExactlyOnce)));

		let next = iter.next();
		assert_eq!(next, None);
	}

	#[test]
	fn decode_bytes_error() {
		let bytes = [
			0b0000_0000, // 1
			0b0000_0001,
			0x61,        // 'a'
			0x0000_0000, // AtMostOnce

			0b0000_0000, // 1
			0b0000_0001,
			0x62,        // 'b'
			0b0000_0001, // AtLeastOnce

			0b0000_0000, // 1
			0b0000_0001,
			0x63,        // 'c'

			// Intentionally omitted
			//0b0000_0010, // ExactlyOnce
			//
		];

		let sub = Subscribe::decode(&bytes);
		assert!(sub.is_err());
		assert_eq!(sub.unwrap_err(), DecodeError::InvalidLength);
	}
}
