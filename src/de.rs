use crate::{Error, RESPType, Result};
use serde::de::{DeserializeOwned, DeserializeSeed, SeqAccess, Visitor};
use serde::{de, Deserialize};
use std::fmt::Formatter;
use std::io::Read;

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

pub fn from_str<T>(s: & str) -> Result<T>
where
    T: DeserializeOwned,
{
    let mut de = Deserializer::from_str(s);
    let t = T::deserialize(&mut de)?;
    if de.input.is_empty() {
        Ok(t)
    } else {
        Err(Error::TrailingCharacters)
    }
}

pub fn from_reader<R, T>(reader: &mut R) -> Result<T>
where
    R: Read,
    T: DeserializeOwned
{
    let mut buf= Vec::new();
    reader.read_to_end(&mut buf)?;
    let s = String::from_utf8(buf)?;
    let mut de = Deserializer::from_str(&s);
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
    // If not found "\r\n", return Error::Eof
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

    // Assume the next part is an integer and read it.
    // Consume all the reading bytes.
    fn parse_int(&mut self) -> Result<i64> {
        let prefix = self.peek_char()?;
        if prefix != ':' {
            return Err(Error::UnexpectedSign{ found: prefix, expected: ':', pos: self.offset});
        }
        self.next_char()?;
        let str = self.read_to_end()?;
        let int = str.parse::<i64>()?;
        Ok(int)
    }

    // Assume the next part is a simple string and read it.
    // Consume all the reading bytes.
    fn parse_simple_string(&mut self) -> Result<&'de str> {
        let prefix = self.peek_char()?;
        if prefix != '+' {
            return Err(Error::UnexpectedSign{ found: prefix, expected: '+', pos: self.offset});
        }
        self.next_char()?;
        self.read_to_end()
    }

    // Assume the next part is an error and read it.
    // Consume all the reading bytes.
    fn parse_error(&mut self) -> Result<&str> {
        let prefix = self.peek_char()?;
        if prefix != '-' {
            return Err(Error::UnexpectedSign{ found: prefix, expected: '-', pos: self.offset});
        }
        self.next_char()?;
        self.read_to_end()
    }

    // Assume the next part is a bulk string and read it.
    // Consume all the reading bytes.
    fn parse_bytes(&mut self) -> Result<Option<&'de [u8]>> {
        let prefix = self.peek_char()?;
        if prefix != '$' {
            return Err(Error::UnexpectedSign{ found: prefix, expected: '$', pos: self.offset});
        }
        self.next_char()?;
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
            _ => Err(Error::ExpectedSign(self.offset)),
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
        visitor.visit_string(self.parse_error()?.to_owned())
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
            Err(Error::UnexpectedSign{pos: self.offset, found: self.peek_char()?, expected: '*'})
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

#[cfg(test)]
mod de_test {
    use crate::{de, Error, RESPType};
    use crate::error::ErrorKind;
    use crate::Result;

    #[test]
    fn test_simple_string() -> Result<()> {
        let buf = "+hello\r\n";
        let sstr: RESPType = de::from_str(buf)?;
        assert_eq!(sstr, RESPType::SimpleString("hello".to_string()));
        Ok(())
    }

    #[test]
    fn test_error() -> Result<()> {
        let err = "-Err unknown error\r\n";
        let resp_err: RESPType = de::from_str(err)?;
        assert_eq!(resp_err, RESPType::Error("Err unknown error".to_string()));
        Ok(())
    }

    #[test]
    fn test_integer() -> Result<()> {
        let int = ":114514\r\n";
        let resp_int: RESPType = de::from_str(int)?;
        assert_eq!(resp_int, RESPType::Integer(114514));
        Ok(())
    }

    #[test]
    fn test_bulk_string() -> Result<()> {
        let str = "$13\r\nhello, world!\r\n";
        let resp_str: RESPType = de::from_str(str)?;
        assert_eq!(resp_str, RESPType::BulkString("hello, world!".as_bytes().to_vec()));
        Ok(())
    }

    #[test]
    fn test_array() -> Result<()> {
        let arr = "*3\r\n:32\r\n+foobar\r\n$11\r\nreally bulk\r\n";
        let resp_arr: RESPType = de::from_str(arr)?;
        assert_eq!(resp_arr, RESPType::Array(vec![
            RESPType::Integer(32),
            RESPType::SimpleString("foobar".to_owned()),
            RESPType::BulkString("really bulk".as_bytes().to_vec()),
        ]));
        Ok(())
    }

    #[test]
    fn test_null() -> Result<()> {
        let null_bulk_str = "$-1\r\n";
        let null_array = "*-1\r\n";
        let resp_null: RESPType = de::from_str(null_bulk_str)?;
        assert_eq!(resp_null, RESPType::None);
        let resp_null: RESPType = de::from_str(null_array)?;
        assert_eq!(resp_null, RESPType::None);
        Ok(())
    }

    #[test]
    fn test_error_eof() -> Result<()>{
        let bulk_str = "$6\r\nhello\r\n";
        assert!(
            de::from_str::<RESPType>(bulk_str)
                .is_err_and(|err| err.kind() == ErrorKind::Eof )
        );
        Ok(())
    }

    #[test]
    fn test_error_expected_sign() -> Result<()> {
        let array = "*2\r\n+514\r\n12\r\n";
        assert!(
            de::from_str::<RESPType>(array)
                .is_err_and(|err| {
                    if let Error::ExpectedSign(pos) = err {
                        return pos == 10;
                    }
                    false
                })
        );
        Ok(())
    }

    #[test]
    fn test_error_unexpected_cr() -> Result<()> {
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
        Ok(())
    }

    #[test]
    fn test_error_integer_overflow() -> Result<()> {
        let int = ":11111111111111111111111\r\n";
        assert!(
            de::from_str::<RESPType>(int)
                .is_err_and(|err| err.kind() == ErrorKind::ParseIntError)
        );
        Ok(())
    }

    #[test]
    fn from_reader() -> Result<()> {
        let mut buf = b"+hello\r\n".as_slice();
        let resp_str: RESPType = de::from_reader(&mut buf)?;
        assert_eq!(resp_str, RESPType::SimpleString("hello".to_owned()));
        Ok(())
    }
}