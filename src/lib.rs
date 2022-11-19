pub mod ast;
pub mod parser;
pub mod tokenizer;

#[cfg(test)]
mod tests {
    use super::tokenizer::{tokenize, TokenKind};

    #[test]
    fn tokenize_array() {
        let data = " [  1,  2, 3, 4 ] ";
        let tokens = tokenize(data);
        if let Ok(tokens) = tokens {
            let tokens: Vec<_> = tokens.iter().map(|(tok, _, _)| tok).collect();
            let expected_tokens = vec![
                &TokenKind::OpenSquare,
                &TokenKind::Integer(1),
                &TokenKind::Comma,
                &TokenKind::Integer(2),
                &TokenKind::Comma,
                &TokenKind::Integer(3),
                &TokenKind::Comma,
                &TokenKind::Integer(4),
                &TokenKind::CloseSquare,
            ];
            assert_eq!(tokens, expected_tokens);
        }
    }
}
