#![deny(warnings)]

extern crate core;

extern crate byteorder;

#[cfg(test)]
extern crate rayon;

pub mod error;
#[macro_use]
pub mod status;
pub mod header;
pub mod connect;
pub mod packet;
pub mod result;

mod string;
