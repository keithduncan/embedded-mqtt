/// The result of a successful parse pass. Taken from the `httparse` crate.
///
/// `Complete` is used when the buffer contained the complete value.
/// `Partial` is used when parsing did not reach the end of the expected value,
/// but no invalid data was found.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Status<T> {
    /// The completed result.
    Complete(T),
    /// A partial result and how much is needed to continue parsing.
    Partial(usize),
}

impl<T> Status<T> {
    /// Convenience method to check if status is complete.
    #[inline]
    pub fn is_complete(&self) -> bool {
        match *self {
            Status::Complete(..) => true,
            Status::Partial(..) => false,
        }
    }

    /// Convenience method to check if status is partial.
    #[inline]
    pub fn is_partial(&self) -> bool {
        match *self {
            Status::Complete(..) => false,
            Status::Partial(..) => true,
        }
    }

    /// Convenience method to unwrap a Complete value. Panics if the status is
    /// `Partial`.
    #[inline]
    pub fn unwrap(self) -> T {
        match self {
            Status::Complete(t) => t,
            Status::Partial(..) => panic!("Tried to unwrap Status::Partial"),
        }
    }
}

#[macro_export]
macro_rules! complete {
    ($e:expr) => {
        match $e? {
            Status::Complete(v) => v,
            Status::Partial(x) => return Ok(Status::Partial(x)),
        }
    };
}

macro_rules! read {
    ($fn:path, $bytes:expr, $offset:expr) => {
        match $fn(&$bytes[$offset..])? {
            Status::Complete(v) => ($offset + v.0, v.1),
            Status::Partial(x) => return Ok(Status::Partial(x)),
        }
    };
}
