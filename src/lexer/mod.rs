mod base;
pub mod token;

use base::{unescape, Cursor};
pub use token::{Span, StrError, Token, TokenKind};

pub fn tokenize(input: &str) -> impl Iterator<Item = (Token, bool)> + '_ {
    let mut tokenizer = Tokenizer::new(input);

    std::iter::from_fn(move || {
        let (token, whitespace) = tokenizer.next_token();
        if token.kind != token::Eof {
            Some((token, whitespace))
        } else {
            None
        }
    })
}

pub struct Tokenizer<'a> {
    pos: usize,
    input: &'a str,
    cursor: Cursor<'a>,
}

impl<'a> Tokenizer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            pos: 0,
            input,
            cursor: Cursor::new(&input),
        }
    }

    /// Returns the next token, paired with a bool indicating if the token was
    /// preceded by whitespace.
    pub fn next_token(&mut self) -> (Token, bool) {
        let mut preceded_by_whitespace = false;

        loop {
            let token = self.cursor.advance_token();
            let start = self.pos;
            self.pos = self.pos + token.len;

            let kind = match token.kind {
                // Whitespace: skip.
                base::TokenKind::Whitespace => {
                    preceded_by_whitespace = true;
                    continue;
                }

                // Identifier.
                base::TokenKind::Ident => self.cook_base_ident(start),

                // Integer.
                base::TokenKind::Int => self.cook_base_integer(start),

                // Float.
                base::TokenKind::Float => self.cook_base_decimal(start),

                // Quoted string.
                base::TokenKind::Str { terminated } => {
                    self.cook_base_quoted_string(start, terminated)
                }

                base::TokenKind::OpenBracket => token::OpenBracket,
                base::TokenKind::CloseBracket => token::CloseBracket,
                base::TokenKind::OpenSquare => token::OpenSquare,
                base::TokenKind::CloseSquare => token::CloseSquare,
                base::TokenKind::Colon => token::Colon,
                base::TokenKind::Comma => token::Comma,

                base::TokenKind::Unknown => self.cook_base_unknown(start),
                base::TokenKind::Eof => token::Eof,
            };
            let span = Span::new(start, self.pos);
            return (Token::new(kind, span), preceded_by_whitespace);
        }
    }

    fn cook_base_ident(&self, start: usize) -> TokenKind {
        let slice = self.str_from(start);
        match slice {
            "true" => token::True,
            "false" => token::False,
            "null" => token::Null,
            ident => token::InvalidIdent(ident.to_owned()),
        }
    }

    fn cook_base_integer(&self, start: usize) -> TokenKind {
        let slice = self.str_from(start);
        token::Int(slice.parse().unwrap())
    }

    fn cook_base_decimal(&self, start: usize) -> TokenKind {
        let slice = self.str_from(start);
        token::Float(slice.parse().unwrap())
    }

    fn cook_base_quoted_string(&self, start: usize, terminated: bool) -> TokenKind {
        if !terminated {
            return token::InvalidStr(StrError::Unterminated, self.pos);
        }
        let start = start + 1;
        let end = self.pos - 1;
        let slice = self.str_from_to(start, end);
        match unescape::unescape_string(slice) {
            Ok(s) => token::Str(s),
            Err((e, range)) => {
                // plus 1 because we unescape after first '\"'
                token::InvalidStr(e.into(), range.start + 1)
            }
        }
    }

    fn cook_base_unknown(&self, start: usize) -> TokenKind {
        let slice = self.str_from(start);
        token::Unknown(slice.to_owned())
    }

    fn str_from(&self, start: usize) -> &str {
        self.str_from_to(start, self.pos)
    }

    fn str_from_to(&self, start: usize, end: usize) -> &str {
        &self.input[start..end]
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = (Token, bool);

    fn next(&mut self) -> Option<Self::Item> {
        let (token, whitespace) = self.next_token();
        if matches!(token.kind, token::Eof) {
            None
        } else {
            Some((token, whitespace))
        }
    }
}

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

// Identifier tests.

tokenize_test!(
    it_tokenizes_true,
    "true",
    [(Token::new(token::True, Span::new(0, 4)), false)]
);

tokenize_test!(
    it_tokenizes_false,
    "false",
    [(Token::new(token::False, Span::new(0, 5)), false)]
);

tokenize_test!(
    it_tokenizes_null,
    "null",
    [(Token::new(token::Null, Span::new(0, 4)), false)]
);

tokenize_test!(
    it_tokenizes_invalid_ident,
    "potato",
    [(
        Token::new(token::InvalidIdent("potato".to_owned()), Span::new(0, 6)),
        false
    )]
);

// Numeric literal tests.

tokenize_test!(
    it_tokenizes_an_integer,
    "420",
    [(Token::new(token::Int(420), Span::new(0, 3)), false)]
);

tokenize_test!(
    it_tokenizes_an_integer_with_space_around,
    " 69\n \r",
    [(Token::new(token::Int(69), Span::new(1, 3)), true)]
);

tokenize_test!(
    it_tokenizes_a_decimal,
    "3.14",
    [(Token::new(token::Float(3.14), Span::new(0, 4)), false)]
);

