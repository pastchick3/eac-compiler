use std::path::PathBuf;
use structopt::StructOpt;
use eac_compiler::Compiler;


#[derive(StructOpt)]
#[structopt(name = "parser")]
struct Opt {
    #[structopt(parse(from_os_str))]
    input: PathBuf,

    #[structopt(short, long, parse(from_os_str), default_value = "./program.exe")]
    output: PathBuf,
}

fn main() {
    let opt = Opt::from_args();
    let compiler = Compiler::new();
    compiler.run(&opt.input, &opt.output);
}
