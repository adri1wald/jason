mod cursor;
pub mod unescape;

pub use cursor::Cursor;

use self::TokenKind::*;

#[derive(Debug, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub len: usize,
}

impl Token {
    fn new(kind: TokenKind, len: usize) -> Self {
        Self { kind, len }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Int,
    Float,
    Str {
        terminated: bool,
    },
    OpenBracket,
    CloseBracket,
    OpenSquare,
    CloseSquare,
    Colon,
    Comma,
    Ident,
    Whitespace,
    /// Not part of spec
    Eof,
    Unknown,
}

pub fn is_id_start(c: char) -> bool {
    // This is XID_Start OR '_' (which formally is not a XID_Start).
    unicode_xid::UnicodeXID::is_xid_start(c)
}

pub fn is_id_continue(c: char) -> bool {
    unicode_xid::UnicodeXID::is_xid_continue(c)
}

impl Cursor<'_> {
    pub fn advance_token(&mut self) -> Token {
        let first_char = match self.bump() {
            Some(c) => c,
            None => return Token::new(Eof, 0),
        };
        let token_kind = match first_char {
            // Whitespace sequence.
            c if c.is_whitespace() => self.whitespace(),

            // Identifier.
            c if is_id_start(c) => self.ident(),

            // Numeric literal.
            c @ '0'..='9' => self.number(c),
            '-' => {
                match self.first() {
                    '0'..='9' => {
                        // correctness: first() returned a non-eof char so there will be one.
                        let c = self.bump().unwrap();
                        self.number(c)
                    }
                    _ => return Token::new(Unknown, 1),
                }
            }

            '{' => OpenBracket,
            '}' => CloseBracket,
            '[' => OpenSquare,
            ']' => CloseSquare,
            ':' => Colon,
            ',' => Comma,

            // String literal.
            '"' => {
                let terminated = self.double_quoted_string();
                Str { terminated }
            }
            _ => Unknown,
        };

        let tok = Token::new(token_kind, self.pos_within_token());
        self.reset_pos_within_token();
        tok
    }

    fn whitespace(&mut self) -> TokenKind {
        debug_assert!(self.prev().is_whitespace());
        self.eat_while(|ch| ch.is_whitespace());
        Whitespace
    }

    fn ident(&mut self) -> TokenKind {
        debug_assert!(is_id_start(self.prev()));
        self.eat_while(is_id_continue);
        Ident
    }

    fn number(&mut self, first_digit: char) -> TokenKind {
        debug_assert!('0' <= self.prev() && self.prev() <= '9');

        if first_digit == '0' {
            let has_digits = match self.first() {
                '.' => match self.second() {
                    '0'..='9' => true,
                    _ => false,
                },
                'e' | 'E' => match self.second() {
                    '0'..='9' => true,
                    '+' | '-' => match self.third() {
                        '0'..='9' => true,
                        _ => false,
                    },
                    _ => false,
                },
                _ => false,
            };
            if !has_digits {
                // Just a 0.
                // E.g. JSON spec says `000` and `001` are invalid.
                return Int;
            }
        } else {
            self.eat_decimal_digits();
        }

        match self.first() {
            '.' => match self.second() {
                '0'..='9' => {
                    self.bump();
                    self.eat_decimal_digits();
                    match self.first() {
                        'e' | 'E' => match self.second() {
                            '0'..='9' => {
                                self.bump();
                                self.eat_decimal_digits();
                                Float
                            }
                            '+' | '-' => match self.third() {
                                '0'..='9' => {
                                    self.bump();
                                    self.bump();
                                    self.eat_decimal_digits();
                                    Float
                                }
                                _ => Float,
                            },
                            _ => Float,
                        },
                        _ => Float,
                    }
                }
                _ => Int,
            },
            'e' | 'E' => match self.second() {
                '0'..='9' => {
                    self.bump();
                    self.eat_decimal_digits();
                    Float
                }
                '+' | '-' => match self.third() {
                    '0'..='9' => {
                        self.bump();
                        self.bump();
                        self.eat_decimal_digits();
                        Float
                    }
                    _ => Int,
                },
                _ => Int,
            },
            _ => Int,
        }
    }

    fn double_quoted_string(&mut self) -> bool {
        debug_assert!(self.prev() == '"');
        while let Some(c) = self.bump() {
            match c {
                '"' => {
                    return true;
                }
                '\\' if self.first() == '\\' || self.first() == '"' => {
                    self.bump();
                }
                _ => (),
            }
        }
        // End of file reached.
        false
    }

