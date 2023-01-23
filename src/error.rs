use core::num;
use serde::{de, ser};
use std::fmt::{Display, Formatter};
use std::{io, string};

pub type Result<T> = std::result::Result<T, Error>;

/// Error type that represent possible errors occurred
/// during serialization and deserialization.
///
/// Provide offset information of some errors for quick fixed location.
#[derive(Debug)]
pub enum Error {
    Message(String),
    Eof,
    Syntax(usize),
    TrailingCharacters,
    ExpectedSign(usize),
    UnexpectedCR(usize),
    UnexpectedSign{ expected: char, found: char, pos: usize },
    BulkStringOverflow,
    WrongSizeOfBulkString{ expected: usize, found: usize },
    FromUtf8Error(string::FromUtf8Error),
    IoError(io::Error),
    ParseIntError(num::ParseIntError),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Message(msg) => write!(f, "{}", msg),
            Error::Eof => write!(f, "unexpected end of input"),
            Error::Syntax(pos) => {
                write!(f, "expect one of these signs: + - : $ * in {}th bytes", pos)
            }
            Error::TrailingCharacters => write!(f, "trailing characters"),
            Error::ExpectedSign(pos) => write!(f, "expect sign in {}th bytes", pos),
            Error::UnexpectedCR(pos) => write!(f, "meet unexpected '\r' in {}th bytes", pos),
            Error::UnexpectedSign { expected, found, pos } =>
                write!(f, "found sign {} in pos {}, expected: {}", found, pos, expected),
            Error::BulkStringOverflow => write!(f, "bulk string overflow"),
            Error::WrongSizeOfBulkString{ expected, found } => write!(
                f,
                "wrong size of bulk string: expected {} bytes, found {} bytes",
                expected, found
            ),
            Error::FromUtf8Error(err) => write!(f, "{err}"),
            Error::IoError(err) => write!(f, "{err}"),
            Error::ParseIntError(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for Error {}

impl ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Error::Message(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Error::Message(msg.to_string())
    }
}

impl From<string::FromUtf8Error> for Error {
    fn from(err: string::FromUtf8Error) -> Self {
        Error::FromUtf8Error(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::IoError(err)
    }
}

impl From<num::ParseIntError> for Error {
    fn from(err: num::ParseIntError) -> Self {
        Error::ParseIntError(err)
    }
}

#[derive(Eq, PartialEq)]
pub enum ErrorKind {
    Message,
    Eof,
    Syntax,
    TrailingCharacters,
    ExpectedSign,
    UnexpectedCR,
    UnexpectedSign,
    BulkStringOverflow,
    WrongSizeOfBulkString,
    FromUtf8Error,
    IoError,
    ParseIntError,
}

impl Error {
    pub fn kind(&self) -> ErrorKind {
        match *self {
            Error::Message(_) => ErrorKind::Message,
            Error::Eof => ErrorKind::Eof,
            Error::Syntax(_) => ErrorKind::Syntax,
            Error::TrailingCharacters => ErrorKind::TrailingCharacters,
            Error::ExpectedSign{..} => ErrorKind::ExpectedSign,
            Error::UnexpectedCR(_) => ErrorKind::UnexpectedCR,
            Error::UnexpectedSign {..} => ErrorKind::UnexpectedSign,
            Error::BulkStringOverflow => ErrorKind::BulkStringOverflow,
            Error::WrongSizeOfBulkString{..} => ErrorKind::WrongSizeOfBulkString,
            Error::FromUtf8Error(_) => ErrorKind::FromUtf8Error,
            Error::IoError(_) => ErrorKind::IoError,
            Error::ParseIntError(_) => ErrorKind::ParseIntError
        }
    }
}
