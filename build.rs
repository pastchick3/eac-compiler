use cmake::Config;

fn main() {
    println!("cargo:rerun-if-changed=parser/parser.cpp");
    let dst = Config::new("parser").profile("Release").build();
    println!("cargo:rustc-link-search=native={}", dst.display());
    println!("cargo:rustc-link-lib=static=parser");
    println!("cargo:rustc-link-search=native=.", );
    println!("cargo:rustc-link-lib=static=antlr4-runtime");
}
