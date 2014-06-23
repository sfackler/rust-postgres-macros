#![feature(phase)]

#[phase(plugin)]
extern crate postgres_macros;

fn main() {
    let bad = sql!("SELECT * FROMBSDGF bar");
}
