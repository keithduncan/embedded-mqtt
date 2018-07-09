#![deny(warnings)]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate byteorder;
#[cfg(feature = "std")]
extern crate std as core;

#[cfg(test)]
extern crate rayon;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum QoS {
    AtMostOnce,
    AtLeastOnce,
    ExactlyOnce,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum PacketType {
    Connect,
    Connack,
    Publish,
    Puback,
    Pubrec,
    Pubrel,
    Pubcomp,
    Subscribe,
    Suback,
    Unsubscribe,
    Unsuback,
    Pingreq,
    Pingresp,
    Disconnect,
}

pub type PacketTypeFlags = u8;
pub type PacketId = u16;

pub mod error;
pub use error::{Error, Result};

pub mod header;
pub use header::Header;
