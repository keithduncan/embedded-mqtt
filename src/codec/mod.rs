use crate::{
    error::{DecodeError, EncodeError},
    status::Status,
};

pub mod string;
pub mod values;

pub trait Decodable<'a>
where
    Self: core::marker::Sized,
{
    fn decode(bytes: &'a [u8]) -> Result<Status<(usize, Self)>, DecodeError>;
}

pub trait Encodable {
    fn encoded_len(&self) -> usize;
    fn encode(&self, bytes: &mut [u8]) -> Result<usize, EncodeError>;
}
