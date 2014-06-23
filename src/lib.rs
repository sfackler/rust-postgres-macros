#![crate_id="postgres_macros"]
#![crate_type="dylib"]
#![feature(plugin_registrar)]

extern crate libc;
extern crate rustc;
extern crate syntax;

use rustc::plugin::Registry;
use std::mem;
use syntax::ast::{TokenTree, ExprLit, LitStr};
use syntax::codemap::Span;
use syntax::ext::base::{ExtCtxt, MacResult, MacExpr, DummyResult};
use syntax::parse;

mod ffi {
    use libc::{c_char, c_int};

    pub struct ParseResult {
        pub success: c_int,
    }

    #[link(name="parser", kind="static")]
    extern {
        pub fn init_parser();
        pub fn parse_query(query: *c_char, result: *mut ParseResult);
    }
}

#[plugin_registrar]
#[doc(hidden)]
pub fn registrar(reg: &mut Registry) {
    reg.register_macro("sql", expand_sql);
    unsafe { ffi::init_parser() }
}

fn expand_sql(cx: &mut ExtCtxt, sp: Span, tts: &[TokenTree])
              -> Box<MacResult> {
    let mut parser = parse::new_parser_from_tts(cx.parse_sess(), cx.cfg(),
                                                Vec::from_slice(tts));

    let e = parser.parse_expr();

    let query = match e.node {
        ExprLit(lit) => {
            match lit.node {
                LitStr(ref s, _) => s.clone(),
                _ => {
                    cx.span_err(e.span, "expected string literal");
                    return DummyResult::expr(sp);
                }
            }
        }
        _ => {
            cx.span_err(e.span, "expected string literal");
            return DummyResult::expr(sp);
        }
    };

    if !parse(query.get()) {
        cx.span_err(e.span, "Invalid SQL");
    }

    MacExpr::new(e)
}

pub fn parse(query: &str) -> bool {
    unsafe {
        ffi::init_parser();
        let mut result = mem::uninitialized();
        query.with_c_str(|query| {
            ffi::parse_query(query, &mut result);
        });
        result.success != 0
    }
}
