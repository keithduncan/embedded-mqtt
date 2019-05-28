use core::{
    convert::From,
    fmt,
    str::Utf8Error,
};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ParseError {
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
    /// Invalid protocol level
    InvalidProtocolLevel,
    /// Invalid connect flag value
    InvalidConnectFlag,
    /// Invalid Connack flag
    InvalidConnackFlag,
    /// Invalid Connack Return Code
    InvalidConnackReturnCode,
}

impl ParseError {
    fn desc(&self) -> &'static str {
        match *self {
            ParseError::PacketType => "invalid packet type in header",
            ParseError::PacketFlag => "invalid packet type flag in header",
            ParseError::RemainingLength => "malformed remaining length in header",
            ParseError::InvalidLength => "invalid buffer length",
            ParseError::Utf8 => "invalid utf-8 encoding",
            ParseError::InvalidProtocolLevel => "invalid protocol level",
            ParseError::InvalidConnectFlag => "invalid connect flag value",
            ParseError::InvalidConnackFlag => "invalid connack flag value",
            ParseError::InvalidConnackReturnCode => "invalid connack return code",
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.desc())
    }
}

#[cfg(feature = "std")]
impl ::std::error::Error for ParseError {
    fn description(&self) -> &str {
        self.desc()
    }
}

impl From<Utf8Error> for ParseError {
    fn from(_: Utf8Error) -> Self {
        ParseError::Utf8
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum EncodeError {
    /// Not enough space in buffer to encode
    OutOfSpace,
}

impl EncodeError {
    fn desc(&self) -> &'static str {
        match *self {
            EncodeError::OutOfSpace => "not enough space in encode buffer",
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
