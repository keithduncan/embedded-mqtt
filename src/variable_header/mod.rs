use crate::{
    status::Status,
    result::Result,
};

pub mod connect;

#[derive(Debug)]
pub enum VariableHeader<'a> {
	Connect(connect::Connect<'a>),
}

impl<'a> VariableHeader<'a> {
	pub fn connect(bytes: &'a [u8]) -> Result<Status<(usize, VariableHeader)>> {
		let (offset, connect) = complete!(connect::Connect::from_bytes(bytes));
		Ok(Status::Complete((offset, VariableHeader::Connect(connect))))
	}
}