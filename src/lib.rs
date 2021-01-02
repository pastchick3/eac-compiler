mod parser;
mod ast;

use std::path::Path;
use crate::parser::parse;

pub struct Compiler {
    
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {}
    }

    pub fn run(&self, input: &Path, output: &Path) {
        let ast = unsafe { parse(input) };
        println!("{:#?}", ast);
    }
}
