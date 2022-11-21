use std::iter::Peekable;

use crate::ast::Node;
use crate::lexer::{
    token,
    token::{Span, StrError},
    Token, TokenKind, Tokenizer,
};

use self::ParseErrorKind::*;

#[derive(Debug, PartialEq)]
pub enum ParseErrorKind {
    UnexpectedContinuation(TokenKind),
    UnexpectedEof,
    UnexpectedToken(TokenKind),
    InvalidStr(StrError),
    InvalidIdent(String),
    UnknownToken(String),
}

#[derive(Debug, PartialEq)]
pub struct ParseError {
    pub kind: ParseErrorKind,
    pub span: Span,
}

impl ParseError {
    fn new(kind: ParseErrorKind, span: Span) -> Self {
        Self { kind, span }
    }

    fn unexpected_eof(input: &str) -> Self {
        let eof = input.len();
        Self {
            kind: UnexpectedEof,
            span: Span::new(eof, eof),
        }
    }

    fn unexpected_continuation(token: Token) -> Self {
        Self::new(UnexpectedContinuation(token.kind), token.span)
    }

    fn from_token(token: Token) -> Self {
        match token.kind {
            token::InvalidStr(err, offset) => {
                let loc = token.span.base + offset;
                let span = Span::new(loc, loc);
                Self::new(InvalidStr(err), span)
            }
            token::InvalidIdent(ident) => Self::new(InvalidIdent(ident), token.span),
            token::Unknown(unk) => Self::new(UnknownToken(unk), token.span),
            token::Eof => Self::new(UnexpectedEof, token.span),
            _ => Self::new(UnexpectedToken(token.kind), token.span),
        }
    }
}

pub fn parse(input: &str) -> Result<Node, ParseError> {
    let mut parser = Parser::new(input);
    let res = parser.parse();
    res
}

pub struct Parser<'a> {
    input: &'a str,
    tokenizer: Peekable<Tokenizer<'a>>,
}

impl<'a> Parser<'a> {
    /// Create a new parser.
    fn new(input: &'a str) -> Self {
        let tokenizer = Tokenizer::new(input).peekable();
        Self { input, tokenizer }
    }

    fn parse(&mut self) -> Result<Node, ParseError> {
        let node = self.json()?;
        self.end()?;
        Ok(node)
    }

    fn json(&mut self) -> Result<Node, ParseError> {
        self.value()
    }

    fn value(&mut self) -> Result<Node, ParseError> {
        let token = self.peek()?;
        let node = match token.kind {
            token::OpenBracket => self.object()?,
            token::OpenSquare => self.array()?,
            token::Str(_) => self.string()?,
            token::Int(_) => self.integer()?,
            token::Float(_) => self.float()?,
            token::True => self.ident_true()?,
            token::False => self.ident_false()?,
            token::Null => self.ident_null()?,
            _ => return Err(ParseError::from_token(token.clone())),
        };
        Ok(node)
    }

    fn object(&mut self) -> Result<Node, ParseError> {
        self.eat_open_bracket()?;
        let token = self.peek()?;
        let items: Vec<(String, Node)> = match token.kind {
            token::CloseBracket => vec![],
            _ => self.members()?,
        };
        self.eat_close_bracket()?;
        Ok(Node::Object(items))
    }

    fn members(&mut self) -> Result<Vec<(String, Node)>, ParseError> {
        let mut members = vec![self.member()?];
        loop {
            let token = self.peek()?;
            match token.kind {
                token::CloseBracket => {
                    break;
                }
                _ => {
                    self.eat_comma()?;
                    let next_member = self.member()?;
                    members.push(next_member);
                }
            };
        }
        Ok(members)
    }

    fn member(&mut self) -> Result<(String, Node), ParseError> {
        let token = self.next()?;
        let key = match token.kind {
            token::Str(s) => s,
            _ => return Err(ParseError::from_token(token)),
        };
        self.eat_colon()?;
        let value = self.value()?;
        Ok((key, value))
    }