tokenize_test!(
    it_tokenizes_a_negative_decimal,
    "-0.618",
    [(Token::new(token::Float(-0.618), Span::new(0, 6)), false)]
);

tokenize_test!(
    it_tokenizes_two_zeros,
    "00",
    [
        (Token::new(token::Int(0), Span::new(0, 1)), false),
        (Token::new(token::Int(0), Span::new(1, 2)), false)
    ]
);

tokenize_test!(
    it_tokenizes_a_integer_with_lone_period,
    "1.",
    [
        (Token::new(token::Int(1), Span::new(0, 1)), false),
        (
            Token::new(token::Unknown(".".to_owned()), Span::new(1, 2)),
            false
        )
    ]
);

tokenize_test!(
    it_tokenizes_a_number_with_expontent,
    "0E000",
    [(Token::new(token::Float(0f64), Span::new(0, 5)), false)]
);

tokenize_test!(
    it_tokenizes_a_number_with_negative_expontent,
    "1.125e-5",
    [(Token::new(token::Float(1.125e-5), Span::new(0, 8)), false)]
);

tokenize_test!(
    it_tokenizes_a_number_with_positive_expontent,
    "-5e+20",
    [(Token::new(token::Float(-5e+20), Span::new(0, 6)), false)]
);

tokenize_test!(
    it_tokenizes_a_number_with_lone_expontent,
    "-0.12E",
    [
        (Token::new(token::Float(-0.12), Span::new(0, 5)), false),
        (
            Token::new(token::InvalidIdent("E".to_owned()), Span::new(5, 6)),
            false
        )
    ]
);

tokenize_test!(
    it_tokenizes_a_number_with_decimal_expontent,
    "12.0e1.0",
    [
        (Token::new(token::Float(12.0e1), Span::new(0, 6)), false),
        (
            Token::new(token::Unknown(".".to_owned()), Span::new(6, 7)),
            false
        ),
        (Token::new(token::Int(0), Span::new(7, 8)), false)
    ]
);

// String literal tests.

tokenize_test!(
    it_tokenizes_the_empty_string,
    "\"\"",
    [(
        Token::new(token::Str("".to_owned()), Span::new(0, 2)),
        false
    )]
);

tokenize_test!(
    it_tokenizes_a_string_with_an_escaped_quote,
    "\"\\\"\"",
    [(
        Token::new(token::Str("\"".to_owned()), Span::new(0, 4)),
        false
    )]
);

tokenize_test!(
    it_tokenizes_an_unterminated_string,
    "\"",
    [(
        Token::new(
            token::InvalidStr(StrError::Unterminated, 1),
            Span::new(0, 1)
        ),
        false
    )]
);

tokenize_test!(
    it_tokenizes_another_unterminated_string,
    "\"\\\"",
    [(
        Token::new(
            token::InvalidStr(StrError::Unterminated, 3),
            Span::new(0, 3)
        ),
        false
    )]
);

tokenize_test!(
    it_tokenizes_a_string_with_bare_line_feed,
    "\"\n\"",
    [(
        Token::new(
            token::InvalidStr(StrError::BareLineFeed, 1),
            Span::new(0, 3)
        ),
        false
    )]
);

// Full tests.

tokenize_test!(
    it_tokenizes_an_array_of_objects,
    "[{ \"name\": \"Adrien\", \"age\": 23, \"hungry\": true, \"health\": 0.9, \"girlfriend\": null }]",
    [
        (Token::new(token::OpenSquare, Span::new(0, 1)), false),
        (Token::new(token::OpenBracket, Span::new(1, 2)), false),
        (Token::new(token::Str("name".into()), Span::new(3, 9)), true),
        (Token::new(token::Colon, Span::new(9, 10)), false),
        (Token::new(token::Str("Adrien".into()), Span::new(11, 19)), true),
        (Token::new(token::Comma, Span::new(19, 20)), false),
        (Token::new(token::Str("age".into()), Span::new(21, 26)), true),
        (Token::new(token::Colon, Span::new(26, 27)), false),
        (Token::new(token::Int(23), Span::new(28, 30)), true),
        (Token::new(token::Comma, Span::new(30, 31)), false),
        (Token::new(token::Str("hungry".into()), Span::new(32, 40)), true),
        (Token::new(token::Colon, Span::new(40, 41)), false),
        (Token::new(token::True, Span::new(42, 46)), true),
        (Token::new(token::Comma, Span::new(46, 47)), false),
        (Token::new(token::Str("health".into()), Span::new(48, 56)), true),
        (Token::new(token::Colon, Span::new(56, 57)), false),
        (Token::new(token::Float(0.9), Span::new(58, 61)), true),
        (Token::new(token::Comma, Span::new(61, 62)), false),
        (Token::new(token::Str("girlfriend".into()), Span::new(63, 75)), true),
        (Token::new(token::Colon, Span::new(75, 76)), false),
        (Token::new(token::Null, Span::new(77, 81)), true),
        (Token::new(token::CloseBracket, Span::new(82, 83)), true),
        (Token::new(token::CloseSquare, Span::new(83, 84)), false),
    ]
);
