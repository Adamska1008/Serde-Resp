use core::num;
use serde::{de, ser};
use std::fmt::{Display, Formatter};
use std::{io, string};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Message(String),
    Eof,
    Syntax(usize),
    TrailingCharacters,
    ExpectedSign(usize, char),
    ExpectedBulkString(usize),
    ExpectedArrayElement(usize),
    UnexpectedCR(usize),
    UnexpectedType,
    IntegerOverflow,
    BulkStringOverflow,
    WrongSizeOfBulkString(usize, usize),
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
            Error::ExpectedSign(pos, sign) => write!(f, "expect {} in {}th bytes", sign, pos),
            Error::ExpectedBulkString(pos) => write!(f, "expect bulk string in {}th bytes", pos),
            Error::ExpectedArrayElement(pos) => {
                write!(f, "expect array element in {}th bytes", pos)
            }
            Error::UnexpectedCR(pos) => write!(f, "meet unexpected '\r' in {}th bytes", pos),
            Error::UnexpectedType => write!(f, "unexpected type"),
            Error::IntegerOverflow => write!(f, "integer overflow"),
            Error::BulkStringOverflow => write!(f, "bulk string overflow"),
            Error::WrongSizeOfBulkString(expected, found) => write!(
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
