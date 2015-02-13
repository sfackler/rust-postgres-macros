rust-postgres-macros
====================

A set of support macros for Rust-Postgres.

[![Build Status](https://travis-ci.org/sfackler/rust-postgres-macros.svg?branch=master)](https://travis-ci.org/sfackler/rust-postgres-macros)

sql!
====

The `sql!` macro will validate that its string literal argument parses as a
valid Postgres query.

```rust
#![feature(plugin)]
#![plugin(postgres_macros)]

fn main() {
    let query = sql!("SELECT * FROM users WHERE name = $1");
    let bad_query = sql!("SELECT * FORM users WHERE name = $1");
}
```

```
test.rs:8:26: 8:63 error: Invalid syntax at position 10: syntax error at or near "FORM"
test.rs:8     let bad_query = sql!("SELECT * FORM users WEHRE name = $1");
                                   ^~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
error: aborting due to previous error
```

Credits
=======

Major thanks to [pganalyze](http://pganalyze.com) for their
[writeup](https://pganalyze.com/blog/parse-postgresql-queries-in-ruby.html) on
how to link to the Postgres query parser directly!
