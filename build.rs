use std::process::{Command, Stdio};
use std::env;

fn main() {
    Command::new("make")
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .unwrap();
    let out_dir = env::var("OUT_DIR").unwrap();
    println!("cargo:rustc-flags=-L {} -l parser:static", out_dir);
}