    fn eat_decimal_digits(&mut self) -> bool {
        let mut has_digits = false;
        loop {
            match self.first() {
                '0'..='9' => {
                    has_digits = true;
                    self.bump();
                }
                _ => break,
            }
        }
        has_digits
    }
}

// Tests.

macro_rules! tokenize_test {
    ($name:ident, $input:expr, $tokens:expr) => {
        #[cfg(test)]
        #[test]
        fn $name() {
            pub fn tokenize(input: &str) -> impl Iterator<Item = Token> + '_ {
                let mut cursor = Cursor::new(input);
                std::iter::from_fn(move || {
                    let token = cursor.advance_token();
                    if token.kind != Eof {
                        Some(token)
                    } else {
                        None
                    }
                })
            }

            let mut token_iterator = tokenize($input);

            for token in $tokens {
                assert_eq!(token_iterator.next(), Some(token));
            }

            assert_eq!(token_iterator.next(), None);
        }
    };
}

// Identifier tests.

tokenize_test!(it_tokenizes_true, "true", [Token::new(Ident, 4)]);

tokenize_test!(it_tokenizes_false, "false", [Token::new(Ident, 5)]);

tokenize_test!(it_tokenizes_null, "null", [Token::new(Ident, 4)]);

tokenize_test!(it_tokenizes_invalid_ident, "potato", [Token::new(Ident, 6)]);

// Numeric literal tests.

tokenize_test!(it_tokenizes_an_integer, "420", [Token::new(Int, 3)]);

tokenize_test!(
    it_tokenizes_a_negative_integer,
    "-1600",
    [Token::new(Int, 5)]
);

tokenize_test!(
    it_tokenizes_an_integer_with_space_around,
    " 69\n \r",
    [
        Token::new(Whitespace, 1),
        Token::new(Int, 2),
        Token::new(Whitespace, 3)
    ]
);

tokenize_test!(it_tokenizes_a_decimal, "3.14", [Token::new(Float, 4)]);

tokenize_test!(
    it_tokenizes_a_negative_decimal,
    "-0.618",
    [Token::new(Float, 6)]
);

tokenize_test!(
    it_tokenizes_two_zeros,
    "00",
    [Token::new(Int, 1), Token::new(Int, 1)]
);

tokenize_test!(
    it_tokenizes_a_integer_with_lone_period,
    "1.",
    [Token::new(Int, 1), Token::new(Unknown, 1)]
);

tokenize_test!(
    it_tokenizes_a_number_with_expontent,
    "0E000",
    [Token::new(Float, 5)]
);

tokenize_test!(
    it_tokenizes_a_number_with_negative_expontent,
    "1.125e-5",
    [Token::new(Float, 8)]
);

tokenize_test!(
    it_tokenizes_a_number_with_positive_expontent,
    "-5e+20",
    [Token::new(Float, 6)]
);

tokenize_test!(
    it_tokenizes_a_number_with_lone_expontent,
    "-0.12E",
    [Token::new(Float, 5), Token::new(Ident, 1)]
);

tokenize_test!(
    it_tokenizes_a_number_with_decimal_expontent,
    "12.0e1.0",
    [
        Token::new(Float, 6),
        Token::new(Unknown, 1),
        Token::new(Int, 1)
    ]
);

// String literal tests.

tokenize_test!(
    it_tokenizes_the_empty_string,
    "\"\"",
    [Token::new(Str { terminated: true }, 2)]
);

tokenize_test!(
    it_tokenizes_a_string_with_linefeed,
    "\"\n\"",
    [Token::new(Str { terminated: true }, 3)]
);

tokenize_test!(
    it_tokenizes_a_string_with_carriage_return,
    "\"\r\"",
    [Token::new(Str { terminated: true }, 3)]
);

tokenize_test!(
    it_tokenizes_a_string_with_an_escaped_quote,
    "\"\\\"\"",
    [Token::new(Str { terminated: true }, 4)]
);

tokenize_test!(
    it_tokenizes_a_string_with_an_unescaped_quote,
    "\"\"\"",
    [
        Token::new(Str { terminated: true }, 2),
        Token::new(Str { terminated: false }, 1)
    ]
);

tokenize_test!(
    it_tokenizes_a_luxembourgish_flag,
    // is actually two valid unicode characters under the hood
    "🇱🇺",
    [Token::new(Unknown, 4), Token::new(Unknown, 4)]
);
