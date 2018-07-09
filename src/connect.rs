use super::{parse_string, Result, Status};

pub struct Connect<'buf> {
    name: &'buf str,
    revision: u8,
    flags: u8,
}

impl<'buf> Connect<'buf> {
    pub fn from_bytes(bytes: &[u8]) -> Result<Status<Connect>> {
        // read protocol name
        let name = complete!(parse_string(bytes));
        let mut read = 2 + name.len(); // 2 bytes for the string len prefix + length of string in bytes

        // read protocol revision
        let revision = next!(bytes, read);
        read += 1;

        // read protocol flags
        let flags = next!(bytes, read);

        Ok(Status::Complete(Connect {
            name,
            revision,
            flags,
        }))
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
