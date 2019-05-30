use core::result::Result;

use crate::{
    codec::{self, Decodable, Encodable},
    error::{DecodeError, EncodeError},
    status::Status,
};

mod packet_type;
mod packet_flags;

pub use self::{
    packet_type::PacketType,
    packet_flags::{
        PacketFlags,
        PublishFlags,
    },
};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct FixedHeader {
    r#type: PacketType,
    flags: PacketFlags,
    len: u32,
}

impl FixedHeader {
    pub fn new(r#type: PacketType, flags: PacketFlags, len: u32) -> Self {
        FixedHeader {
            r#type,
            flags,
            len
        }
    }

    pub fn r#type(&self) -> PacketType {
        self.r#type
    }

    pub fn flags(&self) -> PacketFlags {
        self.flags
    }

    pub fn len(&self) -> u32 {
        self.len
    }
}

impl<'buf> Decodable<'buf> for FixedHeader {
    fn decode(bytes: &'buf [u8]) -> Result<Status<(usize, Self)>, DecodeError> {
        // "bytes" must be at least 2 bytes long to be a valid fixed header
        if bytes.len() < 2 {
            return Ok(Status::Partial(2 - bytes.len()));
        }

        let (r#type, flags) = parse_packet_type(bytes[0])?;

        let offset = 1;

        let (offset, len) = read!(parse_remaining_length, bytes, offset);

        Ok(Status::Complete((offset, Self {
            r#type,
            flags,
            len
        })))
    }
}

impl Encodable for FixedHeader {
    fn encoded_len(&self) -> usize {
        let mut buf = [0u8; 4];
        let u = encode_remaining_length(self.len, &mut buf);
        1 + u
    }

    fn encode(&self, bytes: &mut [u8]) -> Result<usize, EncodeError> {
        let offset = 0;
        let offset = {
            let o = codec::values::encode_u8(encode_packet_type(self.r#type, self.flags), &mut bytes[offset..])?;
            offset + o
        };
        let offset = {
            let mut remaining_length = [0u8; 4];
            let o = encode_remaining_length(self.len, &mut remaining_length);
            (&mut bytes[offset..offset+o]).copy_from_slice(&remaining_length[..o]);
            offset + o
        };
        Ok(offset)
    }
}

fn parse_remaining_length(bytes: &[u8]) -> Result<Status<(usize, u32)>, DecodeError> {
    let mut multiplier = 1;
    let mut value = 0u32;
    let mut index = 0;

    loop {
        if multiplier > 128 * 128 * 128 {
            return Err(DecodeError::RemainingLength);
        }

        if index >= bytes.len() {
            return Ok(Status::Partial(1));
        }

        let byte = bytes[index];
        index += 1;

        value += (byte & 0b01111111) as u32 * multiplier;

        multiplier *= 128;

        if byte & 128 == 0 {
            return Ok(Status::Complete((index, value)));
        }
    }
}

fn encode_remaining_length(mut len: u32, buf: &mut [u8; 4]) -> usize {
    let mut index = 0;
    loop {
        let mut byte = len as u8 % 128;
        len /= 128;
        if len > 0 {
            byte |= 128;
        }
        buf[index] = byte;
        index = index + 1;

        if len == 0 {
            break index;
        }
    }
}

fn parse_packet_type(inp: u8) -> Result<(PacketType, PacketFlags), DecodeError> {
    // high 4 bits are the packet type
    let packet_type = match (inp & 0xF0) >> 4 {
        1 => PacketType::Connect,
        2 => PacketType::Connack,
        3 => PacketType::Publish,
        4 => PacketType::Puback,
        5 => PacketType::Pubrec,
        6 => PacketType::Pubrel,
        7 => PacketType::Pubcomp,
        8 => PacketType::Subscribe,
        9 => PacketType::Suback,
        10 => PacketType::Unsubscribe,
        11 => PacketType::Unsuback,
        12 => PacketType::Pingreq,
        13 => PacketType::Pingresp,
        14 => PacketType::Disconnect,
        _ => return Err(DecodeError::PacketType),
    };

    // low 4 bits represent control flags
    let flags = PacketFlags(inp & 0xF);

    validate_flag(packet_type, flags)
}

fn encode_packet_type(r#type: PacketType, flags: PacketFlags) -> u8 {
    let packet_type: u8 = match r#type {
        PacketType::Connect => 1,
        PacketType::Connack => 2,
        PacketType::Publish => 3,
        PacketType::Puback => 4,
        PacketType::Pubrec => 5,
        PacketType::Pubrel => 6,
        PacketType::Pubcomp => 7,
        PacketType::Subscribe => 8,
        PacketType::Suback => 9,
        PacketType::Unsubscribe => 10,
        PacketType::Unsuback => 11,
        PacketType::Pingreq => 12,
        PacketType::Pingresp => 13,
        PacketType::Disconnect => 14,
    };

    (packet_type << 4) | flags.0
}

fn validate_flag(packet_type: PacketType, flags: PacketFlags) -> Result<(PacketType, PacketFlags), DecodeError> {
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

    validate_flag_val(packet_type, flags, ZERO_TYPES, PacketFlags(0b0000))
        .and_then(|_| validate_flag_val(packet_type, flags, ONE_TYPES, PacketFlags(0b0010)))
}

fn validate_flag_val(
    packet_type: PacketType,
    flags: PacketFlags,
    types: &[PacketType],
    expected_flags: PacketFlags,
) -> Result<(PacketType, PacketFlags), DecodeError> {
    if let Some(_) = types.iter().find(|&&v| v == packet_type) {
        if flags != expected_flags {
            return Err(DecodeError::PacketFlag);
        }
    }

    Ok((packet_type, flags))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rayon::prelude::*;
    use std::format;

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
            let expected_flag = PacketFlags(buf[0] & 0xF);
            let (packet_type, flag) = parse_packet_type(buf[0]).unwrap();
            assert_eq!(packet_type, *expected_type);
            assert_eq!(flag, expected_flag);
        }
    }

