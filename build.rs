use std::io::Command;
use std::os;

fn main() {
    Command::new("make").status().unwrap();
    let out_dir = os::getenv("OUT_DIR").unwrap();
    println!("cargo:rustc-flags=-L {} -l parser:static", out_dir);
}
