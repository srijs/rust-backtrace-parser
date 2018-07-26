//! This crate implements a parser for backtraces.
//!
//! The aim is to parse backtraces in the standard format
//! that any Rust program can generate, for instance when
//! crashing due to a panic, by creating a `failure::Error`,
//! or by using the [`backtrace`][1] crate directly.
//!
//! The parser follows a zero-copy approach, which means that
//! the input string can be provided by reference, and will not
//! be copied during parsing. This has the effect that parsing
//! a captured backtrace tends to be very performant.
//!
//! [1]: https://crates.io/crates/backtrace
//!
//! ## Example
//!
//! ```rust
//! use backtrace_parser::Backtrace;
//!
//! # let input = "stack backtrace: 0: 0x0 - <no info>";
//! let backtrace = Backtrace::parse(input).unwrap();
//!
//! for frame in backtrace.frames() {
//!     for symbol in frame.symbols() {
//!         println!("symbol: {:?}", symbol);
//!     }
//! }
//! ```
//!

#![deny(warnings)]
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]

extern crate pest;
#[macro_use]
extern crate pest_derive;

use std::error;
use std::fmt;
use std::path::Path;

use pest::Parser;

mod parser;
use self::parser::{BacktraceParser, Rule};

#[derive(Debug)]
/// Represents a parser error.
pub struct Error<'a> {
    inner: pest::Error<'a, Rule>,
}

impl<'a> fmt::Display for Error<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.inner, f)
    }
}

impl<'a> error::Error for Error<'a> {}

#[derive(Debug)]
/// Represents a parsed backtrace.
pub struct Backtrace<'a> {
    pairs: pest::iterators::Pairs<'a, Rule>,
}

impl<'a> Backtrace<'a> {
    /// Parse the provided input string and return either a parsed backtrace,
    /// or a parse error.
    pub fn parse(input: &'a str) -> Result<Backtrace<'a>, Error<'a>> {
        let pairs =
            BacktraceParser::parse(Rule::backtrace, input).map_err(|err| Error { inner: err })?;

        Ok(Backtrace { pairs })
    }

    /// Create an iterator over the stack frames in this backtrace.
    pub fn frames(&self) -> Frames<'a> {
        Frames {
            inner: self.pairs.clone(),
        }
    }
}

#[derive(Debug)]
/// Iterator over the stack frames in a parsed backtrace.
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
/// Represents a parsed stack frame.
pub struct Frame<'a> {
    pairs: pest::iterators::Pairs<'a, Rule>,
}

impl<'a> Frame<'a> {
    /// Create an iterator over the symbols in this stack frame.
    pub fn symbols(&self) -> Symbols<'a> {
        Symbols {
            inner: self.pairs.clone(),
        }
    }
}

#[derive(Debug)]
/// Iterator over the symbols in a parsed stack frame.
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
                            symbol_location_lineno.into_span().as_str().parse().ok();
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
/// Represents a parsed symbol.
pub struct Symbol<'a> {
    name: Option<&'a str>,
    filename: Option<&'a Path>,
    lineno: Option<u32>,
}

impl<'a> Symbol<'a> {
    /// Return the name of the symbol, if resolved.
    pub fn name(&self) -> Option<&'a str> {
        self.name
    }

    /// Return the path of the source file, if known.
    pub fn filename(&self) -> Option<&'a Path> {
        self.filename
    }

    /// Return the line number in source file, if known.
    pub fn lineno(&self) -> Option<u32> {
        self.lineno
    }
}
