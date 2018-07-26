extern crate pest;
#[macro_use]
extern crate pest_derive;

use std::error::Error;
use std::fmt;
use std::path::Path;

use pest::Parser;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct BacktraceParser;

const _GRAMMAR: &str = include_str!("grammar.pest");

#[derive(Debug)]
pub struct ParseError<'a> {
    inner: pest::Error<'a, Rule>,
}

impl<'a> fmt::Display for ParseError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.inner, f)
    }
}

impl<'a> Error for ParseError<'a> {}

#[derive(Debug)]
pub struct Backtrace<'a> {
    pairs: pest::iterators::Pairs<'a, Rule>,
}

impl<'a> Backtrace<'a> {
    pub fn parse(input: &'a str) -> Result<Backtrace<'a>, ParseError<'a>> {
        let pairs = BacktraceParser::parse(Rule::backtrace, input)
            .map_err(|err| ParseError { inner: err })?;

        Ok(Backtrace { pairs })
    }
}

impl<'a> Backtrace<'a> {
    pub fn into_frames(self) -> Frames<'a> {
        Frames { inner: self.pairs }
    }
}

pub struct Frames<'a> {
    inner: pest::iterators::Pairs<'a, Rule>,
}

impl<'a> Iterator for Frames<'a> {
    type Item = Frame<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(frame) = self.inner.next() {
            debug_assert!(frame.as_rule() == Rule::frame);
            let mut frame_inner = frame.into_inner();

            let frame_index = frame_inner.next().unwrap();
            debug_assert!(frame_index.as_rule() == Rule::frame_index);
            let frame_pointer = frame_inner.next().unwrap();
            debug_assert!(frame_pointer.as_rule() == Rule::frame_pointer);

            Some(Frame { pairs: frame_inner })
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct Frame<'a> {
    pairs: pest::iterators::Pairs<'a, Rule>,
}

impl<'a> Frame<'a> {
    pub fn into_symbols(self) -> Symbols<'a> {
        Symbols { inner: self.pairs }
    }
}

pub struct Symbols<'a> {
    inner: pest::iterators::Pairs<'a, Rule>,
}

impl<'a> Iterator for Symbols<'a> {
    type Item = Symbol<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(symbol) = self.inner.next() {
            match symbol.as_rule() {
                Rule::symbol_non_empty => {
                    let mut parsed_symbol = Symbol {
                        name: None,
                        filename: None,
                        lineno: None,
                    };
                    let mut symbol_inner = symbol.into_inner();
                    let symbol_name = symbol_inner.next().unwrap();
                    match symbol_name.as_rule() {
                        Rule::symbol_name_known => {
                            parsed_symbol.name = Some(symbol_name.into_span().as_str())
                        }
                        _ => {}
                    }
                    if let Some(symbol_location) = symbol_inner.next() {
                        debug_assert!(symbol_location.as_rule() == Rule::symbol_location);
                        let mut symbol_location_inner = symbol_location.into_inner();
                        let symbol_location_path = symbol_location_inner.next().unwrap();
                        debug_assert!(symbol_location_path.as_rule() == Rule::symbol_location_path);
                        parsed_symbol.filename =
                            Some(Path::new(symbol_location_path.into_span().as_str()));
                        let symbol_location_lineno = symbol_location_inner.next().unwrap();
                        debug_assert!(
                            symbol_location_lineno.as_rule() == Rule::symbol_location_lineno
                        );
                        parsed_symbol.lineno =
                            Some(symbol_location_lineno.into_span().as_str().parse().unwrap());
                    }
                    Some(parsed_symbol)
                }
                _ => None,
            }
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct Symbol<'a> {
    name: Option<&'a str>,
    filename: Option<&'a Path>,
    lineno: Option<u32>,
}

impl<'a> Symbol<'a> {
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
