#![feature(plugin)]
#![plugin(postgres_macros)]

struct Connection;

impl Connection {
    fn execute(&self, _: &str, _: &[&i32]) {}
}

fn main() {
    execute!(Connection, "SELECT foo FROM bar");

    execute!(Connection, "SELECT foo FROM bar WHERE baz = $1", &1);

    execute!(Connection, "SELECT foo FROM bar WHERE baz = $1 AND buz = $2", &1, &2);
}
