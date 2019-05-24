use crate::{
    error::Error,
    packet::{
        PacketType,
        PacketFlags,
    },
    result::Result,
    status::Status,
};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Header {
    r#type: PacketType,
    flags: PacketFlags,
    len: u32,
}

impl Header {
    pub fn from_bytes(bytes: &[u8]) -> Result<Status<Header>> {
        // "bytes" must be at least 2 bytes long to be a valid fixed header
        if bytes.len() < 2 {
            return Ok(Status::Partial);
        }

        let (r#type, flags) = parse_packet_type(bytes[0])?;
        let (len, _) = parse_remaining_length(&bytes[1..])?;

        Ok(Status::Complete(Header { r#type, flags, len }))
    }

    pub fn r#type(&self) -> &PacketType {
        &self.r#type
    }

    pub fn flags(&self) -> &PacketFlags {
        &self.flags
    }

    pub fn len(&self) -> &u32 {
        &self.len
    }
}

fn parse_remaining_length(bytes: &[u8]) -> Result<(u32, usize)> {
    let mut multiplier = 1;
    let mut value = 0u32;
    let mut index = 0;

    loop {
        if multiplier > 128 * 128 * 128 {
            return Err(Error::RemainingLength);
        } else if index >= bytes.len() {
            return Err(Error::InvalidLength);
        }

        let byte = bytes[index];
        index += 1;
        value += (byte & 127) as u32 * multiplier;
        multiplier *= 128;
        if byte & 128 == 0 {
            return Ok((value, index));
        }
    }
}

fn parse_packet_type(inp: u8) -> Result<(PacketType, PacketFlags)> {
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
    flags: PacketFlags,
) -> Result<(PacketType, PacketFlags)> {
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
    flags: PacketFlags,
    types: &[PacketType],
    expected_flags: PacketFlags,
) -> Result<(PacketType, PacketFlags)> {
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
            let expected_flag = buf[0] & 0xF;
            let (packet_type, flag) = parse_packet_type(buf[0]).unwrap();
            assert_eq!(packet_type, *expected_type);
            assert_eq!(flag, expected_flag);
        }
    }

    #[test]
    fn bad_packet_type() {
        let result = parse_packet_type(15 << 4);
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
            let result = parse_packet_type(buf[0]);
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
            let result = parse_packet_type(buf[0]);
            assert_eq!(result, Err(Error::PacketFlag));
        }
    }

    #[test]
    fn publish_flags() {
        for i in 0..15 {
            let mut input = 03 << 4 | i;
            let (packet_type, flag) = parse_packet_type(input).unwrap();
            assert_eq!(packet_type, PacketType::Publish);
            assert_eq!(flag, i);
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

    #[test]
    fn remaining_length() {
        // NOTE: This test can take a while to complete.
        let _: u32 = (0u32..(268435455 + 1))
            .into_par_iter()
            .map(|i| {
                let mut buf = [0u8; 4];
                let expected_index = encode_remaining_length(i, &mut buf);
                let (len, index) =
                    parse_remaining_length(&buf).expect(&format!("Failed for number: {}", i));
                assert_eq!(i, len);
                assert_eq!(expected_index, index);
                0
            })
            .sum();
    }

    #[test]
    fn bad_remaining_length() {
        let buf = [0xFF, 0xFF, 0xFF, 0xFF];
        let result = parse_remaining_length(&buf);
        assert_eq!(result, Err(Error::RemainingLength));
    }

    #[test]
    fn bad_remaining_length2() {
        let buf = [0xFF, 0xFF];
        let result = parse_remaining_length(&buf);
        assert_eq!(result, Err(Error::InvalidLength));
    }

    #[test]
    fn fixed_header1() {
        let buf = [
            01 << 4 | 0b0000, // PacketType::Connect
            0,                // remaining length
        ];
        let header = Header::from_bytes(&buf).unwrap().unwrap();
        assert_eq!(*header.r#type(), PacketType::Connect);
        assert_eq!(*header.flags(), 0);
        assert_eq!(*header.len(), 0);
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
        let header = Header::from_bytes(&buf).unwrap().unwrap();
        assert_eq!(*header.r#type(), PacketType::Publish);
        assert_eq!(*header.flags(), 0);
        assert_eq!(*header.len(), 2097152);
    }

    #[test]
    fn bad_len() {
        let result = Header::from_bytes(&[03 << 4 | 0]).unwrap();
        assert_eq!(result, Status::Partial);
    }
}
