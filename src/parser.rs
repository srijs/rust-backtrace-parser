#[derive(Parser)]
#[grammar = "grammar.pest"]
pub(crate) struct BacktraceParser;

const _GRAMMAR: &str = include_str!("grammar.pest");
