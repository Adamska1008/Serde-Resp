extern crate core;

pub mod de;
pub mod error;
pub mod ser;

pub use crate::error::{Error, Result};
pub use crate::resp_type::RESPType;

pub mod resp_type {
    pub enum RESPType {
        SimpleString(String),
        Integer(i64),
        Error(String),
        BulkString(Vec<u8>),
        Array(Vec<RESPType>),
        None
    }
}
