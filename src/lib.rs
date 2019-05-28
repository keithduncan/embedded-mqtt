#![no_std]

#![deny(warnings)]

#[cfg(test)]
extern crate std;
#[cfg(test)]
extern crate rayon;

extern crate byteorder;

#[macro_use]
extern crate bitfield;

#[macro_use]
pub mod status;
pub mod result;
pub mod error;

mod decode;

pub mod fixed_header;
pub mod variable_header;
pub mod packet;

pub mod qos;
