use core::{
    convert::{TryFrom, From},
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
            QoS::AtMostOnce  => 0b00,
            QoS::AtLeastOnce => 0b01,
            QoS::ExactlyOnce => 0b10,
        }
    }
}
