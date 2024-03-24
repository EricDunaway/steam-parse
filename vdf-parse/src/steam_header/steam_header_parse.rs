use crate::steam_header::{SteamHeader, SteamHeaderBuilder};
use winnow::combinator::{fail, peek, trace};
use winnow::prelude::*;
use winnow::error::ContextError;
use winnow::binary;
use winnow::combinator::dispatch;
use crate::appinfo::{APPINFO_MAGIC, APPINFO_MAGIC_2022};


pub fn parse_steam_header<'i>(
    input: &mut &'i [u8],
) -> PResult<SteamHeader, ContextError<&'i str>> {
    trace("parse steam header",
    (
        binary::le_u32,
        binary::le_u32,
    )
        .map(|(magic, version)| SteamHeaderBuilder::new().magic(magic).version(version).build())
        .context("parse steam header failed")
    ).parse_next(input)
}


pub fn parse_vdf_file<'i>(
    input: &mut &'i [u8],
) -> PResult<SteamHeader, ContextError<&'i str>> {
    trace("start vdf parse",
dispatch!(
    peek(binary::le_u32);
    APPINFO_MAGIC => parse_steam_header,
    APPINFO_MAGIC_2022 => parse_steam_header,
    _ => fail.context("parse_vdf_file no matching magic value found")
    
)
    .context("parse_vdf_file")
    ).parse_next(input)
}