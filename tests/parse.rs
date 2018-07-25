extern crate backtrace_parser;

use std::path::Path;

use backtrace_parser::parse;

#[test]
fn unresolved_symbols() {
    let data = include_str!("fixtures/unresolved.txt");
    let parsed = parse(data).unwrap();

    assert_eq!(parsed.frames().len(), 1);
    assert_eq!(parsed.frames()[0].symbols().len(), 0);
}

#[test]
fn no_symbol_info() {
    let data = include_str!("fixtures/no-info.txt");
    let parsed = parse(data).unwrap();

    assert_eq!(parsed.frames().len(), 1);
    assert_eq!(parsed.frames()[0].symbols().len(), 0);
}

#[test]
fn full_backtrace() {
    let data = include_str!("fixtures/full.txt");
    let parsed = parse(data).unwrap();

    assert_eq!(parsed.frames().len(), 13);

    assert_eq!(parsed.frames()[0].symbols().len(), 2);
    assert_eq!(
        parsed.frames()[0].symbols()[0].name(),
        Some("backtrace::backtrace::libunwind::trace::h042fc201d46ac6bb")
    );
    assert_eq!(parsed.frames()[0].symbols()[0].filename(), Some(Path::new("/root/.cargo/registry/src/github.com-1ecc6299db9ec823/backtrace-0.3.9/src/backtrace/libunwind.rs")));
    assert_eq!(parsed.frames()[0].symbols()[0].lineno(), Some(53));
    assert_eq!(
        parsed.frames()[0].symbols()[1].name(),
        Some("backtrace::backtrace::trace::hd8156e10e3d1f9ca")
    );
    assert_eq!(parsed.frames()[0].symbols()[1].filename(), Some(Path::new("/root/.cargo/registry/src/github.com-1ecc6299db9ec823/backtrace-0.3.9/src/backtrace/mod.rs")));
    assert_eq!(parsed.frames()[0].symbols()[1].lineno(), Some(42));

    assert_eq!(parsed.frames()[11].symbols()[0].name(), Some("_start"));

    assert_eq!(parsed.frames()[12].symbols()[0].name(), None);
}
