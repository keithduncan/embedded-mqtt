#![no_std]

#![deny(warnings)]

#[cfg(test)]
extern crate std;

extern crate byteorder;

#[cfg(test)]
extern crate rayon;

pub mod error;
#[macro_use]
pub mod status;
pub mod fixed_header;
pub mod connect;
pub mod packet;
pub mod result;

mod string;
