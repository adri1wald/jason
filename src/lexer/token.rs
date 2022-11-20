pub use TokenKind::*;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Integer(isize),
    Decimal(f64),
    QuotedString(String),
    OpenBracket,
    CloseBracket,
    OpenSquare,
    CloseSquare,
    Colon,
    Comma,
    True,
    False,
    Null,
    Whitespace,
    /// Not part of spec
    Eof,
    InvalidIdent(String),
    Unknown(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub base: usize,
    pub len: usize,
}

impl Span {
    pub fn new(lo: usize, hi: usize) -> Self {
        let (base, len) = (lo, hi - lo);
        Span { base, len }
    }
}

const DUMMY_SPAN: Span = Span { base: 0, len: 0 };

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }

    pub fn dummy() -> Self {
        Token::new(Unknown("".to_owned()), DUMMY_SPAN)
    }
}
