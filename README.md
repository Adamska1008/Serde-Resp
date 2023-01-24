# Serde Resp

Serde Resp is used for serializing and deserializing Rust data structures from or into RESP message. It supports `to_string`, `to_writer` and `from_string` `from_reader` functions. More functions will be added soon.

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
use serde_resp::to_string;
use serde_resp::{array, simple, bulk, i64};
use serde_resp::Result;

fn main() -> Result<()> {
    let resp_arr = array!(i64!(32), simple!("foobar"), bulk!("really bulk"));
    assert_eq!(
        to_string(&resp_arr)?,
        "*3\r\n:32\r\n+foobar\r\n$11\r\nreally bulk\r\n"
    );
}
```

`*-1\r\n` or `$-1\r\n` will be both deserialized as `RESPType::None`. If you serialize a `RESPType::None`, you get `$-1\r\n`. It simplfy the process, base on practical experience. 

```rust
use serde_resp::RESPType;

let arr = "*3\r\n:32\r\n+foobar\r\n$11\r\nreally bulk\r\n";
let resp_arr: RESPType = serde_resp::from_str(arr)?;
assert_eq!(
    resp_arr, 
    array!(i64!(32), simple!("foobar"), bulk!("really bulk")
);
```
## Advantage
Sound error types with offset information for quick fixed location.

```rust
let simple_str = "+123\r124\r\n";
assert!(
    de::from_str::<RESPType>(simple_str)
        .is_err_and(|err| {
            if let Error::UnexpectedCR(pos) = err {
                return pos == 4;
            }
            false
        })
);
```