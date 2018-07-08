use core::{fmt, result};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Error {
    /// Invalid packet type in header
    PacketType,
    /// Invalid packet type flag in header
    PacketFlag,
    /// Malformed remaining length in header
    RemainingLength,
}

impl Error {
    fn desc(&self) -> &'static str {
        match *self {
            Error::PacketType => "invalid packet type in header",
            Error::PacketFlag => "invalid packet type flag in header",
            Error::RemainingLength => "malformed remaining length in header",
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.desc())
    }
}

#[cfg(feature = "std")]
impl ::std::error::Error for Error {
    fn description(&self) -> &str {
        self.desc()
    }
}

pub type Result<T> = result::Result<T, Error>;
