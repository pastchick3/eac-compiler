mod asm;
mod ir;
mod parser;
mod ssa;
mod x64;

use asm::X64Builder;

pub fn compile(source: &str) -> Vec<u8> {
    let ast = parser::parse(source);
    let ssa = ssa::construct(ast);
    let cfg = ssa::destruct(ssa);
    let asm = X64Builder::new().build(cfg);
    println!("{:#?}", asm);
    Vec::new()
}
