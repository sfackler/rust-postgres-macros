#![feature(io, env)]

use std::old_io::Command;
use std::old_io::process::StdioContainer::InheritFd;
use std::env;

fn main() {
    Command::new("make")
        .stdin(InheritFd(0))
        .stdout(InheritFd(1))
        .stderr(InheritFd(2))
        .status()
        .unwrap();
    let out_dir = env::var_string("OUT_DIR").unwrap();
    println!("cargo:rustc-flags=-L {} -l parser:static", out_dir);
}
