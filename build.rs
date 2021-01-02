use cmake::Config;

fn main() {
    println!("cargo:rerun-if-changed=parser/parser.cpp");
    let dst = Config::new("parser").profile("Release").build();
    println!("cargo:rustc-link-search=native={}", dst.display());
    println!("cargo:rustc-link-search=native=./parser", );
    println!("cargo:rustc-link-lib=static=CBaseListener");
    println!("cargo:rustc-link-lib=static=CLexer");
    println!("cargo:rustc-link-lib=static=CListener");
    println!("cargo:rustc-link-lib=static=CParser");
    println!("cargo:rustc-link-lib=static=parser");
    println!("cargo:rustc-link-lib=dylib=antlr4-runtime");
}
