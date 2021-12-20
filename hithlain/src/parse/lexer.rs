use crate::parse::source::Source;
use crate::parse::span::Span;
use derive_more::Display;
use logos::{Lexer, Logos};
use miette::{Diagnostic, NamedSource, SourceSpan};
use peekmore::{PeekMore, PeekMoreIterator};
use thiserror::Error;

fn nano(lex: &mut Lexer<Token>) -> Option<u64> {
    let slice = lex.slice();
    let n: u64 = slice[..slice.len() - 2].parse().ok()?; // skip 'ns'
    Some(n)
}

fn micro(lex: &mut Lexer<Token>) -> Option<u64> {
    let slice = lex.slice();
    let n: u64 = slice[..slice.len() - 2].parse().ok()?; // skip 'us'
    Some(n * 1_000)
}

fn milli(lex: &mut Lexer<Token>) -> Option<u64> {
    let slice = lex.slice();
    let n: u64 = slice[..slice.len() - 2].parse().ok()?; // skip 'ms'
    Some(n * 1_000_000)
}

fn second(lex: &mut Lexer<Token>) -> Option<u64> {
    let slice = lex.slice();
    let n: u64 = slice[..slice.len() - 1].parse().ok()?; // skip 'ms'
    Some(n * 1_000_000_000)
}

#[derive(Logos, Debug, PartialEq, Clone, Display)]
pub enum Token {
    #[token("and")]
    #[display(fmt = "and")]
    And,
    #[token("or")]
    #[display(fmt = "or")]
    Or,
    #[token("nand")]
    #[display(fmt = "nand")]
    Nand,
    #[token("nor")]
    #[display(fmt = "nor")]
    Nor,
    #[token("xor")]
    #[display(fmt = "xor")]
    Xor,
    #[token("xnor")]
    #[display(fmt = "xnor")]
    Xnor,
    #[token("==")]
    #[display(fmt = "==")]
    Eq,
    #[token("not")]
    #[display(fmt = "not")]
    Not,

    #[token("assert")]
    #[display(fmt = "assert")]
    Assert,

    #[token("at")]
    #[display(fmt = "absolute time specification")]
    At,

    #[token("after")]
    #[display(fmt = "relative time specification")]
    After,

    #[token("circuit")]
    #[display(fmt = "circuit")]
    Circuit,

    #[token("test")]
    #[display(fmt = "test")]
    Test,

    #[token("process")]
    #[display(fmt = "process")]
    Process,

    #[token(":")]
    #[display(fmt = ":")]
    Colon,
    #[token(";")]
    #[display(fmt = ";")]
    SemiColon,
    #[token(",")]
    #[display(fmt = ",")]
    Comma,

    #[token("{")]
    #[display(fmt = "{{")]
    LBrace,
    #[token("}")]
    #[display(fmt = "}}")]
    RBrace,

    #[token("(")]
    #[display(fmt = "(")]
    LParen,
    #[token(")")]
    #[display(fmt = ")")]
    RParen,

    #[display(fmt = "->")]
    #[token("->")]
    Arrow,

    #[display(fmt = "=")]
    #[token("=")]
    Assignment,

    #[display(fmt = "variable name ({})", _0)]
    #[regex("[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Name(String),

    #[display(fmt = "bit")]
    #[regex("[01]", |lex| lex.slice() != "0", priority=2)]
    Bit(bool),

    #[display(fmt = "number")]
    #[regex("[0-9]+", |lex| lex.slice().parse())]
    Number(u64),

    #[regex("[0-9]+ns", nano, priority = 3)]
    #[regex("[0-9]+us", micro, priority = 3)]
    #[regex("[0-9]+ms", milli, priority = 3)]
    #[regex("[0-9]+s", second, priority = 3)]
    Time(u64),

    #[error]
    #[display(fmt = "error")]
    #[regex(r"//.*\n", logos::skip)]
    #[regex(r"[ \t\n\f\r]+", logos::skip)]
    Error,
}

#[derive(Debug, Error, Diagnostic)]
#[error("invalid token")]
#[diagnostic()]
pub struct LexError {
    #[source_code]
    src: NamedSource,

    #[label("this token")]
    location: SourceSpan,
}

pub struct TokenStream {
    pub(crate) tokens: Vec<(Token, Span)>,
}

pub struct TokenIterator(PeekMoreIterator<std::vec::IntoIter<(Token, Span)>>);

impl TokenIterator {
    pub fn peek(&mut self) -> Option<&(Token, Span)> {
        self.0.peek()
    }

    pub fn peek_2(&mut self) -> Option<[(Token, Span); 2]> {
        let res = self.0.peek_amount(2);
        Some([res[0].clone()?, res[1].clone()?])
    }
}

impl Iterator for TokenIterator {
    type Item = (Token, Span);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl IntoIterator for TokenStream {
    type Item = (Token, Span);
    type IntoIter = TokenIterator;

    fn into_iter(self) -> Self::IntoIter {
        TokenIterator(self.tokens.into_iter().peekmore())
    }
}

pub fn lex(source: &Source) -> Result<TokenStream, LexError> {
    let mut lexed: logos::Lexer<Token> = Token::lexer(source.text());

    let mut res = Vec::new();

    while let Some(i) = lexed.next() {
        if let Token::Error = i {
            return Err(LexError {
                src: NamedSource::new(source.name(), source.to_string()),
                location: Span::from_logos(lexed.span(), source.clone()).into(),
            });
        }

        res.push((i, Span::from_logos(lexed.span(), source.clone())));
    }

    Ok(TokenStream { tokens: res })
}

#[cfg(test)]
mod tests {
    use crate::error::NiceUnwrap;
    use crate::parse::lexer::{lex, Token};
    use crate::parse::source::Source;

    #[test]
    fn test_smoke() {
        let src = "
        circuit main: a b c -> d e {
            d = a and b;
            e = b or c;
        }
        ";

        lex(&Source::test(src)).nice_unwrap_panic();
    }

    #[test]
    fn timespec() {
        let src = "3ns";
        let tokens = lex(&Source::test(src)).nice_unwrap();
        assert!(
            matches!(tokens.tokens.first(), Some(&(Token::Time(3), _))),
            "{:?}",
            tokens.tokens.first()
        );

        let src = "3us";
        let tokens = lex(&Source::test(src)).nice_unwrap();
        assert!(
            matches!(tokens.tokens.first(), Some(&(Token::Time(3_000), _))),
            "{:?}",
            tokens.tokens.first()
        );

        let src = "3ms";
        let tokens = lex(&Source::test(src)).nice_unwrap();
        assert!(
            matches!(tokens.tokens.first(), Some(&(Token::Time(3_000_000), _))),
            "{:?}",
            tokens.tokens.first()
        );

        let src = "3s";
        let tokens = lex(&Source::test(src)).nice_unwrap();
        assert!(
            matches!(
                tokens.tokens.first(),
                Some(&(Token::Time(3_000_000_000), _))
            ),
            "{:?}",
            tokens.tokens.first()
        );
    }

    #[test]
    fn test_bit_number() {
        let src = "3";

        let tokens = lex(&Source::test(src)).nice_unwrap();

        assert!(matches!(
            tokens.tokens.first(),
            Some(&(Token::Number(3), _))
        ));

        let src = "1";

        let tokens = lex(&Source::test(src)).nice_unwrap();

        assert!(matches!(
            tokens.tokens.first(),
            Some(&(Token::Bit(true), _))
        ));
    }
}
