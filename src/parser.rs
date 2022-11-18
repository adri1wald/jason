use crate::ast::Node;

pub struct Parser<'a> {
    string: &'a str,
}

impl<'a> Parser<'a> {
    pub fn new(string: &'a str) -> Self {
        return Parser { string };
    }

    pub fn parse(self) -> Node {
        self.program()
    }

    /// program
    ///   : int_literal
    ///   ;
    fn program(self) -> Node {
        self.int_literal()
    }

    /// int_literal
    ///   : INT
    ///   ;
    fn int_literal(self) -> Node {
        Node::Int(self.string.parse::<i32>().unwrap())
    }
}
