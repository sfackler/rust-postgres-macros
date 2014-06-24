#![feature(phase)]

#[phase(plugin)]
extern crate postgres_macros;

fn main() {
    let bad = sql!("SELECT * FROMBSDGF bar");
    let good = sql!("SELECT * FROM foo WHERE bar = $1 AND baz = $3");
}
