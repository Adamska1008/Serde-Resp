use crate::error::Error::UnexpectedType;
use crate::error::{Error, Result};
use crate::RESPType;
use serde::ser::{Impossible, SerializeSeq};
use serde::{ser, Serialize};
use std::io::Write;

pub struct Serializer<W: Write> {
    buffer: itoa::Buffer,
    writer: W,
}

pub fn to_string<T: Serialize>(value: &T) -> Result<String> {
    let mut buf: Vec<u8> = Vec::new();
    to_writer(value, &mut buf)?;
    Ok(String::from_utf8(buf)?)
}

pub fn to_writer<T, W>(value: &T, writer: &mut W) -> Result<()>
where
    T: Serialize,
    W: Write,
{
    let mut serializer = Serializer {
        buffer: itoa::Buffer::new(),
        writer,
    };
    value.serialize(&mut serializer)?;
    Ok(())
}

impl<'a, W: Write> ser::Serializer for &'a mut Serializer<W> {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Impossible<(), Error>;
    type SerializeMap = Impossible<(), Error>;
    type SerializeStruct = Impossible<(), Error>;
    type SerializeStructVariant = Impossible<(), Error>;

    fn serialize_bool(self, v: bool) -> Result<()> {
        if v {
            self.serialize_i64(1i64)
        } else {
            self.serialize_i64(0i64)
        }
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        let content = format!(":{}\r\n", self.buffer.format(v));
        self.writer.write_all(content.as_bytes())?;
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        if let Ok(v) = i64::try_from(v) {
            self.serialize_i64(v)
        } else {
            Err(Error::IntegerOverflow)
        }
    }

    fn serialize_f32(self, _: f32) -> Result<()> {
        Err(Error::UnexpectedType)
    }

    fn serialize_f64(self, _: f64) -> Result<()> {
        Err(Error::UnexpectedType)
    }

    fn serialize_char(self, v: char) -> Result<()> {
        self.writer.write(&[v as u8])?;
        Ok(())
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        self.writer.write_all(v.as_bytes())?;
        self.writer.write_all(b"\r\n")?;
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        let prefix = format!("${}\r\n", self.buffer.format(v.len()));
        self.writer.write_all(prefix.as_bytes())?;
        self.writer.write_all(v)?;
        self.writer.write_all(b"\r\n")?;
        Ok(())
    }

    fn serialize_none(self) -> Result<()> {
        self.writer.write_all(b"$-1\r\n")?;
        Ok(())
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<()> {
        self.serialize_none()
    }

    fn serialize_unit_struct(self, _: &'static str) -> Result<()> {
        Err(UnexpectedType)
    }

    fn serialize_unit_variant(self, _: &'static str, _: u32, _: &'static str) -> Result<()> {
        Err(UnexpectedType)
    }

    fn serialize_newtype_struct<T: ?Sized>(self, _: &'static str, _: &T) -> Result<()>
    where
        T: Serialize,
    {
        Err(UnexpectedType)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: &T,
    ) -> Result<()>
    where
        T: Serialize,
    {
        Err(UnexpectedType)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        match len {
            Some(x) => self.writer.write_all((&format!("*{x}\r\n")).as_bytes())?,
            None => self.writer.write_all((&format!("*-1\r\n")).as_bytes())?,
        }
        Ok(self)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Err(UnexpectedType)
    }

    fn serialize_map(self, _: Option<usize>) -> Result<Self::SerializeMap> {
        Err(UnexpectedType)
    }

    fn serialize_struct(self, _: &'static str, _: usize) -> Result<Self::SerializeStruct> {
        Err(UnexpectedType)
    }

    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Err(UnexpectedType)
    }
}

impl<'a, W: Write> ser::SerializeSeq for &'a mut Serializer<W> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, W: Write> ser::SerializeTuple for &'a mut Serializer<W> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}
impl<'a, W: Write> ser::SerializeTupleStruct for &'a mut Serializer<W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl Serialize for RESPType {
    fn serialize<S>(
        &self,
        ser: S,
    ) -> std::result::Result<<S as serde::Serializer>::Ok, <S as serde::Serializer>::Error>
    where
        S: serde::Serializer,
    {
        match self {
            RESPType::SimpleString(str) => ser.serialize_str(&("+".to_owned() + str)),
            RESPType::Integer(num) => ser.serialize_i64(*num),
            RESPType::Error(err) => ser.serialize_str(&("-".to_owned() + err)),
            RESPType::BulkString(str) => ser.serialize_bytes(str),
            RESPType::Array(arr) => {
                let mut ser = ser.serialize_seq(Some(arr.len()))?;
                for val in arr {
                    ser.serialize_element(val)?;
                }
                ser.end()
            },
            RESPType::None => ser.serialize_none()
        }
    }
}

#[cfg(test)]
mod ser_test {
    use crate::ser::to_string;
    use crate::RESPType;
    use crate::Result;

    #[test]
    fn test_simple_string() -> Result<()> {
        let resp_sstr = RESPType::SimpleString("hello world".to_string());
        assert_eq!(to_string(&resp_sstr)?, "+hello world\r\n");
        Ok(())
    }

    #[test]
    fn test_bulk_string() -> Result<()> {
        let buf = b"Hello, world!";
        let resp_bstr = RESPType::BulkString(buf.to_vec());
        assert_eq!(
            to_string(&resp_bstr)?,
            format!("${}\r\nHello, world!\r\n", buf.len())
        );
        Ok(())
    }

    #[test]
    fn test_error() -> Result<()> {
        let err = "Err some errors";
        let resp_err = RESPType::Error(err.to_owned());
        assert_eq!(to_string(&resp_err)?, "-Err some errors\r\n");
        Ok(())
    }

    #[test]
    fn test_int() -> Result<()> {
        let int = 114514i64;
        let resp_int = RESPType::Integer(int);
        assert_eq!(to_string(&resp_int)?, ":114514\r\n");
        Ok(())
    }

    #[test]
    fn test_array() -> Result<()> {
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
        Ok(())
    }
}
