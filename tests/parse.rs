extern crate backtrace_parser;

use std::path::Path;

use backtrace_parser::Backtrace;

#[test]
fn unresolved_symbols() {
    let data = include_str!("fixtures/unresolved.txt");
    let parsed = Backtrace::parse(data).unwrap();

    let mut frames = parsed.frames();

    let symbols = frames.next().unwrap().symbols().collect::<Vec<_>>();
    assert_eq!(symbols.len(), 0);

    assert!(frames.next().is_none());
}

#[test]
fn no_symbol_info() {
    let data = include_str!("fixtures/no-info.txt");
    let parsed = Backtrace::parse(data).unwrap();

    let mut frames = parsed.frames();

    let symbols = frames.next().unwrap().symbols().collect::<Vec<_>>();
    assert_eq!(symbols.len(), 0);

    assert!(frames.next().is_none());
}

#[test]
fn line_number_overflow() {
    let data = "stack backtrace: 0: 0x0 - main\nat src/main.rs:1208925819614629174706176";
    let parsed = Backtrace::parse(data).unwrap();

    let mut frames = parsed.frames();

    let symbols = frames.next().unwrap().symbols().collect::<Vec<_>>();
    assert_eq!(symbols.len(), 1);
    assert_eq!(symbols[0].name(), Some("main"));
    assert_eq!(symbols[0].filename(), Some(Path::new("src/main.rs")));
    assert_eq!(symbols[0].lineno(), None);

    assert!(frames.next().is_none());
}

#[test]
fn full_backtrace() {
    let data = include_str!("fixtures/full.txt");
    let parsed = Backtrace::parse(data).unwrap();

    let mut frames = parsed.frames();

    let symbols0 = frames.next().unwrap().symbols().collect::<Vec<_>>();
    assert_eq!(symbols0.len(), 2);
    assert_eq!(
        symbols0[0].name(),
        Some("backtrace::backtrace::libunwind::trace::h042fc201d46ac6bb")
    );
    assert_eq!(symbols0[0].filename(), Some(Path::new("/root/.cargo/registry/src/github.com-1ecc6299db9ec823/backtrace-0.3.9/src/backtrace/libunwind.rs")));
    assert_eq!(symbols0[0].lineno(), Some(53));
    assert_eq!(
        symbols0[1].name(),
        Some("backtrace::backtrace::trace::hd8156e10e3d1f9ca")
    );
    assert_eq!(symbols0[1].filename(), Some(Path::new("/root/.cargo/registry/src/github.com-1ecc6299db9ec823/backtrace-0.3.9/src/backtrace/mod.rs")));
    assert_eq!(symbols0[1].lineno(), Some(42));

    let symbols1 = frames.next().unwrap().symbols().collect::<Vec<_>>();
    assert_eq!(symbols1.len(), 1);

    let symbols2 = frames.next().unwrap().symbols().collect::<Vec<_>>();
    assert_eq!(symbols2.len(), 1);

    let symbols3 = frames.next().unwrap().symbols().collect::<Vec<_>>();
    assert_eq!(symbols3.len(), 1);

    let symbols4 = frames.next().unwrap().symbols().collect::<Vec<_>>();
    assert_eq!(symbols4.len(), 1);

    let symbols5 = frames.next().unwrap().symbols().collect::<Vec<_>>();
    assert_eq!(symbols5.len(), 2);

    let symbols6 = frames.next().unwrap().symbols().collect::<Vec<_>>();
    assert_eq!(symbols6.len(), 1);

    let symbols7 = frames.next().unwrap().symbols().collect::<Vec<_>>();
    assert_eq!(symbols7.len(), 3);

    let symbols8 = frames.next().unwrap().symbols().collect::<Vec<_>>();
    assert_eq!(symbols8.len(), 1);

    let symbols9 = frames.next().unwrap().symbols().collect::<Vec<_>>();
    assert_eq!(symbols9.len(), 1);

    let symbols10 = frames.next().unwrap().symbols().collect::<Vec<_>>();
    assert_eq!(symbols10.len(), 1);

    let symbols11 = frames.next().unwrap().symbols().collect::<Vec<_>>();
    assert_eq!(symbols11[0].name(), Some("_start"));

    let symbols12 = frames.next().unwrap().symbols().collect::<Vec<_>>();
    assert_eq!(symbols12[0].name(), None);

    assert!(frames.next().is_none());
}
