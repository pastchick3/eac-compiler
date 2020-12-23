use libc::c_int;

#[link(name = "parser")]
extern "C" {
    fn parse() -> c_int;
}

fn main() {
    println!("{}", unsafe { parse() });
}
