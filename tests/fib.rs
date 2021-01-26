use eac_compiler::{self, Opt};
use std::fs;
use std::path::PathBuf;

#[test]
fn fib() {
    let opt = Opt {
        input: PathBuf::from("."),
        ast: false,
        ssa: false,
        cfg: false,
        vasm: false,
        asm: false,
    };
    let source = fs::read_to_string("tests/fib.c").unwrap();
    let asm = eac_compiler::compile(&source, opt).unwrap();
    let expected = fs::read_to_string("tests/fib.asm").unwrap();
    assert_eq!(asm, expected);
}
