use std::error::Error;
use std::fmt;

use combine::error::StreamError;
use combine::parser::char::{char, digit, hex_digit, newline, spaces, string};
use combine::parser::combinator::try;
use combine::parser::range::{recognize, take_until_range, take_while1};
use combine::parser::repeat::{many, many1};
use combine::stream::easy;
use combine::{self, eof, optional, position, sep_by1, Parser};

use super::*;

#[derive(Debug)]
pub struct ParseError<'a> {
    inner: easy::ParseError<&'a str>,
}

impl<'a> fmt::Display for ParseError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.inner, f)
    }
}

impl<'a> Error for ParseError<'a> {}

parser!{
    fn frame_index['a, I]()(I) -> u64
    where [
        I: combine::RangeStream<Item = char, Range = &'a str> + combine::Positioned,
        I::Error: combine::ParseError<char, &'a str, I::Position>,
        <I::Error as combine::ParseError<I::Item, &'a str, I::Position>>::StreamError:
            From<::std::num::ParseIntError>,
    ]
    {
        let digits = take_while1(|c: char| c.is_digit(10)).skip(char(':'));
        digits.and_then(|s: &str| s.parse())
    }
}

parser!{
    fn frame_pointer['a, I]()(I) -> u64
    where [
        I: combine::RangeStream<Item = char, Range = &'a str> + combine::Positioned,
        I::Error: combine::ParseError<char, &'a str, I::Position>,
        <I::Error as combine::ParseError<I::Item, &'a str, I::Position>>::StreamError:
            From<::std::num::ParseIntError>,
    ]
    {
        let digits = char('0').with(char('x')).with(take_while1(|c: char| c.is_ascii_hexdigit()));
        digits.and_then(|s: &str| u64::from_str_radix(s, 16))
    }
}

parser!{
    fn symbol_name['a, I]()(I) -> Option<&'a str>
    where [
        I: combine::RangeStream<Item = char, Range = &'a str> + combine::Positioned,
        I::Error: combine::ParseError<char, &'a str, I::Position>,
        <I::Error as combine::ParseError<I::Item, &'a str, I::Position>>::StreamError:
            From<::std::num::ParseIntError>,
    ]
    {
        choice!{
            try(string("<unknown>").map(|_| None)),
            take_until_range("\n").map(|s| Some(s))
        }
    }
}

parser!{
    fn symbol_location['a, I]()(I) -> (&'a Path, u32)
    where [
        I: combine::RangeStream<Item = char, Range = &'a str> + combine::Positioned,
        I::Error: combine::ParseError<char, &'a str, I::Position>,
        <I::Error as combine::ParseError<I::Item, &'a str, I::Position>>::StreamError:
            From<::std::num::ParseIntError>,
    ]
    {
        let digits = take_while1(|c: char| c.is_digit(10));
        let lineno = digits.and_then(|s: &str| s.parse());
        let filename = take_until_range(":").map(|s| Path::new(s));
        filename.skip(char(':')).and(lineno)
    }
}

parser!{
    fn frame_symbols['a, I]()(I) -> Vec<ParsedSymbol<'a>>
    where [
        I: combine::RangeStream<Item = char, Range = &'a str> + combine::Positioned,
        I::Error: combine::ParseError<char, &'a str, I::Position>,
        <I::Error as combine::ParseError<I::Item, &'a str, I::Position>>::StreamError:
            From<::std::num::ParseIntError>,
    ]
    {
        let parse_unresolved_symbols = char('-')
            .skip(spaces())
            .with(string("<unresolved>"))
            .map(|_| vec![]);

        let parse_empty_symbols = char('-')
            .skip(spaces())
            .with(string("<no info>"))
            .map(|_| vec![]);

        let parse_symbol_location = string("at")
            .skip(spaces())
            .with(symbol_location());

        let parse_symbol = char('-')
            .skip(spaces())
            .with(symbol_name())
            .and(optional(try(spaces().with(parse_symbol_location))))
            .map(|(symbol_name, symbol_location)| ParsedSymbol {
                name: symbol_name,
                lineno: symbol_location.as_ref().map(|(_, lineno)| *lineno),
                filename: symbol_location.map(|(filename, _)| filename),
            });

        choice!{
            try(parse_unresolved_symbols),
            try(parse_empty_symbols),
            many1(try(optional(try(newline())).skip(spaces()).with(parse_symbol)))
        }
    }
}

parser!{
    fn frame['a, I]()(I) -> ParsedFrame<'a>
    where [
        I: combine::RangeStream<Item = char, Range = &'a str> + combine::Positioned,
        I::Error: combine::ParseError<char, &'a str, I::Position>,
        <I::Error as combine::ParseError<I::Item, &'a str, I::Position>>::StreamError:
            From<::std::num::ParseIntError>,
    ]
    {
        frame_index()
            .skip(spaces())
            .and(frame_pointer())
            .skip(spaces())
            .and(frame_symbols())
            .map(|((_, _), symbols)| ParsedFrame { symbols })
    }
}

parser!{
    fn backtrace['a, I]()(I) -> ParsedBacktrace<'a>
    where [
        I: combine::RangeStream<Item = char, Range = &'a str> + combine::Positioned,
        I::Error: combine::ParseError<char, &'a str, I::Position>,
        <I::Error as combine::ParseError<I::Item, &'a str, I::Position>>::StreamError:
            From<::std::num::ParseIntError>,
    ]
    {
        string("stack backtrace:")
            .with(many1(try(spaces().with(frame()))))
            .map(|frames| ParsedBacktrace { frames })
    }
}

pub(crate) fn parse(input: &str) -> Result<ParsedBacktrace, ParseError> {
    let result = backtrace().skip(spaces()).skip(eof()).easy_parse(input);

    match result {
        Ok((trace, _)) => Ok(trace),
        Err(err) => Err(ParseError { inner: err }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_frame_idx() {
        let (idx, rest) = frame_index().easy_parse("1:").unwrap();
        assert_eq!(idx, 1);
        assert_eq!(rest, "");
    }

    #[test]
    fn parse_frame_ptr_null() {
        let (ptr, rest) = frame_pointer().easy_parse("0x0").unwrap();
        assert_eq!(ptr, 0);
        assert_eq!(rest, "");
    }

    #[test]
    fn parse_frame_ptr_long() {
        let (ptr, rest) = frame_pointer().easy_parse("0x55e06f94d05d").unwrap();
        assert_eq!(ptr, 94422433058909);
        assert_eq!(rest, "");
    }

    #[test]
    fn parse_symbol_name_unknown() {
        let (name, rest) = symbol_name().easy_parse("<unknown>").unwrap();
        assert_eq!(name, None);
        assert_eq!(rest, "");
    }

    #[test]
    fn parse_symbol_name_main_and_newline() {
        let (name, rest) = symbol_name().easy_parse("main\n").unwrap();
        assert_eq!(name, Some("main"));
        assert_eq!(rest, "\n");
    }

    #[test]
    fn parse_symbol_location_short() {
        let ((path, line), rest) = symbol_location().easy_parse("src/main.rs:6").unwrap();
        assert_eq!(path, Path::new("src/main.rs"));
        assert_eq!(line, 6);
        assert_eq!(rest, "");
    }

    #[test]
    fn parse_symbol_location_long() {
        let ((path, line), rest) = symbol_location().easy_parse("/root/.cargo/registry/src/github.com-1ecc6299db9ec823/backtrace-0.3.9/src/capture.rs:63").unwrap();
        assert_eq!(
            path,
            Path::new("/root/.cargo/registry/src/github.com-1ecc6299db9ec823/backtrace-0.3.9/src/capture.rs")
        );
        assert_eq!(line, 63);
        assert_eq!(rest, "");
    }

    #[test]
    fn parse_frame_no_info() {
        let (frame, rest) = frame().easy_parse("0: 0x0 - <no info>\n").unwrap();
        assert_eq!(frame.symbols().len(), 0);
        assert_eq!(rest, "\n");
    }

    #[test]
    fn parse_backtrace_single_frame() {
        let (trace, rest) = backtrace()
            .easy_parse("stack backtrace:\n  0: 0x0 - <no info>\n")
            .unwrap();
        assert_eq!(trace.frames().len(), 1);
        assert_eq!(trace.frames()[0].symbols().len(), 0);
        assert_eq!(rest, "\n");
    }

    #[test]
    fn parse_backtrace_two_frames() {
        let (trace, rest) = backtrace()
            .easy_parse("stack backtrace:\n  0: 0x1234 - main\n  1: 0x0 - <no info>\n")
            .unwrap();
        assert_eq!(trace.frames().len(), 2);
        assert_eq!(trace.frames()[0].symbols().len(), 1);
        assert_eq!(trace.frames()[0].symbols()[0].name(), Some("main"));
        assert_eq!(trace.frames()[1].symbols().len(), 0);
        assert_eq!(rest, "\n");
    }
}
