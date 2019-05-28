use core::{
    fmt::Debug,
    convert::{TryFrom, TryInto},
    result::Result,
};

use crate::{
    codec,
    status::Status,
    error::ParseError,
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
    pub fn from_bytes(bytes: &[u8]) -> Result<Status<(usize, Self)>, ParseError> {
    	if bytes.len() < 2 {
    		return Ok(Status::Partial(2 - bytes.len()));
    	}

        let offset = 0;

        // read connack flags
        let (offset, flags) = read!(codec::values::parse_u8, bytes, offset);
        let flags = flags.try_into().map_err(|_| ParseError::InvalidConnackFlag)?;

        // read return code
        let (offset, return_code) = read!(codec::values::parse_u8, bytes, offset);
        let return_code = return_code.try_into().map_err(|_| ParseError::InvalidConnackReturnCode)?;

        #[cfg(feature = "std")]
        println!("connack::from_bytes {:?}", offset);

        Ok(Status::Complete((offset, Connack {
            flags,
            return_code,
        })))
    }

    pub fn flags(&self) -> Flags {
        self.flags
    }

    pub fn return_code(&self) -> ReturnCode {
        self.return_code
    }
}

#[cfg(test)]
mod tests {
    
}
