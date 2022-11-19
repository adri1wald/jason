use anyhow::{bail, Result};
use std::{io::ErrorKind, str};

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Integer(usize),
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
}

impl From<usize> for TokenKind {
    fn from(other: usize) -> TokenKind {
        TokenKind::Integer(other)
    }
}

impl From<f64> for TokenKind {
    fn from(other: f64) -> TokenKind {
        TokenKind::Decimal(other)
    }
}

impl From<&str> for TokenKind {
    fn from(other: &str) -> TokenKind {
        TokenKind::QuotedString(other.into())
    }
}

pub fn tokenize(data: &str) -> Result<Vec<(TokenKind, usize, usize)>> {
    let mut tokenizer = Tokenizer::new(data);
    let mut tokens = Vec::new();

    while let Some(tok) = tokenizer.next_token()? {
        tokens.push(tok);
    }

    Ok(tokens)
}

struct Tokenizer<'a> {
    current_index: usize,
    remaining_data: &'a str,
}

impl<'a> Tokenizer<'a> {
    fn new(data: &'a str) -> Self {
        Self {
            current_index: 0,
            remaining_data: data,
        }
    }

    fn next_token(&mut self) -> Result<Option<(TokenKind, usize, usize)>> {
        self.skip_whitespace();

        if self.remaining_data.is_empty() {
            Ok(None)
        } else {
            let start = self.current_index;
            let tok = self._next_token()?;
            let end = self.current_index;
            Ok(Some((tok, start, end)))
        }
    }

    fn skip_whitespace(&mut self) {
        let skipped = skip_whitespace(self.remaining_data);
        self.chomp(skipped);
    }

    fn _next_token(&mut self) -> Result<TokenKind> {
        let (tok, bytes_read) = tokenize_single_token(self.remaining_data)?;
        self.chomp(bytes_read);
        Ok(tok)
    }

    fn chomp(&mut self, num_bytes: usize) {
        self.remaining_data = &self.remaining_data[num_bytes..];
        self.current_index += num_bytes;
    }
}

fn tokenize_single_token(data: &str) -> Result<(TokenKind, usize)> {
    let next = match data.chars().next() {
        Some(c) => c,
        None => bail!(ErrorKind::UnexpectedEof),
    };

    let (tok, length) = match next {
        '{' => (TokenKind::OpenBracket, 1),
        '}' => (TokenKind::CloseBracket, 1),
        '[' => (TokenKind::OpenSquare, 1),
        ']' => (TokenKind::CloseSquare, 1),
        ':' => (TokenKind::Colon, 1),
        ',' => (TokenKind::Comma, 1),
        '0'..='9' => tokenize_number(data)?,
        '"' => tokenize_quoted_string(data)?,
        _ => tokenize_literals(data)?,
    };

    Ok((tok, length))
}

fn tokenize_number(data: &str) -> Result<(TokenKind, usize)> {
    let mut seen_dot = false;

    let (decimal, bytes_read) = take_while(data, |c| {
        if c.is_digit(10) {
            true
        } else if c == '.' {
            if !seen_dot {
                seen_dot = true;
                true
            } else {
                false
            }
        } else {
            false
        }
    })?;

    if seen_dot {
        let n: f64 = decimal.parse()?;
        Ok((n.into(), bytes_read))
    } else {
        let n: usize = decimal.parse()?;
        Ok((n.into(), bytes_read))
    }
}

fn tokenize_quoted_string(data: &str) -> Result<(TokenKind, usize)> {
    todo!()
}

fn tokenize_literals(data: &str) -> Result<(TokenKind, usize)> {
    if data.starts_with("null") {
        Ok((TokenKind::Null, 4))
    } else if data.starts_with("true") {
        Ok((TokenKind::True, 4))
    } else if data.starts_with("false") {
        Ok((TokenKind::False, 5))
    } else {
        bail!("Unexpected character {:?}", data.chars().next())
    }
}

fn skip_whitespace(data: &str) -> usize {
    match take_while(data, |ch| ch.is_whitespace()) {
        Ok((_, bytes_skipped)) => bytes_skipped,
        _ => 0,
    }
}

fn take_while<F>(data: &str, mut pred: F) -> Result<(&str, usize)>
where
    F: FnMut(char) -> bool,
{
    let mut current_index = 0;

    for ch in data.chars() {
        let should_continue = pred(ch);

        if !should_continue {
            break;
        }

        current_index += ch.len_utf8();
    }

    if current_index == 0 {
        bail!("No Matches")
    } else {
        Ok((&data[..current_index], current_index))
    }
}

macro_rules! lexer_test {
    (FAIL: $name:ident, $func:ident, $src:expr) => {
        #[cfg(test)]
        #[test]
        fn $name() {
            let src: &str = $src;
            let func = $func;

            let got = func(src);
            assert!(got.is_err(), "{:?} should be an error", got);
        }
    };
    ($name:ident, $func:ident, $src:expr => $should_be:expr) => {
        #[cfg(test)]
        #[test]
        fn $name() {
            let src: &str = $src;
            let should_be = TokenKind::from($should_be);
            let func = $func;

            let (got, _bytes_read) = func(src).unwrap();
            assert_eq!(got, should_be, "Input was {:?}", src);
        }
    };
}

lexer_test!(central_tokenizer_integer, tokenize_single_token, "1234" => 1234);
lexer_test!(central_tokenizer_decimal, tokenize_single_token, "420.69" => 420.69);
lexer_test!(central_tokenizer_open_bracket, tokenize_single_token, "{" => TokenKind::OpenBracket);
lexer_test!(central_tokenizer_close_bracket, tokenize_single_token, "}" => TokenKind::CloseBracket);
lexer_test!(central_tokenizer_open_square, tokenize_single_token, "[" => TokenKind::OpenSquare);
lexer_test!(central_tokenizer_close_square, tokenize_single_token, "]" => TokenKind::CloseSquare);
lexer_test!(central_tokenizer_colon, tokenize_single_token, ":" => TokenKind::Colon);
lexer_test!(central_tokenizer_comma, tokenize_single_token, "," => TokenKind::Comma);
lexer_test!(central_tokenizer_null, tokenize_single_token, "null" => TokenKind::Null);
lexer_test!(central_tokenizer_true, tokenize_single_token, "true" => TokenKind::True);
lexer_test!(central_tokenizer_false, tokenize_single_token, "false" => TokenKind::False);

#[test]
fn it_skips_past_several_whitespace_chars() {
    let data = " \t\n\r123";
    let should_be = 4;
    let num_skipped = skip_whitespace(data);
    assert_eq!(num_skipped, should_be);
}

#[test]
fn it_does_not_skip_when_first_is_not_whitespace() {
    let data = "Hello World";
    let should_be = 0;
    let num_skipped = skip_whitespace(data);
    assert_eq!(num_skipped, should_be);
}
