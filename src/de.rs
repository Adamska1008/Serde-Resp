use serde::Deserialize;
use crate::{Result, Error};

struct Deserializer<'de> {
    input: &'de str,
    offset: u64
}

impl<'de> Deserializer<'de> {
    pub fn from_str(input: &'de str) -> Self {
        Deserializer { input, offset: 0 }
    }
}

pub fn from_str<'a, T>(s: &'a str) -> Result<T>
where T: Deserialize<'a>
{
    let mut deserializer = Deserializer::from_str(s);
    let t = T::deserialize(&mut deserializer)?;
    if deserializer.input.is_empty() {
        Ok(t)
    } else {
        Err(Error::TrailingCharacters)
    }
}

impl<'de> Deserializer<'de> {
    fn peek_char(&mut self) -> Result<char> {
        self.input.chars().next().ok_or(Error::Eof)
    }

    fn next_char(&mut self) -> Result<char> {
        let ch = self.peek_char()?;
        self.input = &self.input[ch.len_utf8()..];
        Ok(ch)
    }

    fn parse_int(&mut self) -> Result<i64> {

    }
}