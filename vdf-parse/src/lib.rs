use nom::{
    branch::alt,
    bytes::complete::{tag, take, take_until},
    combinator::{self, map_res},
    error::VerboseError,
    number,
    sequence::{self, preceded},
    IResult,
};
use std::{
    borrow::Cow,
    collections::HashMap,
    str::{self, from_utf8, Utf8Error},
};

#[derive(PartialEq, Eq, Debug)]
pub enum MapValue<'i> {
    Number(u32),
    String(Cow<'i, str>),
    Object(HashMap<Cow<'i, str>, MapValue<'i>>),
}

type IResutlCowStrMapValue<'a> = IResult<&'a [u8], (Cow<'a,str>, MapValue<'a>), VerboseError<&'a [u8]>>;

#[derive(PartialEq, Eq, Debug)]
pub struct VdfFile<'i> {
    entries: HashMap<Cow<'i, str>, MapValue<'i>>,
}

pub fn parse_string<'i>(input: &'i [u8]) -> IResult<&'i [u8], Cow<str>, VerboseError<&'i [u8]>> {
    nom::error::context(
        "parseString",
        map_res(
            sequence::terminated(take_until(&[0x00][..]), take(1usize)),
            |bytes| -> Result<Cow<'i, str>, Utf8Error> {
                return Ok(Cow::Borrowed(from_utf8(bytes)?));
            },
        ),
    )(input)
}

#[allow(clippy::needless_lifetimes)]
pub fn parse_integer_entity<'i>(
    input: &'i [u8],
) -> IResutlCowStrMapValue {
    nom::error::context(
        "parse_integer_entity",
        sequence::tuple((
            preceded(tag(b"\x02"), parse_string),
            combinator::map(number::complete::le_u32, MapValue::Number),
        )),
    )(input)
}

#[allow(clippy::needless_lifetimes)]
pub fn parse_string_entity<'i>(
    input: &'i [u8],
) -> IResutlCowStrMapValue {
    nom::error::context(
        "parse_string_entity",
        sequence::tuple((
            preceded(tag(b"\x01"), parse_string),
            combinator::map(parse_string, MapValue::String),
        )),
    )(input)
}

#[allow(clippy::needless_lifetimes)]
pub fn parse_hash_entity<'i>(
    input: &'i [u8],
) -> IResutlCowStrMapValue {
    nom::error::context(
        "parse_hash_entity",
        sequence::delimited(
            tag(b"\x00"),
            sequence::tuple((
                parse_string,
                combinator::map(nom::multi::many0(parse_map_value), |x| {
                    MapValue::Object(x.into_iter().collect())
                }),
            )),
            tag(b"\x08"),
        ),
    )(input)
}

#[allow(clippy::needless_lifetimes)]
pub fn parse_map_value<'i>(
    input: &'i [u8],
) -> IResutlCowStrMapValue {
    nom::error::context(
        "parse_map_entity",
        alt((parse_string_entity, parse_integer_entity, parse_hash_entity)),
    )(input)
}

#[cfg(test)]
mod tests {
    use std::{borrow::Cow, collections::HashMap};

    use crate::{parse_hash_entity, parse_integer_entity, parse_string_entity, MapValue};

    #[test]
    fn test_parse_string_entity() {
        let input = &b"\x01exe\0\"C:\\Program Files (x86)\\Games\\Game.exe\"\0"[..];
        let expected = (
            Cow::Borrowed("exe"),
            MapValue::String(Cow::from("\"C:\\Program Files (x86)\\Games\\Game.exe\"")),
        );
        match parse_string_entity(input) {
            Ok((_, actual)) => assert_eq!(actual, expected),
            Err(err) => panic!("{}", err),
        }
    }

    #[test]
    fn parse_int_kv() {
        let input: &[u8; 11] = &b"\x02appid\x00Jz\x86\xF4";
        match parse_integer_entity(input) {
            Ok((_, actual)) => assert_eq!(
                actual,
                (Cow::Borrowed("appid"), MapValue::Number(4102453834))
            ),
            Err(err) => panic!("{}", err),
        }
    }

    #[test]
    fn parse_hash_entity_test() {
        let input= b"\x00shortcuts\x00\x02appid\x00\x51\x2D\xEB\x82\x01exe\0\"C:\\Program Files (x86)\\Games\\Game.exe\"\0\x08";

        let mut expected: HashMap<Cow<str>, MapValue> = HashMap::new();
        expected.insert(Cow::Borrowed("appid"), MapValue::Number(2196450641));
        expected.insert(
            Cow::Borrowed("exe"),
            MapValue::String(Cow::from("\"C:\\Program Files (x86)\\Games\\Game.exe\"")),
        );

        match parse_hash_entity(input) {
            Ok((_, actual)) => assert_eq!(
                actual,
                (Cow::Borrowed("shortcuts"), MapValue::Object(expected))
            ),
            Err(err) => panic!("{}", err),
        }
    }
}
