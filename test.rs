#![feature(phase)]

#[phase(plugin)]
extern crate postgres_macros;

struct Foo;

impl Foo {
    fn execute(&self, query: &str, args: &[int]) {
        println!("query {} args {}", query, args);
    }
}

fn main() {
    let foo = Foo;

    execute!(foo, "SELECT * FROM foo WHERE a = $1 AND b = $2", 1, 2);
}
