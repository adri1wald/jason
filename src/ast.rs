#[derive(Debug, PartialEq, PartialOrd)]
pub enum Node {
    Object(Vec<(String, Node)>),
    Array(Vec<Node>),
    Str(String),
    Int(isize),
    Float(f64),
    True,
    False,
    Null,
}
