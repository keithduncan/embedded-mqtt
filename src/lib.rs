// #![deny(warnings)]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate byteorder;
#[cfg(feature = "std")]
extern crate std as core;

use core::io::Read;

use byteorder::{BigEndian, ReadBytesExt};

pub mod error;
pub use error::{Error, Result};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum PacketType {
    Connect,
    Connack,
    Publish,
    Puback,
    Pubrec,
    Pubrel,
    Pubcomp,
    Subscribe,
    Suback,
    Unsubscribe,
    Unsuback,
    Pingreq,
    Pingresp,
    Disconnect,
}

pub type PacketTypeFlags = u8;

pub struct FixedHeader<'buf> {
    type_: PacketType,
    // TODO: Temporary field to avoid 'unused lifetime param' errors
    _ver: &'buf str,
}

impl<'buf> FixedHeader<'buf> {
    pub fn type_(&self) -> &PacketType {
        &self.type_
    }
}

fn parse_packet_type<T: Read>(bytes: &mut T) -> Result<(PacketType, PacketTypeFlags)> {
    let inp = bytes.read_u8()?;

    // high 4 bits are the packet type
    let packet_type = match (inp & 0xF0) >> 4 {
        1 => Ok(PacketType::Connect),
        2 => Ok(PacketType::Connack),
        3 => Ok(PacketType::Publish),
        4 => Ok(PacketType::Puback),
        5 => Ok(PacketType::Pubrec),
        6 => Ok(PacketType::Pubrel),
        7 => Ok(PacketType::Pubcomp),
        8 => Ok(PacketType::Subscribe),
        9 => Ok(PacketType::Suback),
        10 => Ok(PacketType::Unsubscribe),
        11 => Ok(PacketType::Unsuback),
        12 => Ok(PacketType::Pingreq),
        13 => Ok(PacketType::Pingresp),
        14 => Ok(PacketType::Disconnect),
        _ => Err(Error::PacketType),
    }?;

    // low 4 bits represent control flags
    let flags = inp & 0xF;

    validate_flag(packet_type, flags)
}

fn validate_flag(
    packet_type: PacketType,
    flags: PacketTypeFlags,
) -> Result<(PacketType, PacketTypeFlags)> {
    // for the following packet types, the control flag MUST be zero
    const ZERO_TYPES: &[PacketType] = &[
        PacketType::Connect,
        PacketType::Connack,
        PacketType::Puback,
        PacketType::Pubrec,
        PacketType::Pubcomp,
        PacketType::Suback,
        PacketType::Unsuback,
        PacketType::Pingreq,
        PacketType::Pingresp,
        PacketType::Disconnect,
    ];
    // for the following packet types, the control flag MUST be 0b0010
    const ONE_TYPES: &[PacketType] = &[
        PacketType::Pubrel,
        PacketType::Subscribe,
        PacketType::Unsubscribe,
    ];

    validate_flag_val(packet_type, flags, ZERO_TYPES, 0b0000)
        .and_then(|_| validate_flag_val(packet_type, flags, ONE_TYPES, 0b0010))
}

fn validate_flag_val(
    packet_type: PacketType,
    flags: PacketTypeFlags,
    types: &[PacketType],
    expected_flags: PacketTypeFlags,
) -> Result<(PacketType, PacketTypeFlags)> {
    if let Some(_) = types.iter().find(|&&v| v == packet_type) {
        if flags != expected_flags {
            return Err(Error::PacketFlag);
        }
    }

    Ok((packet_type, flags))
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::io::Cursor;

    #[test]
    fn packet_type() {
        let mut inputs: [([u8; 1], PacketType); 14] = [
            ([01 << 4 | 0b0000], PacketType::Connect),
            ([02 << 4 | 0b0000], PacketType::Connack),
            ([03 << 4 | 0b0000], PacketType::Publish),
            ([04 << 4 | 0b0000], PacketType::Puback),
            ([05 << 4 | 0b0000], PacketType::Pubrec),
            ([06 << 4 | 0b0010], PacketType::Pubrel),
            ([07 << 4 | 0b0000], PacketType::Pubcomp),
            ([08 << 4 | 0b0010], PacketType::Subscribe),
            ([09 << 4 | 0b0000], PacketType::Suback),
            ([10 << 4 | 0b0010], PacketType::Unsubscribe),
            ([11 << 4 | 0b0000], PacketType::Unsuback),
            ([12 << 4 | 0b0000], PacketType::Pingreq),
            ([13 << 4 | 0b0000], PacketType::Pingresp),
            ([14 << 4 | 0b0000], PacketType::Disconnect),
        ];

        for (buf, expected_type) in inputs.iter_mut() {
            let expected_flag = buf[0] & 0xF;
            let mut buf = Cursor::new(buf);
            let (packet_type, flag) = parse_packet_type(&mut buf).unwrap();
            assert_eq!(packet_type, *expected_type);
            assert_eq!(flag, expected_flag);
        }
    }

    #[test]
    fn bad_packet_type() {
        let mut buf = Cursor::new(&[15 << 4]);
        let result = parse_packet_type(&mut buf);
        assert_eq!(result, Err(Error::PacketType));
    }

    #[test]
    fn bad_zero_flags() {
        let mut inputs: [([u8; 1], PacketType); 10] = [
            ([01 << 4 | 1], PacketType::Connect),
            ([02 << 4 | 1], PacketType::Connack),
            ([04 << 4 | 1], PacketType::Puback),
            ([05 << 4 | 1], PacketType::Pubrec),
            ([07 << 4 | 1], PacketType::Pubcomp),
            ([09 << 4 | 1], PacketType::Suback),
            ([11 << 4 | 1], PacketType::Unsuback),
            ([12 << 4 | 1], PacketType::Pingreq),
            ([13 << 4 | 1], PacketType::Pingresp),
            ([14 << 4 | 1], PacketType::Disconnect),
        ];
        for (buf, _) in inputs.iter_mut() {
            let mut buf = Cursor::new(buf);
            let result = parse_packet_type(&mut buf);
            assert_eq!(result, Err(Error::PacketFlag));
        }
    }

    #[test]
    fn bad_one_flags() {
        let mut inputs: [([u8; 1], PacketType); 3] = [
            ([06 << 4 | 0], PacketType::Pubrel),
            ([08 << 4 | 0], PacketType::Subscribe),
            ([10 << 4 | 0], PacketType::Unsubscribe),
        ];
        for (buf, _) in inputs.iter_mut() {
            let mut buf = Cursor::new(buf);
            let result = parse_packet_type(&mut buf);
            assert_eq!(result, Err(Error::PacketFlag));
        }
    }

    #[test]
    fn publish_flags() {
        for i in 0..15 {
            let mut input = [03 << 4 | i];
            let mut buf = Cursor::new(input);
            let (packet_type, flag) = parse_packet_type(&mut buf).unwrap();
            assert_eq!(packet_type, PacketType::Publish);
            assert_eq!(flag, i);
        }
    }
}
