#![allow(unstable)]

use std::io::Command;
use std::io::process::StdioContainer::InheritFd;
use std::os;

fn main() {
    Command::new("make")
        .stdin(InheritFd(0))
        .stdout(InheritFd(1))
        .stderr(InheritFd(2))
        .status()
        .unwrap();
    let out_dir = os::getenv("OUT_DIR").unwrap();
    println!("cargo:rustc-flags=-L {} -l parser:static", out_dir);
}
