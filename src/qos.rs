use core::convert::TryFrom;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum QoS {
    AtMostOnce,
    AtLeastOnce,
    ExactlyOnce,
}

#[derive(PartialEq, Debug)]
pub enum Error {
	BadPattern,
}

impl TryFrom<u8> for QoS {
	type Error = Error;
	
	fn try_from(byte: u8) -> core::result::Result<QoS, Error> {
		let qos = match byte & 0b11 {
			0b00 => QoS::AtMostOnce,
			0b01 => QoS::AtLeastOnce,
			0b10 => QoS::ExactlyOnce,
			_ => return Err(Error::BadPattern),
		};

		Ok(qos)
	}
}