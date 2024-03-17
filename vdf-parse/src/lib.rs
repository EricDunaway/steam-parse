use std::{collections::HashMap, str::from_utf8};
use winnow::{
    binary,
    combinator::{delimited, preceded, repeat, terminated, dispatch, fail, peek, trace},
    error::ContextError,
    token::{ take_until, any},
    PResult, Parser,
};
use serde::{Serialize, Serializer, ser::SerializeMap, Deserialize};

#[derive(PartialEq, Debug, Deserialize, Clone)]
pub enum MapValue {
    Number32(u32),
    Number64(u64),
    NumberFloat(f32),
    String(String),
    Object(HashMap<String, MapValue>),
}

impl Serialize for MapValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            MapValue::Number32(num) => serializer.serialize_u32(num),
            MapValue::Number64(num) => serializer.serialize_u64(num),
            MapValue::NumberFloat(num) => serializer.serialize_f32(num),
            MapValue::String(ref s) => serializer.serialize_str(s),
            MapValue::Object(ref obj) => {
                // Here, use serialize_map instead of serialize_struct
                let mut map = serializer.serialize_map(Some(obj.len()))?;
                for (key, value) in obj {
                    map.serialize_entry(key, value)?;
                }
                map.end()
            },
        }
    }
}

#[derive(PartialEq, Debug)]
pub struct VdfFile {
    entries: HashMap<String, MapValue>,
}

pub fn parse_string<'i>(input: &mut &'i [u8]) -> PResult<String, ContextError<&'i str>> {
    trace("parseString",
    terminated(take_until(0.., 0x00), 0x00)
        .context("parseString Before Map")
        .map(|bytes| from_utf8(bytes).unwrap().to_string())
        .context("parseSring")
    ).parse_next(input)
}

pub fn parse_integer32_entity<'i>(
    input: &mut &'i [u8],
) -> PResult<(String, MapValue), ContextError<&'i str>> {
    trace("parse_integer32_entity",
    (
        preceded(b"\x02", parse_string),
        binary::le_u32.map(MapValue::Number32),
    )
        .context("parse_integer32_entity"))
        .parse_next(input)
}

pub fn parse_float_entity<'i>(
    input: &mut &'i [u8],
) -> PResult<(String, MapValue), ContextError<&'i str>> {
    trace("parse_float_entity",
    (
        preceded(b"\x03", parse_string),
        binary::le_f32.map(MapValue::NumberFloat),
    )
        .context("parse_float_entity"))
        .parse_next(input)
}

pub fn parse_integer64_entity<'i>(
    input: &mut &'i [u8],
) -> PResult<(String, MapValue), ContextError<&'i str>> {
    trace("parse_integer64_entity",
    (
        preceded(b"\x07", parse_string),
        binary::le_u64.map(MapValue::Number64),
    )
        .context("parse_integer64_entity"))
        .parse_next(input)
}

pub fn parse_string_entity<'i>(
    input: &mut &'i [u8],
) -> PResult<(String, MapValue), ContextError<&'i str>> {
    trace("parse_string_entity",
    (
        preceded(b"\x01", parse_string),
        parse_string.map(MapValue::String),
    )
        .context("parse_string_entity"))
        .parse_next(input)
}

pub fn parse_hash_entity<'i>(
    input: &mut &'i [u8],
) -> PResult<(String, MapValue), ContextError<&'i str>> {
    trace("parseHashEntity",
    delimited(
        b"\x00".context("start parse_hash_entity"),
        (
            parse_string.context("parse map name"),
            repeat(0.., 
                parse_map_value).context("parse hash cotents")
                .map(|x: Vec<(String, MapValue)>| 
                MapValue::Object(x.into_iter().collect())),
        ).context("convert hash cotents"),
        b"\x08".context("end parse_hash_entity"),
    )
    .context("parse_hash_entity"))
    .parse_next(input)
}


pub fn parse_map_value<'i>(
    input: &mut &'i [u8],
) -> PResult<(String, MapValue), ContextError<&'i str>> {
    trace("parseMapEntity",
    dispatch!(peek(any);
        0x01 => parse_string_entity,
        0x02 => parse_integer32_entity,
        0x03 => parse_float_entity,
        0x07 => parse_integer64_entity,
        0x00 => parse_hash_entity,
        _ => fail.context("parse_map_value error"),
    )).parse_next(input)
}


#[cfg(test)]
mod tests {
    use crate::{parse_hash_entity, parse_integer32_entity, parse_string_entity, MapValue};
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
        match parse_integer32_entity(input) {
            Ok(actual) => assert_eq!(actual, ("appid".to_string(), MapValue::Number32(4102453834))),
            Err(err) => panic!("{}", err),
        }
    }

    
    #[test]
    fn parse_hash_entity_test() {
        let input= &mut &b"\x00shortcuts\x00\x02appid\x00\x51\x2D\xEB\x82\x01exe\0\"C:\\Program Files (x86)\\Games\\Game.exe\"\0\x08"[..];

        let mut expected: HashMap<String, MapValue> = HashMap::new();
        expected.insert("appid".to_string(), MapValue::Number32(2196450641));
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
