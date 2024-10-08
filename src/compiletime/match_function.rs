use regex_syntax::ast::{
    Ast, ClassAscii, ClassAsciiKind, ClassBracketed, ClassPerl, ClassPerlKind, ClassSet,
    ClassSetBinaryOp, ClassSetBinaryOpKind, ClassSetItem, ClassSetRange, ClassSetUnion,
    ClassUnicode,
    ClassUnicodeKind::{Named, NamedValue, OneLetter},
    Literal,
};

use crate::{Result, ScanGenError};

macro_rules! unsupported {
    ($feature:expr) => {
        ScanGenError::new($crate::ScanGenErrorKind::UnsupportedFeature(
            $feature.to_string(),
        ))
    };
}

/// A function that takes a character and returns a boolean.
pub(crate) struct MatchFunction(pub(crate) Box<dyn Fn(char) -> bool + 'static>);

impl MatchFunction {
    /// Create a new match function from a closure.
    pub(crate) fn new<F>(f: F) -> Self
    where
        F: Fn(char) -> bool + 'static,
    {
        MatchFunction(Box::new(f))
    }

    /// Call the match function with a character.
    #[inline]
    pub(crate) fn call(&self, c: char) -> bool {
        (self.0)(c)
    }

    fn try_from_class_set(set: ClassSet) -> Result<Self> {
        let negated = false;
        match &set {
            ClassSet::Item(item) => Self::try_from_set_item(item.clone(), negated),
            ClassSet::BinaryOp(bin_op) => Self::try_from_binary_op(bin_op.clone(), negated),
        }
    }

    fn try_from_class_unicode(unicode: ClassUnicode) -> Result<Self> {
        let negated = unicode.is_negated();
        let kind = unicode.kind.clone();
        let match_function = match kind {
            OneLetter(ch) => {
                match ch {
                    // Unicode class for Letters
                    'L' => MatchFunction::new(|ch| ch.is_alphabetic()),
                    // Unicode class for Numbers
                    'N' => MatchFunction::new(|ch| ch.is_numeric()),
                    // Unicode class for Whitespace
                    'Z' => MatchFunction::new(|ch| ch.is_whitespace()),
                    // Unicode class for Punctuation
                    // Attention: Only ASCII based punctuation is supported
                    'P' => MatchFunction::new(|ch| ch.is_ascii_punctuation()),
                    // Unicode class for Control characters
                    'C' => MatchFunction::new(|ch| ch.is_control()),
                    _ => return Err(unsupported!(format!("{:#?}", unicode))),
                }
            }
            Named(_) | NamedValue { .. } => {
                // Actually no support for named classes and named values
                // We need to ensure that this is not a match even if it is negated
                let no_match = negated;
                MatchFunction::new(move |_| no_match)
            }
        };
        Ok(if unicode.is_negated() {
            MatchFunction::new(move |ch| !match_function.call(ch))
        } else {
            match_function
        })
    }

    fn try_from_class_perl(perl: ClassPerl) -> Result<Self> {
        let ClassPerl { negated, kind, .. } = perl;
        let match_function = match kind {
            ClassPerlKind::Digit => MatchFunction::new(|ch| ch.is_numeric()),
            ClassPerlKind::Space => MatchFunction::new(|ch| ch.is_whitespace()),
            ClassPerlKind::Word => MatchFunction::new(|ch| ch.is_alphanumeric()),
        };
        Ok(if negated {
            MatchFunction::new(move |ch| !match_function.call(ch))
        } else {
            match_function
        })
    }

    fn try_from_class_bracketed(bracketed: ClassBracketed) -> Result<Self> {
        let negated = bracketed.negated;
        match &bracketed.kind {
            ClassSet::Item(item) => Self::try_from_set_item(item.clone(), negated),
            ClassSet::BinaryOp(bin_op) => Self::try_from_binary_op(bin_op.clone(), negated),
        }
    }

    // Match one of the set items, i.e.
    fn try_from_class_set_union(union: ClassSetUnion) -> Result<Self> {
        union
            .items
            .iter()
            .try_fold(MatchFunction::new(|_| false), |acc, s| {
                Self::try_from_set_item(s.clone(), false)
                    .map(|f| MatchFunction::new(move |ch| acc.call(ch) || f.call(ch)))
            })
    }

    fn try_from_binary_op(bin_op: ClassSetBinaryOp, negated: bool) -> Result<Self> {
        let ClassSetBinaryOp { kind, lhs, rhs, .. } = bin_op;
        let lhs = Self::try_from_class_set(*lhs)?;
        let rhs = Self::try_from_class_set(*rhs)?;
        let match_function = match kind {
            ClassSetBinaryOpKind::Intersection => {
                MatchFunction::new(move |ch| lhs.call(ch) && rhs.call(ch))
            }
            ClassSetBinaryOpKind::Difference => {
                MatchFunction::new(move |ch| lhs.call(ch) && !rhs.call(ch))
            }
            ClassSetBinaryOpKind::SymmetricDifference => {
                MatchFunction::new(move |ch| lhs.call(ch) != rhs.call(ch))
            }
        };
        Ok(if negated {
            MatchFunction::new(move |ch| !match_function.call(ch))
        } else {
            match_function
        })
    }

    fn try_from_set_item(item: ClassSetItem, negated: bool) -> Result<Self> {
        let match_function = match item {
            ClassSetItem::Empty(_) => MatchFunction::new(|_| false),
            ClassSetItem::Literal(ref l) => {
                let Literal { c, .. } = *l;
                MatchFunction::new(move |ch| ch == c)
            }
            ClassSetItem::Range(ref r) => {
                let ClassSetRange {
                    ref start, ref end, ..
                } = *r;
                let start = start.c;
                let end = end.c;
                MatchFunction::new(move |ch| start <= ch && ch <= end)
            }
            ClassSetItem::Ascii(ref a) => {
                let ClassAscii {
                    ref kind, negated, ..
                } = *a;
                let match_function = match kind {
                    ClassAsciiKind::Alnum => MatchFunction::new(|ch| ch.is_alphanumeric()),
                    ClassAsciiKind::Alpha => MatchFunction::new(|ch| ch.is_alphabetic()),
                    ClassAsciiKind::Ascii => MatchFunction::new(|ch| ch.is_ascii()),
                    ClassAsciiKind::Blank => MatchFunction::new(|ch| ch.is_ascii_whitespace()),
                    ClassAsciiKind::Cntrl => MatchFunction::new(|ch| ch.is_ascii_control()),
                    ClassAsciiKind::Digit => MatchFunction::new(|ch| ch.is_numeric()),
                    ClassAsciiKind::Graph => MatchFunction::new(|ch| ch.is_ascii_graphic()),
                    ClassAsciiKind::Lower => MatchFunction::new(|ch| ch.is_lowercase()),
                    ClassAsciiKind::Print => MatchFunction::new(|ch| ch.is_ascii_graphic()),
                    ClassAsciiKind::Punct => MatchFunction::new(|ch| ch.is_ascii_punctuation()),
                    ClassAsciiKind::Space => MatchFunction::new(|ch| ch.is_whitespace()),
                    ClassAsciiKind::Upper => MatchFunction::new(|ch| ch.is_uppercase()),
                    ClassAsciiKind::Word => MatchFunction::new(|ch| ch.is_alphanumeric()),
                    ClassAsciiKind::Xdigit => MatchFunction::new(|ch| ch.is_ascii_hexdigit()),
                };
                if negated {
                    MatchFunction::new(move |ch| !match_function.call(ch))
                } else {
                    match_function
                }
            }
            ClassSetItem::Unicode(ref c) => Self::try_from_class_unicode(c.clone())?,
            ClassSetItem::Perl(ref c) => Self::try_from_class_perl(c.clone())?,
            ClassSetItem::Bracketed(ref c) => Self::try_from_class_bracketed(*c.clone())?,
            ClassSetItem::Union(ref c) => Self::try_from_class_set_union(c.clone())?,
        };
        Ok(if negated {
            MatchFunction::new(move |ch| !match_function.call(ch))
        } else {
            match_function
        })
    }

    pub(crate) fn generate_code(
        ast: &Ast,
        match_function_index: usize,
        output: &mut dyn std::io::Write,
    ) -> Result<()> {
        // Add code generation here
        writeln!(output, "        /* {} */", ast)?;
        writeln!(output, "        {} => {{", match_function_index)?;
        match ast {
            Ast::Empty(_) => write!(output, "            true")?,
            Ast::Dot(_) => write!(output, "            c != '\\n' && c != '\\r'")?,
            Ast::Literal(ref l) => {
                let Literal { c, .. } = **l;
                write!(output, "            c == '{}'", c.escape_default())?
            }
            Ast::ClassUnicode(ref c) => {
                Self::generate_code_from_class_unicode(c, output)?;
            }
            Ast::ClassPerl(ref c) => {
                Self::generate_code_from_class_perl(c, output)?;
            }
            Ast::ClassBracketed(ref c) => {
                Self::generate_code_from_class_bracketed(c, output)?;
            }
            _ => return Err(unsupported!(format!("{:#?}", ast))),
        }
        writeln!(output)?;
        writeln!(output, "        }},")?;
        Ok(())
    }

    fn generate_code_from_class_set(set: &ClassSet, output: &mut dyn std::io::Write) -> Result<()> {
        let negated = false;
        match set {
            ClassSet::Item(item) => {
                Self::generate_code_from_set_item(item.clone(), negated, output)
            }
            ClassSet::BinaryOp(bin_op) => {
                Self::generate_code_from_binary_op(bin_op.clone(), negated, output)
            }
        }
    }

    fn generate_code_from_class_unicode(
        c: &ClassUnicode,
        output: &mut dyn std::io::Write,
    ) -> Result<()> {
        let ClassUnicode { negated, kind, .. } = c;
        write!(output, "            ")?;
        if *negated {
            write!(output, "!")?;
        }
        match kind {
            Named(_) | NamedValue { .. } => {
                // Actually no support for named classes and named values
                // We need to ensure that this is not a match even if it is negated
                let no_match = *negated;
                write!(output, "{}", no_match)?;
            }
            OneLetter(ch) => match ch {
                // Unicode class for Letters
                'L' => write!(output, "c.is_alphabetic()")?,
                // Unicode class for Numbers
                'N' => write!(output, "c.is_numeric()")?,
                // Unicode class for Whitespace
                'Z' => write!(output, "c.is_whitespace()")?,
                // Unicode class for Punctuation
                // Attention: Only ASCII based punctuation is supported
                'P' => write!(output, "c.is_ascii_punctuation()")?,
                // Unicode class for Control characters
                'C' => write!(output, "c.is_control()")?,
                _ => return Err(unsupported!(format!("{:#?}", c))),
            },
        }
        Ok(())
    }

    fn generate_code_from_class_perl(
        perl: &ClassPerl,
        output: &mut dyn std::io::Write,
    ) -> Result<()> {
        let ClassPerl { negated, kind, .. } = perl;
        write!(output, "            ")?;
        if *negated {
            write!(output, "!")?;
        }
        match kind {
            ClassPerlKind::Digit => write!(output, "c.is_numeric()")?,
            ClassPerlKind::Space => write!(output, "c.is_whitespace()")?,
            ClassPerlKind::Word => write!(output, "c.is_alphanumeric()")?,
        };
        Ok(())
    }

    fn generate_code_from_class_bracketed(
        bracketed: &ClassBracketed,
        output: &mut dyn std::io::Write,
    ) -> Result<()> {
        let ClassBracketed { negated, kind, .. } = bracketed;
        match kind {
            ClassSet::Item(item) => {
                Self::generate_code_from_set_item(item.clone(), *negated, output)
            }
            ClassSet::BinaryOp(bin_op) => {
                Self::generate_code_from_binary_op(bin_op.clone(), *negated, output)
            }
        }
    }

    fn generate_code_from_class_set_union(
        union: &ClassSetUnion,
        negated: bool,
        output: &mut dyn std::io::Write,
    ) -> Result<()> {
        let num_items = union.items.len();
        if negated {
            write!(output, "!(")?;
        }
        union.items.iter().enumerate().try_for_each(|(i, s)| {
            Self::generate_code_from_set_item(s.clone(), false, output)?;
            if i < num_items - 1 {
                write!(output, " || ")?;
            }
            Ok::<(), ScanGenError>(())
        })?;
        if negated {
            write!(output, ")")?;
        }
        Ok(())
    }

    fn generate_code_from_set_item(
        item: ClassSetItem,
        negated: bool,
        output: &mut dyn std::io::Write,
    ) -> Result<()> {
        match item {
            ClassSetItem::Empty(_) => write!(output, "false")?,
            ClassSetItem::Literal(ref l) => {
                let Literal { c, .. } = *l;
                if negated {
                    write!(output, "c != '{}'", c.escape_default())?
                } else {
                    write!(output, "c == '{}'", c.escape_default())?
                }
            }
            ClassSetItem::Range(ref r) => {
                let ClassSetRange {
                    ref start, ref end, ..
                } = *r;
                let start = start.c;
                let end = end.c;
                if negated {
                    write!(output, "!")?
                }
                write!(output, "('{}'..='{}').contains(&c)", start, end)?
            }
            ClassSetItem::Ascii(ref a) => {
                let ClassAscii {
                    ref kind, negated, ..
                } = *a;
                let match_function = match kind {
                    ClassAsciiKind::Alnum => "c.is_alphanumeric()",
                    ClassAsciiKind::Alpha => "c.is_alphabetic()",
                    ClassAsciiKind::Ascii => "c.is_ascii()",
                    ClassAsciiKind::Blank => "c.is_ascii_whitespace()",
                    ClassAsciiKind::Cntrl => "c.is_ascii_control()",
                    ClassAsciiKind::Digit => "c.is_numeric()",
                    ClassAsciiKind::Graph => "c.is_ascii_graphic()",
                    ClassAsciiKind::Lower => "c.is_lowercase()",
                    ClassAsciiKind::Print => "c.is_ascii_graphic()",
                    ClassAsciiKind::Punct => "c.is_ascii_punctuation()",
                    ClassAsciiKind::Space => "c.is_whitespace()",
                    ClassAsciiKind::Upper => "c.is_uppercase()",
                    ClassAsciiKind::Word => "c.is_alphanumeric()",
                    ClassAsciiKind::Xdigit => "c.is_ascii_hexdigit()",
                };
                if negated {
                    write!(output, "!")?
                }
                write!(output, "{}", match_function)?
            }
            ClassSetItem::Unicode(ref c) => Self::generate_code_from_class_unicode(c, output)?,
            ClassSetItem::Perl(ref c) => Self::generate_code_from_class_perl(c, output)?,
            ClassSetItem::Bracketed(ref c) => {
                Self::generate_code_from_class_bracketed(c, output)?;
            }
            ClassSetItem::Union(ref c) => {
                Self::generate_code_from_class_set_union(c, negated, output)?;
            }
        }
        Ok(())
    }

    fn generate_code_from_binary_op(
        clone: ClassSetBinaryOp,
        negated: bool,
        output: &mut dyn std::io::Write,
    ) -> Result<()> {
        let ClassSetBinaryOp { kind, lhs, rhs, .. } = clone;
        if negated {
            write!(output, "!(")?;
        }
        match kind {
            ClassSetBinaryOpKind::Intersection => {
                Self::generate_code_from_class_set(&lhs, output)?;
                write!(output, " && ")?;
                Self::generate_code_from_class_set(&rhs, output)?;
            }
            ClassSetBinaryOpKind::Difference => {
                Self::generate_code_from_class_set(&lhs, output)?;
                write!(output, " && !(")?;
                Self::generate_code_from_class_set(&rhs, output)?;
                write!(output, ")")?;
            }
            ClassSetBinaryOpKind::SymmetricDifference => {
                Self::generate_code_from_class_set(&lhs, output)?;
                write!(output, " != ")?;
                Self::generate_code_from_class_set(&rhs, output)?;
            }
        };
        if negated {
            write!(output, ")")?;
        }
        Ok(())
    }
}

impl std::fmt::Debug for MatchFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MatchFunction")
    }
}

