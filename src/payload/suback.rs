use core::{
	result::Result,
	convert::{From, TryFrom, TryInto},
	fmt::Debug,
	mem,
};

use crate::{
	codec::{Decodable, Encodable},
	error::{DecodeError, EncodeError},
	status::Status,
	qos,
};

use bitfield::BitRange;

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct ReturnCode(u8);

bitfield_bitrange! {
    struct ReturnCode(u8)
}

impl ReturnCode {
	pub const SUCCESS_QOS_0: ReturnCode = ReturnCode(0b0000_0000);
	pub const SUCCESS_QOS_1: ReturnCode = ReturnCode(0b0000_0001);
	pub const SUCCESS_QOS_2: ReturnCode = ReturnCode(0b0000_0010);
	pub const FAILURE      : ReturnCode = ReturnCode(0b1000_0000);

    bitfield_fields! {
        bool;
        pub failure, set_failure : 7;
    }

    pub fn max_qos(&self) -> Result<qos::QoS, qos::Error> {
        let qos_bits: u8 = self.bit_range(1, 0);
        qos_bits.try_into()
    }

    #[allow(dead_code)]
    pub fn set_max_qos(&mut self, qos: qos::QoS) {
        self.set_bit_range(1, 0, u8::from(qos))
    }
}

impl Debug for ReturnCode {
    bitfield_debug! {
        struct ReturnCode;
        pub failure, _ : 7;
        pub into QoS, max_qos, _ : 1, 0;
    }
}

impl From<ReturnCode> for u8 {
    fn from(val: ReturnCode) -> u8 {
        val.0
    }
}

impl TryFrom<u8> for ReturnCode {
	type Error = ();
	fn try_from(val: u8) -> Result<Self, Self::Error> {
		if 0b0111_1100 & val != 0 {
			return Err(())
		}

		let failure = 0b1000_0000 & val;
		let success = 0b0000_0011 & val;

		if (success != 0) && (failure != 0) {
			return Err(())
		}

		Ok(ReturnCode(val))
	}
}

#[derive(PartialEq, Eq, Debug)]
pub struct Suback<'a> {
	return_codes: &'a [ReturnCode],
}

impl<'a> Suback<'a> {
	pub fn new(return_codes: &'a [ReturnCode]) -> Self {
		Self {
			return_codes,
		}
	}
}

impl<'a> Decodable<'a> for Suback<'a> {
	fn decode(bytes: &'a [u8]) -> Result<Status<(usize, Self)>, DecodeError> {
		// Check all the bytes are valid return codes
		bytes.iter()
			.fold(Ok(()), |acc, byte| {
				acc?;
				ReturnCode::try_from(*byte).map(|_| ())
			})
			.map_err(|_| DecodeError::InvalidSubackReturnCode)?;

		let return_codes = unsafe {
    		mem::transmute::<&[u8], &[ReturnCode]>(bytes)
		};

		Ok(Status::Complete((bytes.len(), Self {
			return_codes,
		})))
	}
}

impl<'a> Encodable for Suback<'a> {
	fn encoded_len(&self) -> usize {
		self.return_codes.len()
	}

	fn encode(&self, bytes: &mut [u8]) -> Result<usize, EncodeError> {
		if bytes.len() < self.return_codes.len() {
			return Err(EncodeError::OutOfSpace)
		}

		let return_code_bytes = unsafe {
			mem::transmute::<&[ReturnCode], &[u8]>(self.return_codes)
		};

		(&mut bytes[..self.return_codes.len()]).copy_from_slice(return_code_bytes);

		Ok(self.return_codes.len())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn encode() {
		let return_codes = [
			ReturnCode::SUCCESS_QOS_0,
		];

		let payload = Suback::new(&return_codes[..]);
		let mut buf = [0u8; 1];
		let used = payload.encode(&mut buf[..]);
		assert_eq!(used, Ok(1));
		assert_eq!(buf, [0b0000_0000]);
	}

	#[test]
	fn decode() {
		let return_code_bytes = [
			0b1000_0000,
			0b0000_0010,
			0b0000_0001,
			0b0000_0000,
		];

		let return_codes = [
			ReturnCode::FAILURE,
			ReturnCode::SUCCESS_QOS_2,
			ReturnCode::SUCCESS_QOS_1,
			ReturnCode::SUCCESS_QOS_0,
		];

		let payload = Suback::decode(&return_code_bytes[..]);
		assert_eq!(payload, Ok(Status::Complete((4, Suback::new(&return_codes[..])))));
	}
}