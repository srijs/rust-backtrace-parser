#[macro_use]
extern crate combine;

use std::path::Path;

mod parse;
pub use self::parse::ParseError;

pub fn parse<'a>(input: &'a str) -> Result<ParsedBacktrace<'a>, ParseError<'a>> {
    parse::parse(input)
}

#[derive(Debug)]
pub struct ParsedBacktrace<'a> {
    frames: Vec<ParsedFrame<'a>>,
}

impl<'a> ParsedBacktrace<'a> {
    pub fn frames(&self) -> &[ParsedFrame<'a>] {
        &self.frames
    }
}

#[derive(Debug)]
pub struct ParsedFrame<'a> {
    symbols: Vec<ParsedSymbol<'a>>,
}

impl<'a> ParsedFrame<'a> {
    pub fn symbols(&self) -> &[ParsedSymbol<'a>] {
        &self.symbols
    }
}

#[derive(Debug)]
pub struct ParsedSymbol<'a> {
    name: Option<&'a str>,
    filename: Option<&'a Path>,
    lineno: Option<u32>,
}

impl<'a> ParsedSymbol<'a> {
    pub fn name(&self) -> Option<&'a str> {
        self.name
    }

    pub fn filename(&self) -> Option<&'a Path> {
        self.filename
    }

    pub fn lineno(&self) -> Option<u32> {
        self.lineno
    }
}
