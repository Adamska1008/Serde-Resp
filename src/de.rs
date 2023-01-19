use crate::{Error, RESPType, Result};
use serde::de::{DeserializeSeed, SeqAccess, Visitor};
use serde::{de, Deserialize};
use std::fmt::Formatter;

const MAX_BULK_STRING_SIZE: usize = 512 * 1024 * 1024;

pub struct Deserializer<'de> {
    input: &'de str,
    offset: usize,
}

impl<'de> Deserializer<'de> {
    pub fn from_str(input: &'de str) -> Self {
        Deserializer { input, offset: 0 }
    }
}

pub fn from_str<'a, T>(s: &'a str) -> Result<T>
where
    T: Deserialize<'a>,
{
    let mut de = Deserializer::from_str(s);
    let t = T::deserialize(&mut de)?;
    if de.input.is_empty() {
        Ok(t)
    } else {
        Err(Error::TrailingCharacters)
    }
}

impl<'de> Deserializer<'de> {
    // Check the first char while not consuming it.
    fn peek_char(&mut self) -> Result<char> {
        self.input.chars().next().ok_or(Error::Eof)
    }

    fn next_char(&mut self) -> Result<char> {
        let ch = self.peek_char()?;
        self.offset += ch.len_utf8();
        self.input = &self.input[ch.len_utf8()..];
        Ok(ch)
    }

    // Read {len} bytes, consume them.
    // May cause Error::Eof.
    fn skip(&mut self, len: usize) -> Result<&'de str> {
        if self.input.len() < len {
            return Err(Error::Eof);
        }
        let s: &'de str = &self.input[..len];
        self.input = &self.input[len..];
        self.offset += len;
        Ok(s)
    }

    // Reading until meet "\r\n".
    // Consume all reading bytes and return them.
    // Consume "\r\n" as well, but not return.
    fn read_to_end(&mut self) -> Result<&'de str> {
        match self.input.find("\r\n") {
            Some(len) => {
                if let Some(len) = self.input[..len].find('\r') {
                    Err(Error::UnexpectedCR(self.offset + len))
                } else {
                    let s = self.skip(len)?;
                    // skip "\r\n"
                    self.skip(2)?;
                    Ok(s)
                }
            }
            None => Err(Error::Eof),
        }
    }

    fn parse_int(&mut self) -> Result<i64> {
        let prefix = self.next_char()?;
        if prefix != ':' {
            return Err(Error::ExpectedSign(self.offset, ':'));
        }
        let str = self.read_to_end()?;
        let int = str.parse::<i64>()?;
        Ok(int)
    }

    fn parse_simple_string(&mut self) -> Result<&'de str> {
        let prefix = self.next_char()?;
        if prefix != '+' {
            return Err(Error::ExpectedSign(self.offset, '+'));
        }
        self.read_to_end()
    }

    fn parse_error(&mut self) -> Result<&str> {
        let prefix = self.next_char()?;
        if prefix != '-' {
            return Err(Error::ExpectedSign(self.offset, '-'));
        }
        self.read_to_end()
    }

    fn parse_bytes(&mut self) -> Result<Option<&'de [u8]>> {
        let prefix = self.next_char()?;
        if prefix != '$' {
            return Err(Error::ExpectedSign(self.offset, '$'));
        }
        let str: &'de str = self.read_to_end()?;
        let len = str.parse::<i32>()?;
        if len > MAX_BULK_STRING_SIZE as i32 {
            return Err(Error::BulkStringOverflow);
        }
        if len < 0 {
            return Ok(None)
        }
        if self.input.len() < len as usize {
            return Err(Error::Eof);
        }
        let bulk_str = self.skip(len as usize)?;
        // skip "\r\n"
        self.skip(2)?;
        Ok(Some(bulk_str.as_bytes()))
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.peek_char()? {
            '+' => self.deserialize_str(visitor),
            '-' => self.deserialize_string(visitor),
            ':' => self.deserialize_i64(visitor),
            '$' => self.deserialize_bytes(visitor),
            '*' => self.deserialize_seq(visitor),
            _ => Err(Error::UnexpectedType),
        }
    }

    fn deserialize_bool<V>(self, _: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_i8<V>(self, _: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_i16<V>(self, _: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_i32<V>(self, _: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i64(self.parse_int()?)
    }

    fn deserialize_u8<V>(self, _: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_u16<V>(self, _: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_u32<V>(self, _: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_u64<V>(self, _: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_f32<V>(self, _: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_f64<V>(self, _: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_char<V>(self, _: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_borrowed_str(self.parse_simple_string()?)
    }

    // Use this to deserialize error.
    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_str(self.parse_error()?)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.parse_bytes()? {
            Some(bytes) => visitor.visit_bytes(bytes),
            None => visitor.visit_none()
        }
    }

    fn deserialize_byte_buf<V>(self, _: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_option<V>(self, _: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_unit<V>(self, _: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_unit_struct<V>(self, _: &'static str, _: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_newtype_struct<V>(self, _: &'static str, _: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if self.peek_char()? == '*' {
            self.skip(1)?;
            let num = self.read_to_end()?.parse::<i32>()?;
            if num == -1 {
                return visitor.visit_none()
            }
            let value = visitor.visit_seq(RESPArrayAccess::new(self, num as usize))?;
            Ok(value)
        } else {
            Err(Error::ExpectedSign(self.offset, '*'))
        }
    }

    fn deserialize_tuple<V>(self, _: usize, _: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_tuple_struct<V>(self, _: &'static str, _: usize, _: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_map<V>(self, _: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_struct<V>(
        self,
        _: &'static str,
        _: &'static [&'static str],
        _: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_enum<V>(
        self,
        _: &'static str,
        _: &'static [&'static str],
        _: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_identifier<V>(self, _: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_ignored_any<V>(self, _: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }
}

struct RESPArrayAccess<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
    remain_cnt: usize,
}

impl<'a, 'de> RESPArrayAccess<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>, remain_cnt: usize) -> Self {
        RESPArrayAccess { de, remain_cnt }
    }
}

impl<'de, 'a> SeqAccess<'de> for RESPArrayAccess<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        if self.remain_cnt == 0 {
            return Ok(None);
        }
        self.remain_cnt -= 1;
        seed.deserialize(&mut *self.de).map(Some)
    }
}

struct RESPVisitor;

impl<'de> Visitor<'de> for RESPVisitor {
    type Value = RESPType;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("data matches Redis Simple Protocol")
    }

    fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(RESPType::Integer(v))
    }

    fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(RESPType::SimpleString(v.to_owned()))
    }

    // remember that this is used for error
    fn visit_string<E>(self, v: String) -> std::result::Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(RESPType::Error(v))
    }

    fn visit_bytes<E>(self, v: &[u8]) -> std::result::Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(RESPType::BulkString(v.to_vec()))
    }

    fn visit_none<E>(self) -> std::result::Result<Self::Value, E> where E: de::Error {
        Ok(RESPType::None)
    }

    fn visit_seq<A>(self, mut seq: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut array: Vec<RESPType> = vec![];
        while let Some(element) = seq.next_element()? {
            array.push(element);
        }
        Ok(RESPType::Array(array))
    }
}

impl<'de> Deserialize<'de> for RESPType {
    fn deserialize<D>(de: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        de.deserialize_any(RESPVisitor)
    }
}
