mod asm;
mod ir;
mod parser;
mod reg_alloc;
mod serializer;
mod ssa;
mod x64;

use asm::X64Builder;

pub fn compile(source: &str) -> String {
    let ast = parser::parse(source);
    let ssa = ssa::construct(ast);
    let cfg = ssa::destruct(ssa);
    let asm = X64Builder::new().build(cfg);
    let asm = reg_alloc::alloc(asm);
    serializer::run(asm)
}
