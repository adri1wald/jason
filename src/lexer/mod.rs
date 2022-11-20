mod cursor;

pub use cursor::Cursor;

use self::TokenKind::*;

#[derive(Debug, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub len: u32,
}

impl Token {
    fn new(kind: TokenKind, len: u32) -> Self {
        Self { kind, len }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Integer,
    Decimal,
    QuotedString {
        terminated: bool,
    },
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
    Unknown,
}

/// Creates an iterator that produces tokens from the input string
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

impl Cursor<'_> {
    pub fn advance_token(&mut self) -> Token {
        let first_char = match self.bump() {
            Some(c) => c,
            None => return Token::new(Eof, 0),
        };
        let token_kind = match first_char {
            // Whitespace sequence.
            c if c.is_whitespace() => self.whitespace(),

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
                QuotedString { terminated }
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
                return Integer;
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
                                Decimal
                            }
                            '+' | '-' => match self.third() {
                                '0'..='9' => {
                                    self.bump();
                                    self.bump();
                                    self.eat_decimal_digits();
                                    Decimal
                                }
                                _ => Decimal,
                            },
                            _ => Decimal,
                        },
                        _ => Decimal,
                    }
                }
                _ => Integer,
            },
            'e' | 'E' => match self.second() {
                '0'..='9' => {
                    self.bump();
                    self.eat_decimal_digits();
                    Decimal
                }
                '+' | '-' => match self.third() {
                    '0'..='9' => {
                        self.bump();
                        self.bump();
                        self.eat_decimal_digits();
                        Decimal
                    }
                    _ => Integer,
                },
                _ => Integer,
            },
            _ => Integer,
        }
    }

    fn double_quoted_string(&mut self) -> bool {
        debug_assert!(self.prev() == '"');
        while let Some(c) = self.bump() {
            match c {
                '"' => {
                    return true;
                }
                '\n' => {
                    return false;
                }
                '\r' => {
                    return false;
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
            let mut token_iterator = tokenize($input);

            for token in $tokens {
                assert_eq!(token_iterator.next(), Some(token));
            }

            assert_eq!(token_iterator.next(), None);
        }
    };
}

// Numeric literal tests.

tokenize_test!(it_tokenizes_an_integer, "420", [Token::new(Integer, 3)]);

tokenize_test!(
    it_tokenizes_a_negative_integer,
    "-1600",
    [Token::new(Integer, 5)]
);

tokenize_test!(
    it_tokenizes_an_integer_with_space_around,
    " 69\n \r",
    [
        Token::new(Whitespace, 1),
        Token::new(Integer, 2),
        Token::new(Whitespace, 3)
    ]
);

tokenize_test!(it_tokenizes_a_decimal, "3.14", [Token::new(Decimal, 4)]);

tokenize_test!(
    it_tokenizes_a_negative_decimal,
    "-0.618",
    [Token::new(Decimal, 6)]
);

tokenize_test!(
    it_tokenizes_two_zeros,
    "00",
    [Token::new(Integer, 1), Token::new(Integer, 1)]
);

tokenize_test!(
    it_tokenizes_a_integer_with_lone_period,
    "1.",
    [Token::new(Integer, 1), Token::new(Unknown, 1)]
);

tokenize_test!(
    it_tokenizes_a_number_with_expontent,
    "0E000",
    [Token::new(Decimal, 5)]
);

tokenize_test!(
    it_tokenizes_a_number_with_negative_expontent,
    "1.125e-5",
    [Token::new(Decimal, 8)]
);

tokenize_test!(
    it_tokenizes_a_number_with_positive_expontent,
    "-5e+20",
    [Token::new(Decimal, 6)]
);

tokenize_test!(
    it_tokenizes_a_number_with_lone_expontent,
    "-0.12E",
    [Token::new(Decimal, 5), Token::new(Unknown, 1)]
);

tokenize_test!(
    it_tokenizes_a_number_with_decimal_expontent,
    "12.0e1.0",
    [
        Token::new(Decimal, 6),
        Token::new(Unknown, 1),
        Token::new(Integer, 1)
    ]
);

// String literal tests.

tokenize_test!(
    it_tokenizes_the_empty_string,
    "\"\"",
    [Token::new(QuotedString { terminated: true }, 2)]
);

tokenize_test!(
    it_tokenizes_a_string_with_linefeed,
    "\"\n\"",
    [
        Token::new(QuotedString { terminated: false }, 2),
        Token::new(QuotedString { terminated: false }, 1)
    ]
);

tokenize_test!(
    it_tokenizes_a_string_with_carriage_return,
    "\"\r\"",
    [
        Token::new(QuotedString { terminated: false }, 2),
        Token::new(QuotedString { terminated: false }, 1)
    ]
);

tokenize_test!(
    it_tokenizes_a_string_with_an_escaped_quote,
    "\"\\\"\"",
    [Token::new(QuotedString { terminated: true }, 4)]
);

tokenize_test!(
    it_tokenizes_a_string_with_an_unescaped_quote,
    "\"\"\"",
    [
        Token::new(QuotedString { terminated: true }, 2),
        Token::new(QuotedString { terminated: false }, 1)
    ]
);
