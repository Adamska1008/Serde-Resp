# Serde Resp

Serde Resp is used for serializing and deserializing Rust data structures from or into RESP message. It supports `to_string`, `to_writer` and `from_string` in serde. More functions will be added soon.

## Usage
The repository use `RESPType` to represent specific RESP data format. Their definition is:
```rust
#[derive(Debug, Eq, PartialEq)]
pub enum RESPType {
    SimpleString(String),
    Integer(i64),
    Error(String),
    BulkString(Vec<u8>),
    Array(Vec<RESPType>),
    None
}
```
Always use RESPType to serialize RESP data format, or the efficiency and usability is not guaranteed.

```rust
use crate::ser::to_string;
use crate::RESPType;
use crate::Result;

fn main() {
    let arr = vec![
        RESPType::Integer(32),
        RESPType::SimpleString("foobar".to_owned()),
        RESPType::BulkString("really bulk".as_bytes().to_vec()),
    ];
    let resp_arr = RESPType::Array(arr);
    assert_eq!(
        to_string(&resp_arr)?,
        "*3\r\n:32\r\n+foobar\r\n$11\r\nreally bulk\r\n"
    );
}
```

`*-1\r\n` or `$-1\r\n` will be both deserialized as `RESPType::None`. If you serialize a `RESPType::None`, you get `$-1\r\n`. It simplfy the process, base on practical experience. 

## Advantage
Sound error types with offset information for quick fixed location.