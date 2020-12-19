use core::{
    convert::{From, TryFrom},
    fmt,
    result::Result,
};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum QoS {
    AtMostOnce,
    AtLeastOnce,
    ExactlyOnce,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum Error {
    BadPattern,
}

impl TryFrom<u8> for QoS {
    type Error = Error;

    fn try_from(byte: u8) -> Result<QoS, Error> {
        let qos = match byte & 0b11 {
            0b00 => QoS::AtMostOnce,
            0b01 => QoS::AtLeastOnce,
            0b10 => QoS::ExactlyOnce,
            _ => return Err(Error::BadPattern),
        };

        Ok(qos)
    }
}

impl From<QoS> for u8 {
    fn from(qos: QoS) -> u8 {
        match qos {
            QoS::AtMostOnce => 0b00,
            QoS::AtLeastOnce => 0b01,
            QoS::ExactlyOnce => 0b10,
        }
    }
}

impl Error {
    fn desc(&self) -> &'static str {
        match *self {
            Error::BadPattern => "invalid QoS bit pattern",
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
