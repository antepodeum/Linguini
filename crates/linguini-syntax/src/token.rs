use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub const fn shift(self, offset: usize) -> Self {
        Self {
            start: self.start + offset,
            end: self.end + offset,
        }
    }
}

impl chumsky::span::Span for Span {
    type Context = ();
    type Offset = usize;

    fn new(_: Self::Context, range: std::ops::Range<Self::Offset>) -> Self {
        Self {
            start: range.start,
            end: range.end,
        }
    }

    fn context(&self) -> Self::Context {}

    fn start(&self) -> Self::Offset {
        self.start
    }

    fn end(&self) -> Self::Offset {
        self.end
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub const fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    Ident(String),
    LocaleTag(String),
    String(String),
    RawText(String),
    Error(String),
    LBrace,
    RBrace,
    LParen,
    RParen,
    Comma,
    Colon,
    Equals,
    Arrow,
    Dot,
    At,
    TripleQuote,
    Newline,
    Whitespace,
    Comment(String),
    DocComment(String),
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ident(value) => write!(f, "identifier `{value}`"),
            Self::LocaleTag(value) => write!(f, "locale tag `{value}`"),
            Self::String(_) => f.write_str("string literal"),
            Self::RawText(_) => f.write_str("text"),
            Self::Error(value) => write!(f, "invalid token `{value}`"),
            Self::LBrace => f.write_str("`{`"),
            Self::RBrace => f.write_str("`}`"),
            Self::LParen => f.write_str("`(`"),
            Self::RParen => f.write_str("`)`"),
            Self::Comma => f.write_str("`,`"),
            Self::Colon => f.write_str("`:`"),
            Self::Equals => f.write_str("`=`"),
            Self::Arrow => f.write_str("`=>`"),
            Self::Dot => f.write_str("`.`"),
            Self::At => f.write_str("`@`"),
            Self::TripleQuote => f.write_str("`\"\"\"`"),
            Self::Newline => f.write_str("newline"),
            Self::Whitespace => f.write_str("whitespace"),
            Self::Comment(_) => f.write_str("comment"),
            Self::DocComment(_) => f.write_str("doc comment"),
        }
    }
}
