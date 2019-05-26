use crate::{
    string,
    status::Status,
    result::Result,
    error,
    qos,
};

use core::{
    fmt::Debug,
    convert::{
        TryInto,
    },
};

use byteorder::{
    BigEndian,
    ByteOrder,
};

use bitfield::BitRange;

// VariableHeader for Connect packet
#[derive(PartialEq, Debug)]
pub struct Connect<'buf> {
    name: &'buf str,
    level: u8,
    flags: Flags,
    keep_alive: u16,
}

#[derive(PartialEq)]
pub struct Flags(u8);

bitfield_bitrange! {
    struct Flags(u8)
}

impl Flags {
    bitfield_fields! {
        bool;
        pub has_username, _       : 8;
        pub has_password, _       : 7;
        pub will_retain, _        : 6;
        
        pub will_flag, _          : 3;
        pub clean_session, _      : 1;
    }

    fn will_qos(&self) -> core::result::Result<qos::QoS, qos::Error> {
        let qos_bits: u8 = self.bit_range(5, 4);
        qos_bits.try_into()
    }
}

impl Debug for Flags {
    bitfield_debug! {
        struct Flags;
        pub has_username, _       : 8;
        pub has_password, _       : 7;
        pub will_retain, _        : 6;
        pub into QoS, will_qos, _ : 5, 4;
        pub will_flag, _          : 3;
        pub clean_session, _      : 1;
    }
}

fn parse_byte(bytes: &[u8]) -> Result<Status<(usize, u8)>> {
    if bytes.len() < 1 {
        return Ok(Status::Partial)
    }

    Ok(Status::Complete((1, bytes[0])))
}

fn parse_length(bytes: &[u8]) -> Result<Status<(usize, u16)>> {
    if bytes.len() < 2 {
        return Ok(Status::Partial)
    }

    Ok(Status::Complete((2, BigEndian::read_u16(&bytes[0..2]))))
}

macro_rules! read {
    ($fn:path, $bytes:expr, $offset:expr) => {
        match try!($fn(&$bytes[$offset..])) {
            Status::Complete(v) => ($offset + v.0, v.1),
            Status::Partial => return Ok(Status::Partial),
        }
    };
}

impl<'buf> Connect<'buf> {
    pub fn from_bytes(bytes: &[u8]) -> Result<Status<(usize, Connect)>> {
        let offset = 0;

        // read protocol name
        let (offset, name) = read!(string::parse_string, bytes, offset);

        // read protocol revision
        let (offset, level) = read!(parse_byte, bytes, offset);

        // read protocol flags
        let (offset, flags) = read!(parse_byte, bytes, offset);

        let flags = Flags(flags);

        match flags.will_qos() {
            Err(qos::Error::BadPattern) => return Err(error::Error::InvalidConnectFlag),
            Ok(_) => (),
        }

        // read protocol keep alive
        let (offset, keep_alive) = read!(parse_length, bytes, offset);

        Ok(Status::Complete((offset, Connect {
            name,
            level,
            flags,
            keep_alive,
        })))
    }

    pub fn name(&self) -> &str {
        self.name
    }

    pub fn level(&self) -> &u8 {
        &self.level
    }

    pub fn flags(&self) -> &Flags {
        &self.flags
    }

    pub fn keep_alive(&self) -> &u16 {
        &self.keep_alive
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_connect() {
        let buf = [
            0b00000000, // Protocol Name Length
            0b00000100,
            0b01001101, // 'M'
            0b01010001, // 'Q'
            0b01010100, // 'T'
            0b01010100, // 'T'
            0b00000100, // Level 4
            0b11001110, // Connect Flags - Username 1
                        //               - Password 1
                        //               - Will Retain 0
                        //               - Will QoS 01
                        //               - Will Flag 1
                        //               - Clean Session 1
                        //               - Reserved 0
            0b00000000, // Keep Alive (10s)
            0b00001010, // 
        ];

        let connect = Connect::from_bytes(&buf);

        assert_eq!(connect, Ok(Status::Complete((10, Connect {
            name: "MQTT",
            level: 4,
            flags: 0b11001110,
            keep_alive: 10,
        }))));
    }
}
