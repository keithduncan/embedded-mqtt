use crate::{
    decode,
    status::Status,
    result::Result,
};

use core::fmt::Debug;

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

// VariableHeader for Connack packet
#[derive(PartialEq, Debug)]
pub struct Connack {
    flags: Flags,
    return_code: u8,
}

impl Connack {
    pub fn from_bytes(bytes: &[u8]) -> Result<Status<(usize, Self)>> {
    	if bytes.len() < 2 {
    		return Ok(Status::Partial(2 - bytes.len()));
    	}

        let offset = 0;

        // read connack flags
        let (offset, flags) = read!(decode::values::parse_u8, bytes, offset);
        let flags = Flags(flags);

        // read return code
        let (offset, return_code) = read!(decode::values::parse_u8, bytes, offset);

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

    pub fn return_code(&self) -> u8 {
        self.return_code
    }
}

#[cfg(test)]
mod tests {
    
}
