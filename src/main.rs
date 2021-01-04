use eac_compiler::Compiler;
use std::fs;
use std::path::PathBuf;
use structopt::StructOpt;

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
    let source = fs::read_to_string(opt.input).expect("Invalid input file path.");
    let binary = Compiler::new().run(&source);
    fs::write(opt.output, binary).expect("Invalid output file path.");
}
