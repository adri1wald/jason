extern crate jason;

use jason::parse;

fn main() {
    let input = "[{}]{";
    let res = parse(input);
    println!("{:?}", res);
}
