use std::{ops::Range, str::Chars};

/// Errors and warnings that can occur during string unescaping.
#[derive(Debug, PartialEq, Eq)]
pub enum EscapeError {
    /// Escaped '\' character without continuation. (Unterminated?)
    LoneSlash,
    /// Invalid escape character (e.g. '\z').
    InvalidEscape,
    /// Raw '\b' encountered.
    BareBackspace,
    /// Raw '\f' encountered.
    BareFormFeed,
    /// Raw '\n' encountered.
    BareLineFeed,
    /// Raw '\r' encountered.
    BareCarriageReturn,
    /// Raw '\r' encountered.
    BareHorizontalTab,
    /// Bad control character encountered.
    BadControlChar,
    /// Unescaped character that was expected to be escaped (e.g. raw '\t').
    EscapeOnlyChar,
    /// Bad unicode escape
    BadUnicodeEscape,
    /// Invalid in-bound unicode character code, e.g. '\u{DFFF}'.
    LoneSurrogateUnicodeEscape,
    /// Out of bounds unicode character code, e.g. '\u{FFFFFF}'.
    OutOfRangeUnicodeEscape,
}

fn scan_escape(chars: &mut Chars<'_>) -> Result<char, EscapeError> {
    // Previous character was '\\', unescape what follows.
    let res = match chars.next().ok_or(EscapeError::LoneSlash)? {
        '"' => '"',
        '\\' => '\\',
        '/' => '/',
        'b' => '\u{0008}',
        'f' => '\u{000C}',
        'n' => '\n',
        'r' => '\r',
        't' => '\t',

        'u' => {
            // We've parsed '\u', now we have to parse 'xxxx'.

            // First character must be a hexadecimal digit.
            let mut n_digits = 1;
            let mut value: u32 = chars
                .next()
                .ok_or(EscapeError::BadUnicodeEscape)?
                .to_digit(16)
                .ok_or(EscapeError::BadUnicodeEscape)?;

            // First character is valid, now parse the rest of the number
            // and closing brace.
            loop {
                match chars.next() {
                    None => return Err(EscapeError::BadUnicodeEscape),
                    Some(c) => {
                        let digit = c.to_digit(16).ok_or(EscapeError::BadUnicodeEscape)?;
                        n_digits += 1;
                        let digit = digit as u32;
                        value = value * 16 + digit;
                        if n_digits < 4 {
                            continue;
                        }
                        break std::char::from_u32(value).ok_or_else(|| {
                            if value > 0x10FFFF {
                                EscapeError::OutOfRangeUnicodeEscape
                            } else {
                                EscapeError::LoneSurrogateUnicodeEscape
                            }
                        })?;
                    }
                };
            }
        }
        _ => return Err(EscapeError::InvalidEscape),
    };
    Ok(res)
}

fn iter_unescape_string(
    input: &str,
) -> impl Iterator<Item = (Range<usize>, Result<char, EscapeError>)> + '_ {
    let mut chars = input.chars();
    std::iter::from_fn(move || {
        if let Some(c) = chars.next() {
            let start = input.len() - chars.as_str().len() - c.len_utf8();
            let res = match c {
                '\\' => scan_escape(&mut chars),
                '"' => Err(EscapeError::EscapeOnlyChar),
                '\u{0008}' => Err(EscapeError::BareBackspace),
                '\u{000C}' => Err(EscapeError::BareFormFeed),
                '\n' => Err(EscapeError::BareLineFeed),
                '\r' => Err(EscapeError::BareCarriageReturn),
                '\t' => Err(EscapeError::BareHorizontalTab),
                c if c.is_control() => Err(EscapeError::BadControlChar),
                _ => Ok(c),
            };
            let end = input.len() - chars.as_str().len();
            Some((start..end, res))
        } else {
            None
        }
    })
}

// pub fn unescape_string_with_cb<F>(input: &str, callback: &mut F)
// where
//     F: FnMut(Range<usize>, Result<char, EscapeError>),
// {
//     let mut unescape_iter = iter_unescape_string(input);
//     while let Some((range, res)) = unescape_iter.next() {
//         callback(range, res);
//     }
// }

