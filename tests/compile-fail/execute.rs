#![feature(plugin)]
#![plugin(postgres_macros)]

struct Connection;

impl Connection {
    fn execute(&self, _: &str, _: &[&i32]) {}
}

fn main() {
    execute!(Connection, "SELECT foo FORM bar"); //~ ERROR syntax error at or near "bar"

    execute!(Connection, "SELECT foo FROM bar", &1); //~ ERROR Expected 0 query parameters but got 1

    execute!(Connection, "SELECT foo FROM bar WHERE baz = $1 AND buz = $2", &1); //~ ERROR Expected 2 query parameters but got 1

    execute!(Connection, "SELECT foo FROM bar WHERE baz = $1", &1, &2); //~ ERROR Expected 1 query parameters but got 2

    execute!(Connection, "CREATE TABLE foo ()", &1); //~ WARN unable to verify the number of query parameters
}
