//! # EBNF lexer
//!
//! EBNF (or Extended Backus-Naur Form) language consists of the following lexemes:
//! - terminals (e.g.: `"fn"`, `">="`);
//! - non-terminals (e.g.: `<Condition>`, `<Rule>`);
//! - 2 operators, namely: 'definition' (`::=`) and 'alternative' (`|`);
//! - repetitions (`{`, `}`);
//! - options (`[`, `]`);
//! - grouping parenthesis (`(`, `)`);
//! - rules delimiter: a semicolon (`;`);
//! - comment: everything after `//` until the end of line.
//!
//! Delimiter is optional after the last rule.
pub use self::{EbnfOperator::*, EbnfToken::*, Side::*};
use lex::{Lexer, LexerBuilder, MetaIter, Token};

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub enum EbnfToken<'a> {
    Terminal(&'a str),
    NonTerminal(&'a str),
    Operator(EbnfOperator),
    Repeat(Side),
    Optional(Side),
    Group(Side),
    Delimiter,
    Comment(&'a str),
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub enum EbnfOperator {
    /// Definition `"::="`
    Def,
    /// Alternative `"|"`
    Alt,
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub enum Side {
    Start,
    End,
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

pub fn make_lexer<'a>() -> Lexer<'a, EbnfToken<'a>> {
    LexerBuilder::new()
        .skip_whitespaces(whitespace_filter)
        .add(r"::=", constant!(Operator(Def)))
        .add(r"\|", constant!(Operator(Alt)))
        .add(r"<(.+?)>", |c| NonTerminal(c.get(1).unwrap().as_str()))
        .add("\"(.*?)\"", |c| Terminal(c.get(1).unwrap().as_str()))
        .add(r"\{", constant!(Repeat(Start)))
        .add(r"\}", constant!(Repeat(End)))
        .add(r"\[", constant!(Optional(Start)))
        .add(r"\]", constant!(Optional(End)))
        .add(r"\(", constant!(Group(Start)))
        .add(r"\)", constant!(Group(End)))
        .add(r";", constant!(Delimiter))
        .add(r"//([^\n]*)\n?", |c| Comment(c.get(1).unwrap().as_str()))
        .add(r"(?s)/\*(.*?)\*/", |c| Comment(c.get(1).unwrap().as_str()))
        .build()
}

impl<'a> Token<'a> for EbnfToken<'a> {
    fn describe(&self) -> String {
        match *self {
            Terminal(t) => format!("\"{}\"", t),
            NonTerminal(t) => format!("<{}>", t),
            Comment(c) => format!("/* {} */\n", c),
            _ => match *self {
                Operator(Def) => "::=",
                Operator(Alt) => "|",
                Repeat(Start) => "{",
                Repeat(End) => "}",
                Optional(Start) => "[",
                Optional(End) => "]",
                Group(Start) => "(",
                Group(End) => ")",
                Delimiter => ";",
                _ => unreachable!(),
            }.to_string(),
        }
    }

    fn descriptor(&self) -> &'static str {
        match *self {
            Terminal(..) => "Terminal",
            NonTerminal(..) => "NonTerminal",
            Operator(Def) => "::=",
            Operator(Alt) => "|",
            Repeat(Start) => "{",
            Repeat(End) => "}",
            Optional(Start) => "[",
            Optional(End) => "]",
            Group(Start) => "(",
            Group(End) => ")",
            Delimiter => ";",
            Comment(_) => "Comment",
        }
    }
}

pub struct DropComments<I> {
    inner: I,
}

pub fn drop_comments<'a, I>(tokens: I) -> DropComments<I>
where
    I: MetaIter<'a, EbnfToken<'a>>,
{
    DropComments { inner: tokens }
}

mod impls {
    use super::*;
    use lex::{MetaResult, TokenMeta};

    impl<'a, I> Iterator for DropComments<I>
    where
        I: Iterator<Item = MetaResult<'a, EbnfToken<'a>>>,
    {
        type Item = MetaResult<'a, EbnfToken<'a>>;

        fn next(&mut self) -> Option<<Self as Iterator>::Item> {
            let mut next = self.inner.next();
            while let Some(Ok(TokenMeta {
                token: EbnfToken::Comment(_),
                ..
            })) = next
            {
                next = self.inner.next();
            }
            next
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lex::TokensExt;

    const SOURCE: &str = r#"
        <A> // x y z
            ::= (<B> | {/**/"c"}) [<D>]
            ;
    "#;

    const FILENAME: &str = "test.bnf";

    const TOKENS: &[EbnfToken] = &[
        NonTerminal("A"),
        Comment(" x y z"),
        Operator(Def),
        Group(Start),
        NonTerminal("B"),
        Operator(Alt),
        Repeat(Start),
        Comment(""),
        Terminal("c"),
        Repeat(End),
        Group(End),
        Optional(Start),
        NonTerminal("D"),
        Optional(End),
        Delimiter,
    ];

    #[test]
    fn test_lexer() {
        let tokens: Vec<_> = make_lexer()
            .into_tokens(SOURCE, FILENAME.into())
            .into_raw()
            .collect();

        assert_eq!(tokens, TOKENS);
    }

    #[test]
    fn test_drop_comments() {
        let tokens: Vec<_> = drop_comments(make_lexer().into_tokens(SOURCE, FILENAME.into()))
            .into_raw()
            .collect();

        let expected: Vec<_> = TOKENS.into_iter()
            .cloned()
            .filter(|t| ::std::mem::discriminant(t) != ::std::mem::discriminant(&Comment("")))
            .collect();
        assert_eq!(tokens, expected);
    }
}
