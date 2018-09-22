use super::{Token, TokenFactory};
use ::{Lexer, LexerBuilder};
use regex::Match;
pub use self::GoToken::*;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub enum GoToken<'a> {
    /// identifiers
    Ident(&'a str),
    /// keywords
    Keyword(GoKeyword),
    /// operators and punctuation,
    Operator(GoOperator),
    // literals
    Literal(GoLiteral<'a>),
    // White space
    Comment(&'a str),
}

/// Go programming language keywords
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub enum GoKeyword {
    Break,
    Default,
    Func,
    Interface,
    Select,
    Case,
    Defer,
    Go,
    Map,
    Struct,
    Chan,
    Else,
    Goto,
    Package,
    Switch,
    Const,
    Fallthrough,
    If,
    Range,
    Type,
    Continue,
    For,
    Import,
    Return,
    Var,
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub enum GoOperator {
    Add,
    Sub,
    Mul,
    Quo,
    Rem,
    
    And,
    Or,
    Xor,
    Shl,
    Shr,
    AndNot,

    AddAssign,
    SubAssign,
    QuoAssign,
    RemAssign,
    MulAssign,

    AndAssign,
    OrAssign,
    XorAssign,
    ShlAssign,
    ShrAssign,
    AndNotAssign,

    LAnd,
    LOr,
    Arrow,
    Inc,
    Dec,

    Eql,
    Lss,
    Gtr,
    Assign,
    Not,

    NEq,
    LEq,
    GEq,
    Define,
    Ellipsis,

    LParen,
    LBrack,
    LBrace,
    Comma,
    Period,

    RParen,
    RBrack,
    RBrace,
    Semicolon,
    Colon,
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub enum GoLiteral<'a> {
    String(&'a str),
    Integer(&'a str),
    Float(&'a str),
    Imaginary(&'a str),
    Rune(&'a str),
}

pub fn make_lexer<'a>() -> Lexer<'a, GoToken<'a>> {
    let constant = |x| { move |_| x };

    let rune: &str = r#"(?x)
        ' # open quote
        ( # unicode_value = unicode_char | little_u_value | big_u_value | escaped_char

              # unicode_char = /* an arbitrary Unicode code point except newline */
                [^\\\n]
            | # little_u_value
                \\u ([0-9A-Fa-f]){4}  # TODO: change to [[:xdigit:]]
            | # big_u_value
                \\U ([0-9A-Fa-f]){8}  # TODO: change to [[:xdigit:]]
            | # escaped_char
                \\   [abfnrtv\\'"]

        | # byte value = octal_byte_value | hex_byte_value

              # octal_byte_value
                \\   [0-7]{3}
            | # hex_byte_value
                \\x ([0-9A-Fa-f]){2}  # TODO: change to [[:xdigit:]]
        )
        ' # close quote
    "#;


    LexerBuilder::new()
        .add(r"-", constant(Operator(GoOperator::Dec)))
        .add(rune, |c| GoToken::Literal(GoLiteral::Rune(c.get(0).unwrap().as_str())))
        .build() 
}


impl<'a> Token<'a> for GoToken<'a> {
    fn describe(&self) -> String {
        match *self {
            GoToken::Ident(ref id) => id.to_string(),
            GoToken::Keyword(ref kw) => format!("{:?}", kw),
            _ => format!("{:?}", self),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_rune() {
        let lexer = make_lexer();

        let runes_source = r"'a'
'ä'
'本'
'\t'
'\000'
'\007'
'\377'
'\x07'
'\xff'
'\u12e4'
'\U00101234'
'\''
";
        for rune in runes_source.lines() {
            assert_eq!(lexer.next(rune).unwrap().1,
                       GoToken::Literal(GoLiteral::Rune(rune)));
        }
    }
}