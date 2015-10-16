#![feature(plugin)]
#![plugin(postgres_macros)]

fn main() {
    let _ = sql!("SELECT foo FORM bar"); //~ ERROR syntax error at or near "bar"
}
