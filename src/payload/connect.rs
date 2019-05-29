#![allow(warnings)]

pub struct Will<'a, 'b> {
	topic: &'a str,
	message: &'b [u8],
}

pub struct Connect<'a, 'b, 'c, 'd, 'e> {
	client_id: &'a str,
	will: Option<Will<'b, 'c>>,
	username: &'d str,
	password: &'e [u8],
}
