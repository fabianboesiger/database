use std::{fmt, io, error};

#[derive(Debug)]
pub struct Error {
    description: String
}

impl Error {
    pub fn new(description: String) -> Error {
        Error {
            description
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description)
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        &self.description
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Error {
        Error::new(format!("IO Error: {}", error))
    }
}