impl TryFrom<Ast> for MatchFunction {
    type Error = ScanGenError;

    fn try_from(ast: Ast) -> Result<Self> {
        let match_function = match ast {
            Ast::Empty(_) => {
                // An empty AST matches everything.
                MatchFunction::new(|_| true)
            }
            Ast::Dot(_) => {
                // A dot AST matches any character except newline.
                MatchFunction::new(|ch| ch != '\n' && ch != '\r')
            }
            Ast::Literal(ref l) => {
                // A literal AST matches a single character.
                let Literal { c, .. } = **l;
                MatchFunction::new(move |ch| ch == c)
            }
            Ast::ClassUnicode(ref c) => Self::try_from_class_unicode(*c.clone())?,
            Ast::ClassPerl(ref c) => Self::try_from_class_perl(*c.clone())?,
            Ast::ClassBracketed(ref c) => Self::try_from_class_bracketed(*c.clone())?,
            _ => return Err(unsupported!(format!("{:#?}", ast))),
        };
        Ok(match_function)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex_syntax::ast::parse::Parser;

    #[test]
    fn test_match_function_unicode_class() {
        let ast = Parser::new().parse(r"\pL").unwrap();
        let match_function = MatchFunction::try_from(ast).unwrap();
        assert!(match_function.call('a'));
        assert!(match_function.call('A'));
        assert!(!match_function.call('1'));
        assert!(!match_function.call(' '));
    }

    #[test]
    fn test_match_function_perl_class() {
        let ast = Parser::new().parse(r"\d").unwrap();
        let match_function = MatchFunction::try_from(ast).unwrap();
        assert!(match_function.call('1'));
        assert!(!match_function.call('a'));
    }

    #[test]
    fn test_match_function_bracketed_class() {
        let ast = Parser::new().parse(r"[a-z]").unwrap();
        let match_function = MatchFunction::try_from(ast).unwrap();
        assert!(match_function.call('a'));
        assert!(match_function.call('z'));
        assert!(!match_function.call('A'));
        assert!(!match_function.call('1'));
    }

    #[test]
    fn test_match_function_binary_op_class_intersection() {
        // Intersection (matching x or y)
        let ast = Parser::new().parse(r"[a-y&&xyz]").unwrap();
        let match_function = MatchFunction::try_from(ast).unwrap();
        assert!(match_function.call('x'));
        assert!(match_function.call('y'));
        assert!(!match_function.call('a'));
        assert!(!match_function.call('z'));
    }

    #[test]
    fn test_match_function_union_class() {
        let ast = Parser::new().parse(r"[0-9a-z]").unwrap();
        let match_function = MatchFunction::try_from(ast).unwrap();
        assert!(match_function.call('a'));
        assert!(match_function.call('z'));
        assert!(match_function.call('0'));
        assert!(match_function.call('9'));
        assert!(!match_function.call('!'));
    }

    #[test]
    fn test_match_function_negated_bracketed_class() {
        let ast = Parser::new().parse(r"[^a-z]").unwrap();
        let match_function = MatchFunction::try_from(ast).unwrap();
        assert!(!match_function.call('a'));
        assert!(!match_function.call('z'));
        assert!(match_function.call('A'));
        assert!(match_function.call('1'));
    }

    #[test]
    fn test_match_function_negated_binary_op_class() {
        let ast = Parser::new().parse(r"[a-z&&[^aeiou]]").unwrap();
        let match_function = MatchFunction::try_from(ast).unwrap();
        assert!(!match_function.call('a'));
        assert!(!match_function.call('e'));
        assert!(match_function.call('z'));
        assert!(!match_function.call('1'));
    }

    // [[:alpha:]]   ASCII character class ([A-Za-z])
    #[test]
    fn test_match_function_ascci_class() {
        let ast = Parser::new().parse(r"[[:alpha:]]").unwrap();
        let match_function = MatchFunction::try_from(ast).unwrap();
        assert!(match_function.call('a'));
        assert!(match_function.call('A'));
        assert!(match_function.call('ä'));
        assert!(!match_function.call('1'));
        assert!(!match_function.call(' '));
    }

    // [[:^alpha:]]  Negated ASCII character class ([^A-Za-z])
    #[test]
    fn test_match_function_negated_ascii_class() {
        let ast = Parser::new().parse(r"[^[:alpha:]]").unwrap();
        let match_function = MatchFunction::try_from(ast).unwrap();
        assert!(!match_function.call('a'));
        assert!(!match_function.call('A'));
        assert!(!match_function.call('ä'));
        assert!(match_function.call('1'));
        assert!(match_function.call(' '));
    }

    #[test]
    fn test_match_function_empty() {
        let ast = Parser::new().parse(r"").unwrap();
        let match_function = MatchFunction::try_from(ast).unwrap();
        assert!(match_function.call('a'));
        assert!(match_function.call('A'));
        assert!(match_function.call('1'));
        assert!(match_function.call(' '));
    }

    // [x[^xyz]]     Nested/grouping character class (matching any character except y and z)
    #[test]
    fn test_nested_classes() {
        let ast = Parser::new().parse(r"[x[^xyz]]").unwrap();
        let match_function = MatchFunction::try_from(ast).unwrap();
        assert!(match_function.call('a'));
        assert!(match_function.call('x'));
        assert!(match_function.call('1'));
        assert!(match_function.call(' '));
        assert!(!match_function.call('y'));
        assert!(!match_function.call('z'));
    }

    // [0-9&&[^4]]   Subtraction using intersection and negation (matching 0-9 except 4)
    #[test]
    fn test_subtraction() {
        let ast = Parser::new().parse(r"[0-9&&[^4]]").unwrap();
        let match_function = MatchFunction::try_from(ast).unwrap();
        assert!(match_function.call('0'));
        assert!(match_function.call('9'));
        assert!(!match_function.call('4'));
        assert!(!match_function.call('a'));
    }

    // [0-9--4]      Direct subtraction (matching 0-9 except 4)
    #[test]
    fn test_direct_subtraction() {
        let ast = Parser::new().parse(r"[0-9--4]").unwrap();
        let match_function = MatchFunction::try_from(ast).unwrap();
        assert!(match_function.call('0'));
        assert!(match_function.call('9'));
        assert!(!match_function.call('4'));
        assert!(!match_function.call('a'));
    }

    // [a-g~~b-h]    Symmetric difference (matching `a` and `h` only)
    #[test]
    fn test_symmetric_difference() {
        let ast = Parser::new().parse(r"[a-g~~b-h]").unwrap();
        let match_function = MatchFunction::try_from(ast).unwrap();
        assert!(match_function.call('a'));
        assert!(match_function.call('h'));
        assert!(!match_function.call('b'));
        assert!(!match_function.call('z'));
    }

    // [\[\]]        Escaping in character classes (matching [ or ])
    #[test]
    fn test_escaping() {
        let ast = Parser::new().parse(r"[\[\]]").unwrap();
        let match_function = MatchFunction::try_from(ast).unwrap();
        assert!(match_function.call('['));
        assert!(match_function.call(']'));
        assert!(!match_function.call('a'));
        assert!(!match_function.call('1'));
    }

    // [a&&b]        An empty character class matching nothing
    #[test]
    fn test_empty_intersection() {
        let ast = Parser::new().parse(r"[a&&b]").unwrap();
        let match_function = MatchFunction::try_from(ast).unwrap();
        assert!(!match_function.call('a'));
        assert!(!match_function.call('b'));
        assert!(!match_function.call('1'));
        assert!(!match_function.call(' '));
    }
}
