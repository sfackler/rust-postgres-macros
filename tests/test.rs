extern crate compiletest_rs as compiletest;

use std::path::PathBuf;

fn run_mode(mode: &'static str) {
    let mut config = compiletest::common::Config::default();
    let cfg_mode = mode.parse().ok().expect("Invalid mode");

    config.mode = cfg_mode;
    config.target_rustcflags = Some("-L target/debug/".to_owned());
    config.src_base = PathBuf::from(format!("tests/{}", mode));

    compiletest::run_tests(&config);
}

#[test]
fn compile_fail() {
    run_mode("compile-fail");
    run_mode("run-pass");
}
