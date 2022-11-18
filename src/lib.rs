pub mod parser;
pub mod ast;

#[cfg(test)]
mod tests {
    use super::parser::Parser;
    use super::ast::Node;

    #[test]
    fn parse_int() {
        let string = "42";
        let parser = Parser::new(&string);
        let ast = parser.parse();
        assert_eq!(ast, Node::Int(42));
    }
}
