use core::{
    convert::{TryFrom, TryInto},
    fmt::Debug,
    result::Result,
};

use crate::{
    codec::{self, Encodable},
    error::{DecodeError, EncodeError},
    fixed_header::PacketFlags,
    status::Status,
};

use super::HeaderDecode;

#[derive(PartialEq, Clone, Copy)]
pub struct Flags(u8);

bitfield_bitrange! {
    struct Flags(u8)
}

impl Flags {
    bitfield_fields! {
        bool;
        pub session_present, _ : 1;
    }
}

impl Debug for Flags {
    bitfield_debug! {
        struct Flags;
        pub session_present, _ : 1;
    }
}

impl TryFrom<u8> for Flags {
    type Error = ();
    fn try_from(from: u8) -> Result<Flags, ()> {
        if 0b11111110 & from != 0 {
            Err(())
        } else {
            Ok(Flags(from))
        }
    }
}

impl Encodable for Flags {
    fn encoded_len(&self) -> usize {
        1
    }

    fn encode(&self, bytes: &mut [u8]) -> Result<usize, EncodeError> {
        if bytes.len() < 1 {
            return Err(EncodeError::OutOfSpace);
        }

        bytes[0] = self.0;

        Ok(1)
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum ReturnCode {
    Accepted,
    RefusedProtocolVersion,
    RefusedClientIdentifier,
    RefusedServerUnavailable,
    RefusedUsernameOrPassword,
    RefusedNotAuthorized,
}

impl TryFrom<u8> for ReturnCode {
    type Error = ();
    fn try_from(from: u8) -> Result<ReturnCode, ()> {
        Ok(match from {
            0 => ReturnCode::Accepted,
            1 => ReturnCode::RefusedProtocolVersion,
            2 => ReturnCode::RefusedClientIdentifier,
            3 => ReturnCode::RefusedServerUnavailable,
            4 => ReturnCode::RefusedUsernameOrPassword,
            5 => ReturnCode::RefusedNotAuthorized,
            _ => return Err(()),
        })
    }
}

impl Encodable for ReturnCode {
    fn encoded_len(&self) -> usize {
        1
    }

    fn encode(&self, bytes: &mut [u8]) -> Result<usize, EncodeError> {
        if bytes.len() < 1 {
            return Err(EncodeError::OutOfSpace);
        }

        let val = match self {
            &ReturnCode::Accepted => 0,
            &ReturnCode::RefusedProtocolVersion => 1,
            &ReturnCode::RefusedClientIdentifier => 2,
            &ReturnCode::RefusedServerUnavailable => 3,
            &ReturnCode::RefusedUsernameOrPassword => 4,
            &ReturnCode::RefusedNotAuthorized => 5,
        };

        bytes[0] = val;

        Ok(1)
    }
}

// VariableHeader for Connack packet
#[derive(PartialEq, Debug)]
pub struct Connack {
    flags: Flags,
    return_code: ReturnCode,
}

impl Connack {
    pub fn flags(&self) -> Flags {
        self.flags
    }

    pub fn return_code(&self) -> ReturnCode {
        self.return_code
    }
}

impl<'buf> HeaderDecode<'buf> for Connack {
    fn decode(_flags: PacketFlags, bytes: &[u8]) -> Result<Status<(usize, Self)>, DecodeError> {
        if bytes.len() < 2 {
            return Ok(Status::Partial(2 - bytes.len()));
        }

        let offset = 0;

        // read connack flags
        let (offset, flags) = read!(codec::values::parse_u8, bytes, offset);
        let flags = flags
            .try_into()
            .map_err(|_| DecodeError::InvalidConnackFlag)?;

        // read return code
        let (offset, return_code) = read!(codec::values::parse_u8, bytes, offset);
        let return_code = return_code
            .try_into()
            .map_err(|_| DecodeError::InvalidConnackReturnCode)?;

        Ok(Status::Complete((offset, Connack { flags, return_code })))
    }
}

impl Encodable for Connack {
    fn encoded_len(&self) -> usize {
        2
    }

    fn encode(&self, bytes: &mut [u8]) -> Result<usize, EncodeError> {
        self.flags.encode(&mut bytes[0..])?;
        self.return_code.encode(&mut bytes[1..])?;
        Ok(2)
    }
}

#[cfg(test)]
mod tests {}
