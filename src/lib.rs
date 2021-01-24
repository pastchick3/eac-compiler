mod asm;
mod ir;
mod parser;
mod reg_alloc;
mod serializer;
mod ssa;
mod x64;

use asm::X64Builder;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "parser")]
pub struct Opt {
    #[structopt(parse(from_os_str))]
    pub input: PathBuf,

    #[structopt(long)]
    ast: bool,

    #[structopt(long)]
    ssa: bool,

    #[structopt(long)]
    cfg: bool,

    #[structopt(long)]
    vasm: bool,

    #[structopt(long)]
    asm: bool,
}

pub fn compile(source: &str, opt: Opt) -> Option<String> {
    let ast = parser::parse(source);
    if opt.ast {
        println!("{:#?}", ast);
        return None;
    }
    let ssa = ssa::construct(ast);
    if opt.ssa {
        println!("{:#?}", ssa);
        return None;
    }
    let cfg = ssa::destruct(ssa);
    if opt.cfg {
        println!("{:#?}", cfg);
        return None;
    }
    let vasm = X64Builder::new().build(cfg);
    if opt.vasm {
        println!("{:#?}", vasm);
        return None;
    }
    let asm = reg_alloc::alloc(vasm);
    if opt.asm {
        println!("{:#?}", asm);
        return None;
    }
    Some(serializer::run(asm))
}
