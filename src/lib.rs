mod ast;
mod parser;

use crate::parser::parse;

pub struct Compiler {}

impl Compiler {
    pub fn new() -> Self {
        Compiler {}
    }

    pub fn run(&self, source: &str) -> Vec<u8> {
        let ast = parse(source);
        println!("{:#?}", ast);
        Vec::new()
    }
}
