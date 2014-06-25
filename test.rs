#![feature(phase)]

#[phase(plugin)]
extern crate postgres_macros;

struct Foo;

impl Foo {
    fn execute(&self, _: &[int]) {}
}

fn main() {
    let foo = Foo;

    execute!(foo, "SELECT * FROM foo WHERE a = $1 AND b = $2", 1, 2);
}
