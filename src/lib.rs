mod ir;
mod parser;
mod ssa;

pub fn compile(source: &str) -> Vec<u8> {
    let ast = parser::parse(source);
    let ssa = ssa::construct(ast);
    let cfg = ssa::destruct(ssa);
    println!("{:#?}", cfg);
    Vec::new()
}
