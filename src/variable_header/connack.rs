use core::{
    fmt::Debug,
    convert::{TryFrom, TryInto},
    result::Result,
};

use crate::{
    codec::{self, Decodable},
    status::Status,
    error::DecodeError,
};

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

impl<'buf> Decodable<'buf> for Connack {
    fn decode(bytes: &[u8]) -> Result<Status<(usize, Self)>, DecodeError> {
        if bytes.len() < 2 {
            return Ok(Status::Partial(2 - bytes.len()));
        }

        let offset = 0;

        // read connack flags
        let (offset, flags) = read!(codec::values::parse_u8, bytes, offset);
        let flags = flags.try_into().map_err(|_| DecodeError::InvalidConnackFlag)?;

        // read return code
        let (offset, return_code) = read!(codec::values::parse_u8, bytes, offset);
        let return_code = return_code.try_into().map_err(|_| DecodeError::InvalidConnackReturnCode)?;

        Ok(Status::Complete((offset, Connack {
            flags,
            return_code,
        })))
    }
}

#[cfg(test)]
mod tests {
    
}
