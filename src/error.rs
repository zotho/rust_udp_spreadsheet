use std::num::ParseIntError;
use std::io::Error as IoError;

use mysql;

#[derive(Debug)]
pub struct Error {
    pub details: String
}

impl Error {
    pub fn new(msg: &str) -> Error {
        Error{details: msg.to_string()}
    }
}

impl From<ParseIntError> for Error {
    fn from(err: ParseIntError) -> Self {
        Error::new(err.to_string().as_str())
    }
}

impl From<mysql::Error> for Error {
    fn from(err: mysql::Error) -> Self {
        Error::new(err.to_string().as_str())
    }
}

impl From<IoError> for Error {
    fn from(err: IoError) -> Self {
        Error::new(err.to_string().as_str())
    }
}