use core::{
    convert::{From, TryFrom, TryInto},
    fmt::Debug,
    result::Result,
};

use crate::{
    codec::{self, Encodable},
    error::{DecodeError, EncodeError},
    fixed_header::PacketFlags,
    qos,
    status::Status,
};

use super::HeaderDecode;

use bitfield::BitRange;

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum Protocol {
    MQTT,
}

impl Protocol {
    fn name(self) -> &'static str {
        match self {
            Protocol::MQTT => "MQTT",
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum Level {
    Level3_1_1,
}

impl TryFrom<u8> for Level {
    type Error = ();
    fn try_from(val: u8) -> Result<Self, Self::Error> {
        if val == 4 {
            Ok(Level::Level3_1_1)
        } else {
            Err(())
        }
    }
}

impl From<Level> for u8 {
    fn from(val: Level) -> u8 {
        match val {
            Level::Level3_1_1 => 4,
        }
    }
}

#[derive(PartialEq, Clone, Copy, Default)]
pub struct Flags(u8);

bitfield_bitrange! {
    struct Flags(u8)
}

impl Flags {
    bitfield_fields! {
        bool;
        pub has_username,  set_has_username  : 7;
        pub has_password,  set_has_password  : 6;
        pub will_retain,   set_will_retain   : 5;

        pub has_will,      set_has_will_flag : 2;
        pub clean_session, set_clean_session : 1;
    }

    pub fn will_qos(&self) -> Result<qos::QoS, qos::Error> {
        let qos_bits: u8 = self.bit_range(4, 3);
        qos_bits.try_into()
    }

    #[allow(dead_code)]
    pub fn set_will_qos(&mut self, qos: qos::QoS) {
        self.set_bit_range(4, 3, u8::from(qos))
    }
}

impl From<Flags> for u8 {
    fn from(val: Flags) -> u8 {
        val.0
    }
}

impl Debug for Flags {
    bitfield_debug! {
        struct Flags;
        pub has_username, _       : 7;
        pub has_password, _       : 6;
        pub will_retain, _        : 5;
        pub into QoS, will_qos, _ : 4, 3;
        pub has_will, _           : 2;
        pub clean_session, _      : 1;
    }
}

// VariableHeader for Connect packet
#[derive(PartialEq, Debug)]
pub struct Connect<'buf> {
    name: &'buf str,
    level: Level,
    flags: Flags,
    keep_alive: u16,
}

impl<'buf> Connect<'buf> {
    pub fn new(protocol: Protocol, level: Level, flags: Flags, keep_alive: u16) -> Self {
        let name = protocol.name();
        Connect {
            name: name,
            level,
            flags,
            keep_alive,
        }
    }

    pub fn name(&self) -> &str {
        self.name
    }

    pub fn level(&self) -> Level {
        self.level
    }

    pub fn flags(&self) -> Flags {
        self.flags
    }

    pub fn keep_alive(&self) -> u16 {
        self.keep_alive
    }
}

impl<'buf> HeaderDecode<'buf> for Connect<'buf> {
    fn decode(
        _flags: PacketFlags,
        bytes: &'buf [u8],
    ) -> Result<Status<(usize, Connect<'buf>)>, DecodeError> {
        let offset = 0;

        // read protocol name
        let (offset, name) = read!(codec::string::parse_string, bytes, offset);

        // read protocol revision
        let (offset, level) = read!(codec::values::parse_u8, bytes, offset);

        let level = level
            .try_into()
            .map_err(|_| DecodeError::InvalidProtocolLevel)?;
        if level != Level::Level3_1_1 {
            return Err(DecodeError::InvalidProtocolLevel);
        }

        // read protocol flags
        let (offset, flags) = read!(codec::values::parse_u8, bytes, offset);

        let flags = Flags(flags);

        if let Err(e) = flags.will_qos() {
            match e {
                qos::Error::BadPattern => return Err(DecodeError::InvalidConnectFlag),
            }
        }

        // read protocol keep alive
        let (offset, keep_alive) = read!(codec::values::parse_u16, bytes, offset);

        Ok(Status::Complete((
            offset,
            Connect {
                name,
                level,
                flags,
                keep_alive,
            },
        )))
    }
}

impl<'buf> Encodable for Connect<'buf> {
    fn encoded_len(&self) -> usize {
        self.name.encoded_len() + 1 + 1 + 2
    }

    fn encode(&self, bytes: &mut [u8]) -> Result<usize, EncodeError> {
        let mut offset = 0;
        offset += codec::string::encode_string(self.name, &mut bytes[offset..])?;
        offset += codec::values::encode_u8(self.level.into(), &mut bytes[offset..])?;
        offset += codec::values::encode_u8(self.flags.into(), &mut bytes[offset..])?;
        offset += codec::values::encode_u16(self.keep_alive, &mut bytes[offset..])?;
        Ok(offset)
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
        assert_eq!(flags.has_will(), true);
        assert_eq!(flags.clean_session(), true);

        let flags = Flags(0b00000000);
        assert_eq!(flags.has_username(), false);
        assert_eq!(flags.has_password(), false);
        assert_eq!(flags.will_retain(), false);
        assert_eq!(flags.has_will(), false);
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
            0b00000100, 0b01001101, // 'M'
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

        let connect = Connect::decode(PacketFlags::CONNECT, &buf);

        assert_eq!(
            connect,
            Ok(Status::Complete((
                10,
                Connect {
                    name: "MQTT",
                    level: Level::Level3_1_1,
                    flags: Flags(0b11001110),
                    keep_alive: 10,
                }
            )))
        );
    }
}
