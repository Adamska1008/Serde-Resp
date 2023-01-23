#![feature(is_some_and)]
extern crate core;

pub mod de;
pub mod error;
pub mod ser;

pub use crate::error::{Error, Result};
pub use crate::resp_type::RESPType;

pub mod resp_type {
    #[derive(Debug, Eq, PartialEq)]
    pub enum RESPType {
        SimpleString(String),
        Integer(i64),
        Error(String),
        BulkString(Vec<u8>),
        Array(Vec<RESPType>),
        None
    }
}
