use serde::{ser, Serialize};
use serde::ser::Impossible;
use crate::error::{Error, Result};

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
    type SerializeSeq = Impossible<(), Error>;
    type SerializeTuple = Impossible<(), Error>;
    type SerializeTupleStruct = Impossible<(), Error>;
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
        self.output += "$-1\r\n";
        Ok(())
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<()> where T: Serialize {
        todo!()
    }

    fn serialize_unit(self) -> Result<()> {
        todo!()
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<()> {
        todo!()
    }

    fn serialize_unit_variant(self, name: &'static str, variant_index: u32, variant: &'static str) -> Result<()> {
        todo!()
    }

    fn serialize_newtype_struct<T: ?Sized>(self, name: &'static str, value: &T) -> Result<()> where T: Serialize {
        todo!()
    }

    fn serialize_newtype_variant<T: ?Sized>(self, name: &'static str, variant_index: u32, variant: &'static str, value: &T) -> Result<()>
        where T: Serialize
    {
        todo!()
    }

    fn serialize_seq(self, len: Option<usize>) -> std::result::Result<Self::SerializeSeq, Self::Error> {
        todo!()
    }

    fn serialize_tuple(self, len: usize) -> std::result::Result<Self::SerializeTuple, Self::Error> {
        todo!()
    }

    fn serialize_tuple_struct(self, name: &'static str, len: usize) -> std::result::Result<Self::SerializeTupleStruct, Self::Error> {
        todo!()
    }

    fn serialize_tuple_variant(self, name: &'static str, variant_index: u32, variant: &'static str, len: usize) -> std::result::Result<Self::SerializeTupleVariant, Self::Error> {
        todo!()
    }

    fn serialize_map(self, len: Option<usize>) -> std::result::Result<Self::SerializeMap, Self::Error> {
        todo!()
    }

    fn serialize_struct(self, name: &'static str, len: usize) -> std::result::Result<Self::SerializeStruct, Self::Error> {
        todo!()
    }

    fn serialize_struct_variant(self, name: &'static str, variant_index: u32, variant: &'static str, len: usize) -> std::result::Result<Self::SerializeStructVariant, Self::Error> {
        todo!()
    }
}