pub fn unescape_string(input: &str) -> Result<String, (EscapeError, Range<usize>)> {
    let result: Result<String, _> = iter_unescape_string(input)
        .map(|(range, res)| match res {
            Ok(c) => Ok(c),
            Err(e) => Err((e, range)),
        })
        .collect();
    result
}

// Tests.

macro_rules! unescape_test {
    (FAIL: $name:ident, $input:expr, $should_be:expr) => {
        #[cfg(test)]
        #[test]
        fn $name() {
            let output = unescape_string($input);
            assert_eq!(output, Err($should_be));
        }
    };
    ($name:ident, $input:expr, $should_be:expr) => {
        #[cfg(test)]
        #[test]
        fn $name() {
            let output = unescape_string($input);
            assert_eq!(output, Ok($should_be.into()));
        }
    };
}

// Succeed.

unescape_test!(it_unescapes_empty_string, "", "");

unescape_test!(
    it_unescapes_string_without_escapes,
    "transformer",
    "transformer"
);

unescape_test!(it_unescapes_string_with_escaped_double_quote, "\\\"", "\"");

unescape_test!(
    it_unescapes_string_with_escaped_backspace,
    "\\b",
    "\u{0008}"
);

unescape_test!(
    it_unescapes_string_with_escaped_form_feed,
    "\\f",
    "\u{000C}"
);

unescape_test!(it_unescapes_string_with_escaped_line_feed, "\\n", "\n");

unescape_test!(
    it_unescapes_string_with_escaped_carriage_return,
    "\\r",
    "\r"
);

unescape_test!(it_unescapes_string_with_escaped_horizontal_tab, "\\t", "\t");

unescape_test!(it_unescapes_string_with_single_quote, "\'", "'");

unescape_test!(it_unescapes_string_with_emojis, "😀❤️‍🔥👬", "😀❤️‍🔥👬");

unescape_test!(
    it_unescapes_string_with_encoded_unicodes,
    "\\u2665\\uFE0F",
    "♥️"
);

// Fail.

unescape_test!(
    FAIL: it_fails_unescape_with_bare_backspace,
    "\u{0008}",
    (EscapeError::BareBackspace, 0..1)
);

unescape_test!(
    FAIL: it_fails_unescape_with_bare_form_feed,
    "\u{000C}",
    (EscapeError::BareFormFeed, 0..1)
);

unescape_test!(
    FAIL: it_fails_unescape_with_bare_line_feed,
    "\n",
    (EscapeError::BareLineFeed, 0..1)
);

unescape_test!(
    FAIL: it_fails_unescape_with_bare_carriage_return,
    "\r",
    (EscapeError::BareCarriageReturn, 0..1)
);

unescape_test!(
    FAIL: it_fails_unescape_with_bare_horizontal_tab,
    "\t",
    (EscapeError::BareHorizontalTab, 0..1)
);

unescape_test!(
    FAIL: it_fails_unescape_with_bare_double_quote,
    "\"",
    (EscapeError::EscapeOnlyChar, 0..1)
);

unescape_test!(
    FAIL: it_fails_unescape_with_escaped_single_quote,
    "\\\'",
    (EscapeError::InvalidEscape, 0..2)
);

unescape_test!(
    FAIL: it_fails_unescape_with_bad_control_char,
    "\0",
    (EscapeError::BadControlChar, 0..1)
);

unescape_test!(
    FAIL: it_fails_unescape_with_lone_slash,
    "hello\\",
    (EscapeError::LoneSlash, 5..6)
);

unescape_test!(
    FAIL: it_fails_unescape_with_bad_unicode_escape,
    "\\u",
    (EscapeError::BadUnicodeEscape, 0..2)
);

// TODO: FIXME: may want to succeed here with '\u{FFFD}'
unescape_test!(
    FAIL: it_fails_unescape_with_lone_surrogate_unicode_escape,
    "\\uDFFF",
    (EscapeError::LoneSurrogateUnicodeEscape, 0..6)
);
