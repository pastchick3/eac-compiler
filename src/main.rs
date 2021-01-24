use eac_compiler::{compile, Opt};
use std::fs;
use std::process::{Command, Stdio};
use structopt::StructOpt;

fn main() {
    let opt = Opt::from_args();
    let source = fs::read_to_string(&opt.input).expect("Invalid input file path.");
    if let Some(asm) = compile(&source, opt) {
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
    };
}
