// Copyright 2016 Martin Grabmueller. See the LICENSE file at the
// top-level directory of this distribution for license information.

//! Mudstuck errors and machinery to make them work with `try!'.

use std::io;
use uuid;
use std::string;
use std::fmt;
use std::error;

/// Errors that may happen during operation.
#[derive(Debug)]
pub enum Error {
    /// IO Error.
    Io(io::Error),
    /// Error on UUID handling.
    UuidParse(uuid::ParseError),
    /// General error in UTF-8 encoding/decpding handling.
    Utf8(string::FromUtf8Error),
    /// Failure parsing a command.
    CommandParse(&'static str),
    /// Some unimplemented functionality was requested.
    Unimplemented(&'static str),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Io(ref err) => write!(f, "IO error: {}", err),
            Error::UuidParse(ref err) => write!(f, "UUID error: {}", err),
            Error::Utf8(ref err) => write!(f, "UTF-8 error: {}", err),
            Error::CommandParse(ref err) => write!(f, "cannot parse command: {}", err),
            Error::Unimplemented(ref err) => write!(f, "unimplemented: {}", err),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Io(ref err) => err.description(),
            Error::UuidParse(_) => "uuid parse error",
            Error::Utf8(ref err) => err.description(),
            Error::CommandParse(_) => "command parse error",
            Error::Unimplemented(_) => "unimplemented",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::Io(ref err) => Some(err),
            Error::UuidParse(_) => None,
            Error::Utf8(ref err) => Some(err),
            Error::CommandParse(_) => None,
            Error::Unimplemented(_) => None,
       } 
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<uuid::ParseError> for Error {
    fn from(err: uuid::ParseError) -> Error {
        Error::UuidParse(err)
    }
}

impl From<string::FromUtf8Error> for Error {
    fn from(err: string::FromUtf8Error) -> Error {
        Error::Utf8(err)
    }
}
