pub use TokenKind::*;

use super::unescape::EscapeError;

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

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Int(isize),
    Float(f64),
    Str(String),
    InvalidStr(StrError, usize),
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

/// Errors that can occur during string parsing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StrError {
    Unterminated,
    InvalidEscape,
    BareBackspace,
    BareFormFeed,
    BareLineFeed,
    BareCarriageReturn,
    BareHorizontalTab,
    BadControlChar,
    EscapeOnlyChar,
    BadUnicodeEscape,
    LoneSurrogateUnicodeEscape,
    OutOfRangeUnicodeEscape,
}

impl From<EscapeError> for StrError {
    fn from(other: EscapeError) -> Self {
        match other {
            EscapeError::LoneSlash => StrError::Unterminated,
            EscapeError::InvalidEscape => StrError::InvalidEscape,
            EscapeError::BareBackspace => StrError::BareBackspace,
            EscapeError::BareFormFeed => StrError::BareFormFeed,
            EscapeError::BareLineFeed => StrError::BareLineFeed,
            EscapeError::BareCarriageReturn => StrError::BareCarriageReturn,
            EscapeError::BareHorizontalTab => StrError::BareHorizontalTab,
            EscapeError::BadControlChar => StrError::BadControlChar,
            EscapeError::EscapeOnlyChar => StrError::EscapeOnlyChar,
            EscapeError::BadUnicodeEscape => StrError::BadUnicodeEscape,
            EscapeError::LoneSurrogateUnicodeEscape => StrError::LoneSurrogateUnicodeEscape,
            EscapeError::OutOfRangeUnicodeEscape => StrError::OutOfRangeUnicodeEscape,
        }
    }
}
