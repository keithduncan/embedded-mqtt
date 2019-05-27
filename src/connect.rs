use crate::{
    string,
    status::Status,
    result::Result,
    error,
    qos,
    decoder,
};

use core::{
    fmt::Debug,
    convert::{
        TryInto,
    },
};

use bitfield::BitRange;

pub const PROTOCOL_LEVEL_MQTT_3_1_1: u8 = 4;

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
        pub has_username, _  : 7;
        pub has_password, _  : 6;
        pub will_retain, _   : 5;
        
        pub will_flag, _     : 2;
        pub clean_session, _ : 1;
    }

    fn will_qos(&self) -> core::result::Result<qos::QoS, qos::Error> {
        let qos_bits: u8 = self.bit_range(4, 3);
        qos_bits.try_into()
    }
}

impl Debug for Flags {
    bitfield_debug! {
        struct Flags;
        pub has_username, _       : 7;
        pub has_password, _       : 6;
        pub will_retain, _        : 5;
        pub into QoS, will_qos, _ : 4, 3;
        pub will_flag, _          : 2;
        pub clean_session, _      : 1;
    }
}

impl<'buf> Connect<'buf> {
    pub fn from_bytes(bytes: &[u8]) -> Result<Status<(usize, Connect)>> {
        let offset = 0;

        // read protocol name
        let (offset, name) = read!(string::parse_string, bytes, offset);

        // read protocol revision
        let (offset, level) = read!(decoder::parse_u8, bytes, offset);

        if level != PROTOCOL_LEVEL_MQTT_3_1_1 {
            return Err(error::Error::InvalidProtocolLevel)
        }

        // read protocol flags
        let (offset, flags) = read!(decoder::parse_u8, bytes, offset);

        let flags = Flags(flags);

        if let Err(e) = flags.will_qos() {
            match e {
                qos::Error::BadPattern => return Err(error::Error::InvalidConnectFlag),
            }
        }

        // read protocol keep alive
        let (offset, keep_alive) = read!(decoder::parse_u16, bytes, offset);

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
    fn parse_flags() {
        let flags = Flags(0b11100110);
        assert_eq!(flags.has_username(), true);
        assert_eq!(flags.has_password(), true);
        assert_eq!(flags.will_retain(), true);
        assert_eq!(flags.will_flag(), true);
        assert_eq!(flags.clean_session(), true);

        let flags = Flags(0b00000000);
        assert_eq!(flags.has_username(), false);
        assert_eq!(flags.has_password(), false);
        assert_eq!(flags.will_retain(), false);
        assert_eq!(flags.will_flag(), false);
        assert_eq!(flags.clean_session(), false);
    }

    #[test]
    fn parse_qos() {
        let flags = Flags(0b00010000);
        assert_eq!(flags.will_qos(), Ok(qos::QoS::ExactlyOnce));

        let flags = Flags(0b00001000);
        assert_eq!(flags.will_qos(), Ok(qos::QoS::AtLeastOnce));

        let flags = Flags(0b00000000);
        assert_eq!(flags.will_qos(), Ok(qos::QoS::AtMostOnce));
    }

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
            flags: Flags(0b11001110),
            keep_alive: 10,
        }))));
    }
}
