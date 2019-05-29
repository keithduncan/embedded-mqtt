use core::{
    convert::From,
    fmt,
    str::Utf8Error,
};

use crate::qos;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum DecodeError {
    /// Invalid packet type in header
    PacketType,
    /// Invalid packet type flag in header
    PacketFlag,
    /// Malformed remaining length in header
    RemainingLength,
    /// Invalid buffer length
    InvalidLength,
    /// Invalid UTF-8 encoding
    Utf8,
    /// Invalid QoS value
    InvalidQoS(qos::Error),
    /// Invalid protocol level
    InvalidProtocolLevel,
    /// Invalid connect flag value
    InvalidConnectFlag,
    /// Invalid Connack flag
    InvalidConnackFlag,
    /// Invalid Connack Return Code
    InvalidConnackReturnCode,
}

impl DecodeError {
    fn desc(&self) -> &'static str {
        match *self {
            DecodeError::PacketType => "invalid packet type in header",
            DecodeError::PacketFlag => "invalid packet type flag in header",
            DecodeError::RemainingLength => "malformed remaining length in header",
            DecodeError::InvalidLength => "invalid buffer length",
            DecodeError::Utf8 => "invalid utf-8 encoding",
            DecodeError::InvalidQoS(_) => "invalid QoS bit pattern",
            DecodeError::InvalidProtocolLevel => "invalid protocol level",
            DecodeError::InvalidConnectFlag => "invalid connect flag value",
            DecodeError::InvalidConnackFlag => "invalid connack flag value",
            DecodeError::InvalidConnackReturnCode => "invalid connack return code",
        }
    }
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.desc())
    }
}

#[cfg(feature = "std")]
impl ::std::error::Error for DecodeError {
    fn description(&self) -> &str {
        self.desc()
    }
}

impl From<Utf8Error> for DecodeError {
    fn from(_: Utf8Error) -> Self {
        DecodeError::Utf8
    }
}

impl From<qos::Error> for DecodeError {
    fn from(err: qos::Error) -> Self {
        DecodeError::InvalidQoS(err)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum EncodeError {
    /// Not enough space in buffer to encode
    OutOfSpace,
    /// Value too big for field
    ValueTooBig,
}

impl EncodeError {
    fn desc(&self) -> &'static str {
        match *self {
            EncodeError::OutOfSpace => "not enough space in encode buffer",
            EncodeError::ValueTooBig => "value too big to ever be encoded"
        }
    }
}

impl fmt::Display for EncodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.desc())
    }
}

#[cfg(feature = "std")]
impl ::std::error::Error for EncodeError {
    fn description(&self) -> &str {
        self.desc()
    }
}

impl From<core::num::TryFromIntError> for EncodeError {
    fn from(_err: core::num::TryFromIntError) -> EncodeError {
        EncodeError::ValueTooBig
    }
}
