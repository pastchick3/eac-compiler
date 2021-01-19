mod ir;
mod parser;
mod ssa;
mod asm;
mod x64;

pub fn compile(source: &str) -> Vec<u8> {
    let ast = parser::parse(source);
    let ssa = ssa::construct(ast);
    let cfg = ssa::destruct(ssa);
    let asm = asm::build(cfg);
    println!("{:#?}", asm);
    Vec::new()
}
