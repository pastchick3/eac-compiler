use eac_compiler::compile;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "parser")]
struct Opt {
    #[structopt(parse(from_os_str))]
    input: PathBuf,
}

fn main() {
    let opt = Opt::from_args();
    let source = fs::read_to_string(opt.input).expect("Invalid input file path.");
    let asm = compile(&source);
    fs::write("main.asm", asm).expect("Fail to write the output assembly file.");
    Command::new("ml64")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .args(&[
            "driver.asm",
            "main.asm",
            "/Fe",
            "main.exe",
            "/link",
            "/subsystem:console",
            "/defaultlib:kernel32.lib",
            "/entry:drive",
        ])
        .output()
        .expect("Fail to call the assembler.");
}
