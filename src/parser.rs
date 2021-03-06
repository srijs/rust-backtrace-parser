#[derive(Parser)]
#[grammar = "grammar.pest"]
pub(crate) struct BacktraceParser;

const _GRAMMAR: &str = include_str!("grammar.pest");

#[cfg(test)]
mod tests {
    use pest::Parser;

    use super::*;

    #[test]
    fn parse_frame_idx() {
        let input = "1:";
        let mut pairs = BacktraceParser::parse(Rule::frame_index, input).unwrap();
        let pair = pairs.next().unwrap();
        assert_eq!(pair.as_rule(), Rule::frame_index);
        assert_eq!(pair.into_span().as_str(), "1:");
    }

    #[test]
    fn parse_frame_ptr_null() {
        let input = "0x0";
        let mut pairs = BacktraceParser::parse(Rule::frame_pointer, input).unwrap();
        let pair = pairs.next().unwrap();
        assert_eq!(pair.as_rule(), Rule::frame_pointer);
        assert_eq!(pair.into_span().as_str(), "0x0");
    }

    #[test]
    fn parse_frame_ptr_long() {
        let input = "0x55e06f94d05d";
        let mut pairs = BacktraceParser::parse(Rule::frame_pointer, input).unwrap();
        let pair = pairs.next().unwrap();
        assert_eq!(pair.as_rule(), Rule::frame_pointer);
        assert_eq!(pair.into_span().as_str(), "0x55e06f94d05d");
    }

    #[test]
    fn parse_symbol_name_unknown() {
        let input = "<unknown>";
        let mut pairs = BacktraceParser::parse(Rule::symbol_name, input).unwrap();
        let pair = pairs.next().unwrap();
        assert_eq!(pair.as_rule(), Rule::symbol_name_unknown);
        assert_eq!(pair.into_span().as_str(), "<unknown>");
    }

    #[test]
    fn parse_symbol_name_main_and_newline() {
        let input = "main\n";
        let mut pairs = BacktraceParser::parse(Rule::symbol_name, input).unwrap();
        let pair = pairs.next().unwrap();
        assert_eq!(pair.as_rule(), Rule::symbol_name_known);
        assert_eq!(pair.into_span().as_str(), "main");
    }

    #[test]
    fn parse_symbol_location_short() {
        let input = "src/main.rs:6";
        let mut pairs = BacktraceParser::parse(Rule::symbol_location, input).unwrap();
        let pair = pairs.next().unwrap();
        assert_eq!(pair.as_rule(), Rule::symbol_location);
        let mut inner_pairs = pair.into_inner();
        let pair1 = inner_pairs.next().unwrap();
        assert_eq!(pair1.as_rule(), Rule::symbol_location_path);
        assert_eq!(pair1.into_span().as_str(), "src/main.rs");
        let pair2 = inner_pairs.next().unwrap();
        assert_eq!(pair2.as_rule(), Rule::symbol_location_lineno);
        assert_eq!(pair2.into_span().as_str(), "6");
    }

    #[test]
    fn parse_symbol_location_long() {
        let input = "/root/.cargo/registry/src/github.com-1ecc6299db9ec823/backtrace-0.3.9/src/capture.rs:63";
        let mut pairs = BacktraceParser::parse(Rule::symbol_location, input).unwrap();
        let pair = pairs.next().unwrap();
        assert_eq!(pair.as_rule(), Rule::symbol_location);
        let mut inner_pairs = pair.into_inner();
        let pair1 = inner_pairs.next().unwrap();
        assert_eq!(pair1.as_rule(), Rule::symbol_location_path);
        assert_eq!(
            pair1.into_span().as_str(),
            "/root/.cargo/registry/src/github.com-1ecc6299db9ec823/backtrace-0.3.9/src/capture.rs"
        );
        let pair2 = inner_pairs.next().unwrap();
        assert_eq!(pair2.as_rule(), Rule::symbol_location_lineno);
        assert_eq!(pair2.into_span().as_str(), "63");
    }

    #[test]
    fn parse_symbols_no_info() {
        let input = "- <no info>\n";
        let mut pairs = BacktraceParser::parse(Rule::symbols, input).unwrap();
        let pair = pairs.next().unwrap();
        assert_eq!(pair.as_rule(), Rule::symbol_no_info);
        assert_eq!(pair.into_span().as_str(), "<no info>");
    }

    #[test]
    fn parse_symbols_unresolved() {
        let input = "- <unresolved>\n";
        let mut pairs = BacktraceParser::parse(Rule::symbols, input).unwrap();
        let pair = pairs.next().unwrap();
        assert_eq!(pair.as_rule(), Rule::symbol_unresolved);
        assert_eq!(pair.into_span().as_str(), "<unresolved>");
    }

    #[test]
    fn parse_symbols_non_empty() {
        let input = "- main\n  at src/main.rs:6\n- _start\n- <unknown>\n";
        let mut pairs = BacktraceParser::parse(Rule::symbols, input).unwrap();
        let pair1 = pairs.next().unwrap();
        assert_eq!(pair1.as_rule(), Rule::symbol_non_empty);
        assert_eq!(pair1.into_span().as_str(), "main\n  at src/main.rs:6");
        let pair2 = pairs.next().unwrap();
        assert_eq!(pair2.as_rule(), Rule::symbol_non_empty);
        assert_eq!(pair2.into_span().as_str(), "_start\n");
        let pair3 = pairs.next().unwrap();
        assert_eq!(pair3.as_rule(), Rule::symbol_non_empty);
        assert_eq!(pair3.into_span().as_str(), "<unknown>\n");
    }

    #[test]
    fn parse_frames_no_info() {
        let input = "0: 0x0 - <no info>\n";
        let mut pairs = BacktraceParser::parse(Rule::frames, input).unwrap();
        let pair1 = pairs.next().unwrap();
        assert_eq!(pair1.as_rule(), Rule::frame);
        let mut inner_pairs1 = pair1.into_inner();
        let inner_pair1 = inner_pairs1.next().unwrap();
        assert_eq!(inner_pair1.as_rule(), Rule::frame_index);
        let inner_pair2 = inner_pairs1.next().unwrap();
        assert_eq!(inner_pair2.as_rule(), Rule::frame_pointer);
        let inner_pair3 = inner_pairs1.next().unwrap();
        assert_eq!(inner_pair3.as_rule(), Rule::symbol_no_info);
    }

    #[test]
    fn parse_frames_non_empty() {
        let input = "0: 0x1234 - main\n  1: 0x0 - <no info>\n";
        let mut pairs = BacktraceParser::parse(Rule::frames, input).unwrap();
        let pair1 = pairs.next().unwrap();
        assert_eq!(pair1.as_rule(), Rule::frame);
        assert_eq!(pair1.into_span().as_str(), "0: 0x1234 - main\n  ");
        let pair2 = pairs.next().unwrap();
        assert_eq!(pair2.as_rule(), Rule::frame);
        assert_eq!(pair2.into_span().as_str(), "1: 0x0 - <no info>");
    }
}
