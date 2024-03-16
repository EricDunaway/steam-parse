use std::{collections::HashMap, str::from_utf8};
use winnow::{
    binary,
    combinator::{delimited, preceded, repeat, terminated, dispatch, fail, peek},
    error::ContextError,
    token::{ literal, take_until, any},
    PResult, Parser,
};

#[derive(PartialEq, Eq, Debug)]
pub enum MapValue {
    Number(u32),
    String(String),
    Object(HashMap<String, MapValue>),
}

#[derive(PartialEq, Eq, Debug)]
pub struct VdfFile {
    entries: HashMap<String, MapValue>,
}

pub fn parse_string<'i>(input: &mut &'i [u8]) -> PResult<String, ContextError<&'i str>> {
    terminated(take_until(0.., 0x00), 0x00)
        .context("parseString Before Map")
        .map(|bytes| from_utf8(bytes).unwrap().to_string())
        .context("parseSring")
        .parse_next(input)
}

pub fn parse_integer_entity<'i>(
    input: &mut &'i [u8],
) -> PResult<(String, MapValue), ContextError<&'i str>> {
    (
        preceded(b"\x02", parse_string),
        binary::le_u32.map(MapValue::Number),
    )
        .context("parse_integer_entity")
        .parse_next(input)
}

pub fn parse_string_entity<'i>(
    input: &mut &'i [u8],
) -> PResult<(String, MapValue), ContextError<&'i str>> {
    (
        preceded(b"\x01", parse_string),
        parse_string.map(MapValue::String),
    )
        .context("parse_string_entity")
        .parse_next(input)
}

pub fn parse_hash_entity<'i>(
    input: &mut &'i [u8],
) -> PResult<(String, MapValue), ContextError<&'i str>> {
    delimited(
        literal([0x00]).context("start parse_hash_entity"),
        (
            parse_string.context("parse map name"),
            repeat(0.., 
                parse_map_value)
                .map(|x: Vec<(String, MapValue)>| 
                MapValue::Object(x.into_iter().collect())),
        ).context("parse hash cotents"),
        b"\x08",
    )
    .context("parse_hash_entity")
    .parse_next(input)
}


pub fn parse_map_value<'i>(
    input: &mut &'i [u8],
) -> PResult<(String, MapValue), ContextError<&'i str>> {
    dispatch!(peek(any);
        0x01 => parse_string_entity,
        0x02 => parse_integer_entity,
        0x00 => parse_hash_entity,
        _ => fail,
    ).parse_next(input)
}


#[cfg(test)]
mod tests {
    use crate::{parse_hash_entity, parse_integer_entity, parse_string_entity, MapValue};
    use std::collections::HashMap;

    #[test]
    fn test_parse_string_entity() {
        let input = &mut &b"\x01exe\0\"C:\\Program Files (x86)\\Games\\Game.exe\"\0"[..];
        let expected = (
            "exe".to_string(),
            MapValue::String("\"C:\\Program Files (x86)\\Games\\Game.exe\"".to_string()),
        );
        match parse_string_entity(input) {
            Ok(actual) => assert_eq!(actual, expected),
            Err(err) => panic!("{}", err),
        }
    }

    #[test]
    fn parse_int_kv() {
        let input = &mut b"\x02appid\x00Jz\x86\xF4".as_slice();
        match parse_integer_entity(input) {
            Ok(actual) => assert_eq!(actual, ("appid".to_string(), MapValue::Number(4102453834))),
            Err(err) => panic!("{}", err),
        }
    }

    
    #[test]
    fn parse_hash_entity_test() {
        let input= &mut &b"\x00shortcuts\x00\x02appid\x00\x51\x2D\xEB\x82\x01exe\0\"C:\\Program Files (x86)\\Games\\Game.exe\"\0\x08"[..];

        let mut expected: HashMap<String, MapValue> = HashMap::new();
        expected.insert("appid".to_string(), MapValue::Number(2196450641));
        expected.insert(
            "exe".to_string(),
            MapValue::String("\"C:\\Program Files (x86)\\Games\\Game.exe\"".to_string()),
        );

        match parse_hash_entity(input) {
            Ok(actual) => assert_eq!(
                actual,
                ("shortcuts".to_string(), MapValue::Object(expected))
            ),
            Err(err) => panic!("{}", err),
        }
    }
}
