use crate::{
    string,
    status::Status,
    result::Result,
};

pub struct Connect<'buf> {
    name: &'buf str,
    revision: u8,
    flags: u8,
}

impl<'buf> Connect<'buf> {
    pub fn from_bytes(bytes: &[u8]) -> Result<Status<(usize, Connect)>> {
        // read protocol name
        let (offset, name) = complete!(string::parse_string(bytes));

        // read protocol revision
        let (offset, revision) = next!(bytes, offset);

        // read protocol flags
        let (offset, flags) = next!(bytes, offset);

        Ok(Status::Complete((offset, Connect {
            name,
            revision,
            flags,
        })))
    }

    pub fn name(&self) -> &str {
        self.name
    }

    pub fn revision(&self) -> &u8 {
        &self.revision
    }

    pub fn flags(&self) -> &u8 {
        &self.flags
    }
}
