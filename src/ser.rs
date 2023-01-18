use serde::{ser, Serialize};
use serde::ser::Impossible;
use crate::error::{Error, Result};
use crate::error::Error::UnexpectedType;

pub struct Serializer {
    output: String
}

pub fn to_string<T>(value: &T) -> Result<String>
where T: Serialize
{
    let mut serializer = Serializer {
        output: String::new(),
    };
    value.serialize(&mut serializer)?;
    Ok(serializer.output)
}

impl<'a> ser::Serializer for &'a mut Serializer {
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
        self.output += &format!(":{}\r\n", v);
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
        self.output += &format!("$1\r\n{}\r\n", v);
        Ok(())
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        self.output += &format!("${}\r\n{}\r\n", v.len(), v);
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        self.output += &format!("${}\r\n{}\r\n", v.len(), String::from_utf8_lossy(v));
        Ok(())
    }

    fn serialize_none(self) -> Result<()> {
        self.serialize_unit()
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<()> where T: Serialize {
        value.serialize(self)
    }


    fn serialize_unit(self) -> Result<()> {
        self.output += "$-1\r\n";
        Ok(())
    }

    fn serialize_unit_struct(self, _: &'static str) -> Result<()> {
        Err(UnexpectedType)
    }

    fn serialize_unit_variant(self, _: &'static str, _: u32, _: &'static str) -> Result<()> {
        Err(UnexpectedType)
    }

    fn serialize_newtype_struct<T: ?Sized>(self, _: &'static str, _: &T) -> Result<()> where T: Serialize {
        Err(UnexpectedType)
    }

    fn serialize_newtype_variant<T: ?Sized>(self, _: &'static str, _: u32, _: &'static str, _: &T) -> Result<()>
        where T: Serialize
    {
        Err(UnexpectedType)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        match len {
            Some(x) => self.output += &format!("*{x}\r\n"),
            None => self.output += &format!("*0\r\n")
        }
        Ok(self)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(self, _: &'static str, len: usize) -> Result<Self::SerializeTupleStruct> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(self, _: &'static str, _: u32, _: &'static str, _: usize) -> Result<Self::SerializeTupleVariant> {
        Err(UnexpectedType)
    }

    fn serialize_map(self, _: Option<usize>) -> Result<Self::SerializeMap> {
        Err(UnexpectedType)
    }

    fn serialize_struct(self, _: &'static str, _: usize) -> Result<Self::SerializeStruct> {
        Err(UnexpectedType)
    }

    fn serialize_struct_variant(self, _: &'static str, _: u32, _: &'static str, _: usize) -> Result<Self::SerializeStructVariant> {
        Err(UnexpectedType)
    }
}

impl<'a> ser::SerializeSeq for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where T: Serialize {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeTuple for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where T: Serialize {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}
impl<'a> ser::SerializeTupleStruct for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where T: Serialize {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod serializer_test {
    use crate::error::Result;
    use crate::ser;

    #[test]
    fn test_string() -> Result<()> {
        let str = "Hello, world!";
        let expected = format!("${}\r\n{}\r\n", str.len(), str);
        assert_eq!(ser::to_string(&str)?, expected);
        let str = str.to_owned();
        assert_eq!(ser::to_string(&str)?, expected);
        Ok(())
    }

    #[test]
    fn test_number() -> Result<()>{
        let number = 114514;
        let expected = ":114514\r\n";
        assert_eq!(ser::to_string(&number)?, expected);
        Ok(())
    }

    #[test]
    fn test_null() -> Result<()> {
        let expected = "$-1\r\n";
        let null: Option<&str> = None;
        assert_eq!(ser::to_string(&null)?, expected);
        Ok(())
    }

    #[test]
    fn test_array() -> Result<()> {
        let array = [1, 2, 3, 4, 5];
        let expected = "*5\r\n:1\r\n:2\r\n:3\r\n:4\r\n:5\r\n";
        assert_eq!(ser::to_string(&array)?, expected);
        let array = ["hello", "world"];
        let expected = "*2\r\n$5\r\nhello\r\n$5\r\nworld\r\n";
        assert_eq!(ser::to_string(&array)?,expected);
        Ok(())
    }

    #[test]
    fn test_tuple() -> Result<()> {
        let tuple = (32, 7, "abcd");
        let expected = "*3\r\n:32\r\n:7\r\n$4\r\nabcd\r\n";
        assert_eq!(ser::to_string(&tuple)?, expected);
        Ok(())
    }
}
