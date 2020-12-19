use core::{default::Default, result::Result};

use crate::{
    codec::{Decodable, Encodable},
    error::{DecodeError, EncodeError},
    fixed_header::PacketType,
    status::Status,
};

pub mod connect;
pub mod suback;
pub mod subscribe;

#[derive(Debug)]
pub enum Payload<'a> {
    Bytes(&'a [u8]),
    Connect(connect::Connect<'a>),
    Subscribe(subscribe::Subscribe<'a>),
    Suback(suback::Suback<'a>),
}

impl<'a> Payload<'a> {
    pub fn decode(
        r#type: PacketType,
        bytes: &'a [u8],
    ) -> Option<Result<Status<(usize, Self)>, DecodeError>> {
        Some(match r#type {
            // TODO need to pass the variable header / flags to the payload parser
            //PacketType::Connect => Payload::Connect(complete!(connect::Connect::decode(bytes))),
            PacketType::Suback => match suback::Suback::decode(bytes) {
                Err(e) => Err(e),
                Ok(Status::Partial(p)) => Ok(Status::Partial(p)),
                Ok(Status::Complete((offset, p))) => {
                    Ok(Status::Complete((offset, Payload::Suback(p))))
                }
            },
            PacketType::Subscribe => match subscribe::Subscribe::decode(bytes) {
                Err(e) => Err(e),
                Ok(Status::Partial(p)) => Ok(Status::Partial(p)),
                Ok(Status::Complete((offset, p))) => {
                    Ok(Status::Complete((offset, Payload::Subscribe(p))))
                }
            },
            _ => return None,
        })
    }
}

impl<'a> Encodable for Payload<'a> {
    fn encoded_len(&self) -> usize {
        match self {
            &Payload::Connect(ref c) => c.encoded_len(),
            &Payload::Subscribe(ref c) => c.encoded_len(),
            &Payload::Suback(ref c) => c.encoded_len(),
            &Payload::Bytes(ref c) => c.len(),
        }
    }

    fn encode(&self, bytes: &mut [u8]) -> Result<usize, EncodeError> {
        match self {
            &Payload::Connect(ref c) => c.encode(bytes),
            &Payload::Subscribe(ref c) => c.encode(bytes),
            &Payload::Suback(ref c) => c.encode(bytes),
            &Payload::Bytes(ref c) => {
                if bytes.len() < c.len() {
                    return Err(EncodeError::OutOfSpace);
                }

                (&mut bytes[0..c.len()]).copy_from_slice(c);

                Ok(c.len())
            }
        }
    }
}

impl<'a> Default for Payload<'a> {
    fn default() -> Self {
        Payload::Bytes(&[])
    }
}
