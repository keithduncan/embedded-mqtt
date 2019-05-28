use crate::{
	connect::Connect,
    status::Status,
    result::Result,
};

#[derive(Debug)]
pub enum VariableHeader<'a> {
	Connect(Connect<'a>),
}

impl<'a> VariableHeader<'a> {
	pub fn connect(bytes: &'a [u8]) -> Result<Status<(usize, VariableHeader)>> {
		let (offset, connect) = complete!(Connect::from_bytes(bytes));
		Ok(Status::Complete((offset, VariableHeader::Connect(connect))))
	}
}
