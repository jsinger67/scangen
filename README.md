# About `scangen`

## WIP

This create is still in an experimental state and should currently not be used by others.

I experiment with the possibilities of scanner/lexer generation using the classical approaches
> Regular expression => NFA => DFA => minimized DFA => Rust code of a scanner

The library uses the `regex-syntax` crate to parse the regular expressions that should be processed.
The resulting ASTs are later processed and finally a scanner source is created.

## Why

In some cases when complicated regular expressions are used the compile time of these regular
expressions can slow down the start time of an application in an undesired way.

So a **compile ahead approach** could be appealing. Unfortunately compiled regular expressions from
the regex crate or the regex-automata crate are huge and can't be easily created as source files.

This crate should fill eventually this gap.

## The approach

The approach taken is to generate a source file that can be used as a module in another crate.
The input for the generation is a slice of patterns where each represent a single token. The
pattern index the scanner returns in the match corresponds to the index in the pattern slice.
The patterns in the given slice should be ordered by precedence, i.e. patterns with lower index
have higher precedence if the match yields multiple results with the same length. This is pretty
much the behavior of Lex/Flex.

Internally the library converts each pattern in a NFA. The NFA is later converted into a DFA which
itself is minimized afterwards. Each character or character class is treated as a character class
eventually and they are shared over all DFAs in the resulting Regex (multi DFA). For each character
class a match function is generated. This approach frees the library from the necessity to include
unicode tables and nevertheless providing basic unicode support.

## Guard rails

* The generated scanners are character oriented, i.e. no `u8` support is intended. Patterns are
`&[&str]` and the input is `&str`.
* The generated scanner uses the `scangen` crate as a reference, so this dependency has to be added.

    Maybe we decide to provide a separate `runtime` crate, or we introduce adequate crate features.


## What currently is not implemented

We have **no anchored matches**, i.e. ^, $, \b, \B, \A, \z and so on, are not available. Mostly,
this can be tolerated because of the overall properties of the scanner. Also the fact that the
longest match will win mitigates the need for such anchors.

Also we currently **do not support flags** (i, m, s, R, U, u, x), like in ```r"(?i)a+(?-i)b+"```.
We need to evaluate if this is a problem, but a the moment we belief that this is tolerable.
