//! # BNF lexer
//!
//! This variant of BNF language consists of the following lexemes:
//! - terminals in double quotes (e.g.: `"fn"`, `">="`);
//! - non-terminals in triangle quotes (e.g.: `<Condition>`, `<Rule>`);
//! - 2 operators: 'definition' (`::=`) and 'alternative' (`|`);
//! - rules delimiter: a semicolon (`;`).
//!
//! Delimiter is optional after the last rule.
use lex::{Lexer, LexerBuilder, Token};

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub enum BnfToken<'a> {
    Terminal(&'a str),
    NonTerminal(&'a str),
    Operator(BnfOperator),
    Delimiter,
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub enum BnfOperator {
    /// Definition `"::="`
    Def,
    /// Alternative `"|"`
    Alt,
}

fn is_whitespace(c: char) -> bool {
    let c = c as u8;
    return c == 0x20 // spaces (U+0020)
        || c == 0x09 // horizontal tabs (U+0009)
        || c == 0x0d // carriage returns (U+000D)
        || c == 0x0a; // newlines (U+000A)
}

fn whitespace_filter(source: &str) -> &str {
    for (i, c) in source.char_indices() {
        if !is_whitespace(c) {
            return &source[i..];
        }
    }
    &source[source.len()..]
}

pub fn make_lexer<'a>() -> Lexer<'a, BnfToken<'a>> {
    LexerBuilder::new()
        .skip_whitespaces(whitespace_filter)
        .add(r";", constant!(BnfToken::Delimiter))
        .add(r"::=", constant!(BnfToken::Operator(BnfOperator::Def)))
        .add(r"\|", constant!(BnfToken::Operator(BnfOperator::Alt)))
        .add(r"<(.+?)>", |c| {
            BnfToken::NonTerminal(c.get(1).unwrap().as_str())
        }).add("\"(.*?)\"", |c| {
            BnfToken::Terminal(c.get(1).unwrap().as_str())
        }).build()
}

impl<'a> Token<'a> for BnfToken<'a> {
    fn describe(&self) -> String {
        match self {
            BnfToken::Terminal(s) => format!("\"{}\"", s),
            BnfToken::NonTerminal(s) => format!("<{}>", s),
            BnfToken::Operator(BnfOperator::Def) => "::=".to_owned(),
            BnfToken::Operator(BnfOperator::Alt) => "|".to_owned(),
            BnfToken::Delimiter => ";".to_owned(),
        }
    }

    fn descriptor(&self) -> &'static str {
        match self {
            BnfToken::Terminal(..) => "Terminal",
            BnfToken::NonTerminal(..) => "NonTerminal",
            BnfToken::Operator(BnfOperator::Def) => "::=",
            BnfToken::Operator(BnfOperator::Alt) => "|",
            BnfToken::Delimiter => ";",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::BnfOperator::*;
    use super::BnfToken::*;
    use super::*;
    use lex::TokensExt;

    const SOURCE: &str = r#"
        <A> ::= <B> | "c" <D> ;
    "#;

    const FILENAME: &str = "test.bnf";

    const TOKENS: &[BnfToken] = &[
        NonTerminal("A"),
        Operator(Def),
        NonTerminal("B"),
        Operator(Alt),
        Terminal("c"),
        NonTerminal("D"),
        Delimiter,
    ];

    #[test]
    fn test_lexer() {
        let lexer = make_lexer();
        let tokens: Vec<_> = lexer
            .into_tokens(SOURCE, FILENAME.into())
            .into_raw()
            .collect();

        assert_eq!(tokens, TOKENS);
    }
}