    fn array(&mut self) -> Result<Node, ParseError> {
        self.eat_open_square()?;
        let token = self.peek()?;
        let items: Vec<Node> = match token.kind {
            token::CloseBracket => vec![],
            _ => self.elements()?,
        };
        self.eat_close_square()?;
        Ok(Node::Array(items))
    }

    fn elements(&mut self) -> Result<Vec<Node>, ParseError> {
        let mut elements = vec![self.value()?];
        loop {
            let token = self.peek()?;
            match token.kind {
                token::CloseSquare => {
                    break;
                }
                _ => {
                    self.eat_comma()?;
                    let next_element = self.value()?;
                    elements.push(next_element);
                }
            };
        }
        Ok(elements)
    }

    fn string(&mut self) -> Result<Node, ParseError> {
        let token = self.next()?;
        match token.kind {
            token::Str(s) => Ok(Node::Str(s)),
            _ => Err(ParseError::from_token(token)),
        }
    }

    fn integer(&mut self) -> Result<Node, ParseError> {
        let token = self.next()?;
        match token.kind {
            token::Int(i) => Ok(Node::Int(i)),
            _ => Err(ParseError::from_token(token)),
        }
    }

    fn float(&mut self) -> Result<Node, ParseError> {
        let token = self.next()?;
        match token.kind {
            token::Float(i) => Ok(Node::Float(i)),
            _ => Err(ParseError::from_token(token)),
        }
    }

    fn ident_true(&mut self) -> Result<Node, ParseError> {
        let token = self.next()?;
        match token.kind {
            token::True => Ok(Node::True),
            _ => Err(ParseError::from_token(token)),
        }
    }

    fn ident_false(&mut self) -> Result<Node, ParseError> {
        let token = self.next()?;
        match token.kind {
            token::False => Ok(Node::False),
            _ => Err(ParseError::from_token(token)),
        }
    }

    fn ident_null(&mut self) -> Result<Node, ParseError> {
        let token = self.next()?;
        match token.kind {
            token::Null => Ok(Node::Null),
            _ => Err(ParseError::from_token(token)),
        }
    }

    fn eat_open_bracket(&mut self) -> Result<(), ParseError> {
        let token = self.next()?;
        match token.kind {
            token::OpenBracket => Ok(()),
            _ => Err(ParseError::from_token(token)),
        }
    }

    fn eat_close_bracket(&mut self) -> Result<(), ParseError> {
        let token = self.next()?;
        match token.kind {
            token::CloseBracket => Ok(()),
            _ => Err(ParseError::from_token(token)),
        }
    }

    fn eat_open_square(&mut self) -> Result<(), ParseError> {
        let token = self.next()?;
        match token.kind {
            token::OpenSquare => Ok(()),
            _ => Err(ParseError::from_token(token)),
        }
    }

    fn eat_close_square(&mut self) -> Result<(), ParseError> {
        let token = self.next()?;
        match token.kind {
            token::CloseSquare => Ok(()),
            _ => Err(ParseError::from_token(token)),
        }
    }

    fn eat_colon(&mut self) -> Result<(), ParseError> {
        let token = self.next()?;
        match token.kind {
            token::Colon => Ok(()),
            _ => Err(ParseError::from_token(token)),
        }
    }

    fn eat_comma(&mut self) -> Result<(), ParseError> {
        let token = self.next()?;
        match token.kind {
            token::Comma => Ok(()),
            _ => Err(ParseError::from_token(token)),
        }
    }

    /// Peek at the next token.
    fn peek(&mut self) -> Result<&Token, ParseError> {
        match self.tokenizer.peek() {
            Some((token, _)) => Ok(token),
            None => Err(ParseError::unexpected_eof(&self.input)),
        }
    }

    /// Get the next token, moving the index along one.
    fn next(&mut self) -> Result<Token, ParseError> {
        match self.tokenizer.next() {
            Some((token, _)) => Ok(token),
            None => Err(ParseError::unexpected_eof(&self.input)),
        }
    }

    fn end(&mut self) -> Result<(), ParseError> {
        match self.tokenizer.next() {
            None => Ok(()),
            Some((token, _)) => Err(ParseError::unexpected_continuation(token)),
        }
    }
}
