#![allow(warnings)]

use core::result::Result;

use crate::{
    status::Status,
    error::{DecodeError, EncodeError},
    codec::{self, Decodable, Encodable},
    variable_header::connect::Flags,
};

pub struct Will<'a, 'b> {
    topic: &'a str,
    message: &'b [u8],
}

impl<'buf> Decodable<'buf> for Will<'buf, 'buf> {
    fn from_bytes(bytes: &'buf [u8]) -> Result<Status<(usize, Will<'buf, 'buf>)>, DecodeError> {
        let offset = 0;
        let (offset, topic) = read!(codec::string::parse_string, bytes, offset);
        let (offset, message) = read!(codec::values::parse_bytes, bytes, offset);

        Ok(Status::Complete((offset, Will {
            topic,
            message,
        })))
    }
}

impl<'a, 'b> Encodable for Will<'a, 'b> {
    fn to_bytes(&self, bytes: &mut [u8]) -> Result<usize, EncodeError> {
        let offset = 0;
        let offset = codec::string::encode_string(self.topic, &mut bytes[offset..])?;
        let offset = codec::values::encode_bytes(self.message, &mut bytes[offset..])?;
        Ok(offset)
    }
}

pub struct Connect<'a, 'b, 'c, 'd, 'e> {
    client_id: &'a str,
    will: Option<Will<'b, 'c>>,
    username: Option<&'d str>,
    password: Option<&'e [u8]>,
}

impl<'buf> Connect<'buf, 'buf, 'buf, 'buf, 'buf> {
    pub fn from_bytes(flags: Flags, bytes: &'buf [u8]) -> Result<Status<(usize, Self)>, DecodeError> {
        let offset = 0;

        let (offset, client_id) = read!(codec::string::parse_string, bytes, offset);

        let (offset, will) = if flags.has_will() {
            let (offset, will) = read!(Will::from_bytes, bytes, offset);
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

        Ok(Status::Complete((offset, Connect {
            client_id,
            will,
            username,
            password,
        })))
    }
}

impl<'a, 'b, 'c, 'd, 'e> Encodable for Connect<'a, 'b, 'c, 'd, 'e> {
    fn to_bytes(&self, bytes: &mut [u8]) -> Result<usize, EncodeError> {
        let offset = 0;

        let offset = codec::string::encode_string(self.client_id, &mut bytes[offset..])?;
        let offset = if let Some(ref will) = self.will {
            will.to_bytes(&mut bytes[offset..])?
        } else {
            offset
        };

        let offset = if let Some(username) = self.username {
            codec::string::encode_string(username, &mut bytes[offset..])?
        } else {
            offset
        };

        let offset = if let Some(password) = self.password {
            codec::values::encode_bytes(password, &mut bytes[offset..])?
        } else {
            offset
        };

        Ok(offset)
    }
}
