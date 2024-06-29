# About `scangen`

## WIP

This create is highly experimental and in no means useful except for myself, at least in this very
early stage.
I experiment with the possibilities of scanner/lexer generation using the classical approaches
> Regular expression => NFA => DFA => minimized DFA [=> compilable (Rust) code]

The library uses the regex-syntax crate to parse the regular expressions. The AST is later
transformed into internal data structures.