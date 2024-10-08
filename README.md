# About `scangen`

## Not developed anymore

This create was an experimentation with the possibilities of scanner/lexer generation using the
classical approaches
> Regular expression => NFA => DFA => minimized DFA => Rust code of a scanner

All relevant results have been taken over to the [scnr](https://github.com/jsinger67/scnr.git)
crate which actually supersedes this crate.

The library used the `regex-syntax` crate to parse the regular expressions that should be processed.
The resulting ASTs are later processed and finally a scanner source is created.

## Why

In some cases when complicated regular expressions are used the compile time of these regular
expressions can slow down the start time of an application in an undesired way.

So a **compile ahead approach** could be appealing. Unfortunately compiled regular expressions from
the regex crate or the regex-automata crate are huge and can't be easily created as source files.

This crate should fill eventually this gap.

## The approach

The approach taken is to generate a source file that can be used as a module in another crate.
The input for the generation is a slice of pattern where each represent a single token. The
pattern index the scanner returns in the match corresponds to the index in the pattern slice.
The pattern in the given slice should be ordered by precedence, i.e. pattern with lower index
have higher precedence if the match yields multiple results with the same length. This is pretty
much the behavior of Lex/Flex.

Internally the library converts each pattern in a NFA. The NFA is later converted into a DFA which
itself is minimized afterwards. Each character or character class is treated as a character class
eventually and they are shared over all DFAs in the resulting Scanner (multi DFA). For each
character class a match function is generated. This approach frees the library from the necessity to
include unicode tables and nevertheless providing basic unicode support.

Also, *multiple scanner modes* should be supported out of the box. They are known from Lex/Flex as
[Start conditions](https://www.cs.princeton.edu/~appel/modern/c/software/flex/flex.html#SEC11).

## Guard rails

* The generated scanners are character oriented, i.e. no `u8` support is intended. Pattern are
`&[&str]` and the input is `&str`.
* The generated scanner uses the `scangen` crate as a reference, so this dependency has to be added.
Use the feature `runtime` when referencing this crate in the generated scanner.

## Create features

The crate has two features:
- `generate`: This feature enables the `compiletime` module which can be used to generate code
from a regex syntax.
- `runtime`: This feature enables the `runtime` module which can be used to scan text for matches.

## What currently is not implemented

We have **no anchored matches**, i.e. ^, $, \b, \B, \A, \z and so on, are not available. Mostly,
this can be tolerated because of the overall properties of the scanner. Also the fact that the
longest match will win mitigates the need for such anchors.

Also we currently **do not support flags** (i, m, s, R, U, u, x), like in ```r"(?i)a+(?-i)b+"```.
We need to evaluate if this is a problem, but a the moment we belief that this is tolerable.

## What will perhaps never be implemented

We have no need for capture groups in the context of token matching, so we see no necessity to
implement this feature.

# Example
The following example shows how to generate code from a set of regexes and format the generated
code.

```rust
use scangen::{generate_code, try_format};
use std::fs;

const TERMINALS: &[&str] = &[
    /* 0 */ "\\r\\n|\\r|\\n",   // Newline
    /* 1 */ "[\\s--\\r\\n]+",   // Whitespace
    /* 2 */ "(//.*(\\r\\n|\\r|\\n))",   // Line comment
    /* 3 */ "(/\\*.*?\\*/)",    // Block comment
    /* 4 */ r",",   // Comma
    /* 5 */ r"0|[1-9][0-9]*",   // Number
    /* 6 */ ".",    // Any character, i.e. error
];

let file_name = "data/scanner.rs";
{
    // Create the file where the generated code should be written to
    let mut out_file = fs::File::create(file_name.clone()).expect("Failed to create file");
    // Generate the code
    generate_code(TERMINALS, &[], None, &mut out_file).expect("Failed to generate code");
}

// Format the generated code
try_format(file_name).expect("Failed to format the generated code");
```

The generated scanner looks like this:

```rust
#![allow(clippy::manual_is_ascii_check)]

use scangen::{DfaData, FindMatches, Scanner, ScannerBuilder, ScannerModeData};

const DFAS: &[DfaData] = &[
    /* 0 */
    (
        "\\r\\n|\\r|\\n",
        &[1, 2],
        &[(0, 2), (0, 0), (2, 3)],
        &[(0, 2), (1, 1), (1, 1)],
    ),
    /* 1 */
    ("[\\s--\\r\\n]+", &[1], &[(0, 1), (1, 2)], &[(2, 1), (2, 1)]),
    /* 2 */
    (
        "(//.*(\\r\\n|\\r|\\n))",
        &[3, 4],
        &[(0, 1), (1, 2), (2, 5), (0, 0), (5, 6)],
        &[(3, 1), (3, 2), (4, 2), (0, 4), (1, 3), (1, 3)],
    ),
    /* 3 */
    (
        "(/\\*.*?\\*/)",
        &[4],
        &[(0, 1), (1, 2), (2, 3), (3, 5), (0, 0)],
        &[(3, 2), (3, 4), (5, 3), (5, 1), (4, 3)],
    ),
    /* 4 */
    (",", &[1], &[(0, 1), (0, 0)], &[(6, 1)]),
    /* 5 */
    (
        "0|[1-9][0-9]*",
        &[1, 2],
        &[(0, 2), (0, 0), (2, 3)],
        &[(7, 1), (8, 2), (9, 2)],
    ),
    /* 6 */
    (".", &[1], &[(0, 1), (0, 0)], &[(4, 1)]),
];

const MODES: &[ScannerModeData] = &[];

fn matches_char_class(c: char, char_class: usize) -> bool {
    match char_class {
        /* \r */
        0 => c == '\r',
        /* \n */
        1 => c == '\n',
        /* [\s--\r\n] */
        2 => c.is_whitespace() && !(c == '\r' || c == '\n'),
        /* / */
        3 => c == '/',
        /* . */
        4 => c != '\n' && c != '\r',
        /* \* */
        5 => c == '*',
        /* , */
        6 => c == ',',
        /* 0 */
        7 => c == '0',
        /* [1-9] */
        8 => ('1'..='9').contains(&c),
        /* [0-9] */
        9 => ('0'..='9').contains(&c),
        _ => false,
    }
}

pub(crate) fn create_scanner() -> Scanner {
    ScannerBuilder::new()
        .add_dfa_data(DFAS)
        .add_scanner_mode_data(MODES)
        .build()
}

pub(crate) fn create_find_iter<'h>(scanner: &Scanner, input: &'h str) -> FindMatches<'h> {
    scanner.find_iter(input, matches_char_class)
}
```

You can use the generated scanner in your code like this:
```rust
use crate::scanner::{create_find_iter, create_scanner};
mod scanner;

fn main() {
    if args().len() != 2 {
        eprintln!("Usage: {} filename", args().next().unwrap());
        std::process::exit(1);
    }
    let file_name = args().nth(1).unwrap();

    let input = std::fs::read_to_string(file_name.clone()).unwrap();
    let scanner = create_scanner();
    let find_iter = create_find_iter(&scanner, &input);

    let mut count = 0;
    for ma in find_iter {
        count += 1;
        println!("Match: {:?}: '{}'", ma, &input[ma.start()..ma.end()]);
    }
    println!("Found {} tokens", count);
}
```
