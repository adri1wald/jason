pub mod ast;
pub mod lexer;
pub mod parser;

pub use parser::parse;

#[cfg(test)]
mod tests {
    use super::ast::Node;
    use super::parser::parse;

    #[test]
    fn it_parses() {
        let input = "[{}]";
        let node = parse(input);
        let expected = Node::Array(vec![Node::Object(vec![])]);
        assert_eq!(node, Ok(expected));
    }
}
