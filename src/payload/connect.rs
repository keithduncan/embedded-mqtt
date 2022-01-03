#![allow(warnings)]

use core::result::Result;

use crate::{
    codec::{self, Decodable, Encodable},
    error::{DecodeError, EncodeError},
    status::Status,
    variable_header::connect::Flags,
};

#[derive(Debug)]
pub struct Will<'buf> {
    topic: &'buf str,
    message: &'buf [u8],
}

impl<'buf> Decodable<'buf> for Will<'buf> {
    fn decode(bytes: &'buf [u8]) -> Result<Status<(usize, Will<'buf>)>, DecodeError> {
        let offset = 0;
        let (offset, topic) = read!(codec::string::parse_string, bytes, offset);
        let (offset, message) = read!(codec::values::parse_bytes, bytes, offset);

        Ok(Status::Complete((offset, Will { topic, message })))
    }
}

impl<'buf> Encodable for Will<'buf> {
    fn encoded_len(&self) -> usize {
        2 + self.topic.len() + 2 + self.message.len()
    }

    fn encode(&self, bytes: &mut [u8]) -> Result<usize, EncodeError> {
        let mut offset = 0;
        offset += codec::string::encode_string(self.topic, &mut bytes[offset..])?;
        offset += codec::values::encode_bytes(self.message, &mut bytes[offset..])?;
        Ok(offset)
    }
}

impl<'buf> Will<'buf> {
    pub fn new(topic: &'buf str, message: &'buf [u8]) -> Self {
        Will { topic, message }
    }
}

#[derive(Debug)]
pub struct Connect<'buf> {
    client_id: &'buf str,
    will: Option<Will<'buf>>,
    username: Option<&'buf str>,
    password: Option<&'buf [u8]>,
}

impl<'buf> Connect<'buf> {
    pub fn new(
        client_id: &'buf str,
        will: Option<Will<'buf>>,
        username: Option<&'buf str>,
        password: Option<&'buf [u8]>,
    ) -> Self {
        Connect {
            client_id,
            will,
            username,
            password,
        }
    }
}

impl<'buf> Connect<'buf> {
    pub fn decode(flags: Flags, bytes: &'buf [u8]) -> Result<Status<(usize, Self)>, DecodeError> {
        let offset = 0;

        let (offset, client_id) = read!(codec::string::parse_string, bytes, offset);

        let (offset, will) = if flags.has_will() {
            let (offset, will) = read!(Will::decode, bytes, offset);
            (offset, Some(will))
        } else {
            (offset, None)
        };

        let (offset, username) = if flags.has_username() {
            let (offset, username) = read!(codec::string::parse_string, bytes, offset);
            (offset, Some(username))
        } else {
            (offset, None)
        };

        let (offset, password) = if flags.has_password() {
            let (offset, password) = read!(codec::values::parse_bytes, bytes, offset);
            (offset, Some(bytes))
        } else {
            (offset, None)
        };

        Ok(Status::Complete((
            offset,
            Connect {
                client_id,
                will,
                username,
                password,
            },
        )))
    }
}

impl<'buf> Encodable for Connect<'buf> {
    fn encoded_len(&self) -> usize {
        self.client_id.encoded_len()
            + self.will.as_ref().map(|w| w.encoded_len()).unwrap_or(0)
            + self.username.as_ref().map(|u| u.encoded_len()).unwrap_or(0)
            + self.password.as_ref().map(|p| p.encoded_len()).unwrap_or(0)
    }

    fn encode(&self, bytes: &mut [u8]) -> Result<usize, EncodeError> {
        let mut offset = 0;

        offset += codec::string::encode_string(self.client_id, &mut bytes[offset..])?;

        if let Some(ref will) = self.will {
            offset += will.encode(&mut bytes[offset..])?;
        }

        if let Some(username) = self.username {
            offset += codec::string::encode_string(username, &mut bytes[offset..])?;
        }

        if let Some(password) = self.password {
            offset += codec::values::encode_bytes(password, &mut bytes[offset..])?;
        }

        Ok(offset)
    }
}
