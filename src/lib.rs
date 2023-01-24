#![feature(is_some_and)]
extern crate core;

pub mod de;
pub mod error;
pub mod ser;
pub mod marco;

pub use crate::error::{Error, Result};
pub use crate::resp_type::RESPType;

pub use crate::de::{from_str, from_reader};
pub use crate::ser::{to_string, to_writer};

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

    impl RESPType {
        pub fn ok() -> RESPType {
            RESPType::SimpleString("OK".to_owned())
        }
    }
}
