#![feature(phase)]

#[phase(plugin)]
extern crate postgres_macros;

fn main() {
    let query = sql!("SELECT * FROM users WHERE name = $1");
    let bad_query = sql!("SELECT * FORM users WEHRE name = $1");
}
