#![feature(plugin)]
#![plugin(postgres_macros)]

fn main() {
    let s = sql!("SELECT foo FROM bar");
    assert_eq!("SELECT foo FROM bar", s);
}