    #[test]
    fn bad_packet_type() {
        let result = parse_packet_type(15 << 4);
        assert_eq!(result, Err(DecodeError::PacketType));
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
            let result = parse_packet_type(buf[0]);
            assert_eq!(result, Err(DecodeError::PacketFlag));
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
            let result = parse_packet_type(buf[0]);
            assert_eq!(result, Err(DecodeError::PacketFlag));
        }
    }

    #[test]
    fn publish_flags() {
        for i in 0..15 {
            let mut input = 03 << 4 | i;
            let (packet_type, flag) = parse_packet_type(input).unwrap();
            assert_eq!(packet_type, PacketType::Publish);
            assert_eq!(flag, PacketFlags(i));
        }
    }

    #[test]
    #[ignore]
    fn remaining_length() {
        // NOTE: This test can take a while to complete.
        let _: u32 = (0u32..(268435455 + 1))
            .into_par_iter()
            .map(|i| {
                let mut buf = [0u8; 4];
                let expected_offset = encode_remaining_length(i, &mut buf);
                let (offset, len) =
                    parse_remaining_length(&buf).expect(&format!("Failed for number: {}", i)).unwrap();
                assert_eq!(i, len);
                assert_eq!(expected_offset, offset);
                0
            })
            .sum();
    }

    #[test]
    fn bad_remaining_length() {
        let buf = [0xFF, 0xFF, 0xFF, 0xFF];
        let result = parse_remaining_length(&buf);
        assert_eq!(result, Err(DecodeError::RemainingLength));
    }

    #[test]
    fn bad_remaining_length2() {
        let buf = [0xFF, 0xFF];
        let result = parse_remaining_length(&buf);
        assert_eq!(result, Ok(Status::Partial(1)));
    }

    #[test]
    fn fixed_header1() {
        let buf = [
            01 << 4 | 0b0000, // PacketType::Connect
            0,                // remaining length
        ];
        let (offset, header) = FixedHeader::decode(&buf).unwrap().unwrap();
        assert_eq!(offset, 2);
        assert_eq!(header.r#type(), PacketType::Connect);
        assert_eq!(header.flags(), PacketFlags(0));
        assert_eq!(header.len(), 0);
    }

    #[test]
    fn fixed_header2() {
        let buf = [
            03 << 4 | 0b0000, // PacketType::Publish
            0x80,             // remaining length
            0x80,
            0x80,
            0x1,
        ];
        let (offset, header) = FixedHeader::decode(&buf).unwrap().unwrap();
        assert_eq!(offset, 5);
        assert_eq!(header.r#type(), PacketType::Publish);
        assert_eq!(header.flags(), PacketFlags(0));
        assert_eq!(header.len(), 2097152);
    }

    #[test]
    fn bad_len() {
        let buf = [03 << 4 | 0];
        let result = FixedHeader::decode(&buf);
        assert_eq!(result, Ok(Status::Partial(1)));
    }
}
