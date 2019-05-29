use core::result::Result;

use crate::{
	fixed_header::PublishFlags,
	codec::{self, Encodable},
	status::Status,
	error::{DecodeError, EncodeError},
	qos,
};

use super::PacketId;

#[derive(Debug)]
pub struct Publish<'a> {
	topic_name: &'a str,
	packet_identifier: Option<PacketId>,
}

impl<'a> Publish<'a> {
	pub fn new(topic_name: &'a str, packet_identifier: Option<PacketId>) -> Self {
		Self {
			topic_name,
			packet_identifier,
		}
	}

	pub fn from_bytes(flags: PublishFlags, bytes: &'a [u8]) -> Result<Status<(usize, Self)>, DecodeError> {
		let offset = 0;
		let (offset, topic_name) = read!(codec::string::parse_string, bytes, offset);

		let (offset, packet_identifier) = if flags.qos()? != qos::QoS::AtMostOnce {
			let (offset, packet_identifier) = read!(codec::values::parse_u16, bytes, offset);
			(offset, Some(packet_identifier))
		} else {
			(offset, None)
		};

		Ok(Status::Complete((offset, Self {
			topic_name,
			packet_identifier
		})))
	}

	pub fn topic_name(&self) -> &'a str {
		self.topic_name
	}

	pub fn packet_identifier(&self) -> Option<PacketId> {
		self.packet_identifier
	}
}

impl<'a> Encodable for Publish<'a> {
	fn encoded_len(&self) -> usize {
		self.topic_name.encoded_len() + self.packet_identifier.map(|_| 2).unwrap_or(0)
	}

	fn to_bytes(&self, bytes: &mut [u8]) -> Result<usize, EncodeError> {
		let offset = 0;
		let offset = {
			let o = self.topic_name.to_bytes(&mut bytes[offset..])?;
			#[cfg(feature = "std")]
			println!("topic_name {:?}", self.topic_name);
			offset + o
		};
		let offset = if let Some(packet_identifier) = self.packet_identifier {
			let o = codec::values::encode_u16(packet_identifier, bytes)?;
			offset + o
		} else {
			offset
		};
		Ok(offset)
	}
}
