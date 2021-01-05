mod ast;
mod parser;

use crate::parser::parse;

pub fn compile(source: &str) -> Vec<u8> {
    let ast = parse(source);
    Vec::new()
}
