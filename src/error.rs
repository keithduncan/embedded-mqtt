use core::convert::From;
use core::io::Error as IoError;
use core::{fmt, result};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Error {
    /// I/O error
    Io,
    /// Invalid packet type in header
    PacketType,
    /// Invalid packet type flag in header
    PacketFlag,
}

impl Error {
    fn desc(&self) -> &'static str {
        match *self {
            Error::Io => "i/o error",
            Error::PacketType => "invalid packet type in header",
            Error::PacketFlag => "invalid packet type flag in header",
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

impl From<IoError> for Error {
    fn from(_: IoError) -> Self {
        // TODO: See if we can avoid ignoring the underlying I/O error.
        Error::Io
    }
}

pub type Result<T> = result::Result<T, Error>;
