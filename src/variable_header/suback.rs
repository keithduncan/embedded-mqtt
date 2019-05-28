use crate::{
    decode,
    status::Status,
    result::Result,
};

// VariableHeader for Connack packet
#[derive(PartialEq, Debug)]
pub struct Suback {
    packet_identifier: u16,
}

impl Suback {
    pub fn from_bytes(bytes: &[u8]) -> Result<Status<(usize, Self)>> {
    	if bytes.len() < 2 {
    		return Ok(Status::Partial(2 - bytes.len()));
    	}

        let offset = 0;

        // read connack flags
        let (offset, packet_identifier) = read!(decode::values::parse_u16, bytes, offset);

        Ok(Status::Complete((offset, Suback {
            packet_identifier,
        })))
    }

    pub fn packet_identifier(&self) -> u16 {
        self.packet_identifier
    }
}

#[cfg(test)]
mod tests {
    
}
