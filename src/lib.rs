mod ir;
mod parser;
mod ssa;

pub fn compile(source: &str) -> Vec<u8> {
    let ast = parser::parse(source);
    let ssa = ssa::build_ssa(ast);
    println!("{:#?}", ssa);
    Vec::new()
}
