#![crate_name="postgres_macros"]
#![crate_type="dylib"]
#![feature(plugin_registrar)]

#![allow(unstable)]

extern crate libc;
extern crate rustc;
extern crate syntax;

use rustc::plugin::Registry;
use std::ffi::CString;
use std::mem;
use std::str;
use syntax::ast::{TokenTree, ExprLit, LitStr, Expr, Ident};
use syntax::codemap::Span;
use syntax::ext::base::{ExtCtxt, MacResult, MacExpr, DummyResult};
use syntax::ext::build::AstBuilder;
use syntax::fold::Folder;
use syntax::parse::token;
use syntax::parse::token::{InternedString, Comma, Eof};
use syntax::parse;
use syntax::parse::parser::Parser;
use syntax::ptr::P;

mod ffi {
    use libc::{c_char, c_int};

    #[repr(C)]
    pub struct ParseResult {
        pub success: c_int,
        pub error_message: *const c_char,
        pub index: c_int,
        pub num_params: c_int,
    }

    extern {
        pub fn init_parser();
        pub fn parse_query(query: *const c_char, result: *mut ParseResult);
    }
}

struct ParseInfo {
    num_params: usize,
}

struct ParseError {
    message: String,
    index: usize,
}

#[plugin_registrar]
#[doc(hidden)]
pub fn registrar(reg: &mut Registry) {
    reg.register_macro("sql", expand_sql);
    reg.register_macro("execute", expand_execute);
    unsafe { ffi::init_parser() }
}

fn expand_sql(cx: &mut ExtCtxt, sp: Span, tts: &[TokenTree])
              -> Box<MacResult+'static> {
    let mut parser = parse::new_parser_from_tts(cx.parse_sess(), cx.cfg(),
                                                tts.to_vec());

    let query_expr = cx.expander().fold_expr(parser.parse_expr());
    let query = match parse_str_lit(cx, &*query_expr) {
        Some(query) => query,
        None => return DummyResult::expr(sp)
    };

    match parse(query.get()) {
        Ok(_) => {}
        Err(err) => parse_error(cx, query_expr.span, err),
    }

    MacExpr::new(query_expr)
}

fn expand_execute(cx: &mut ExtCtxt, sp: Span, tts: &[TokenTree])
                  -> Box<MacResult+'static> {
    let mut parser = parse::new_parser_from_tts(cx.parse_sess(), cx.cfg(),
                                                tts.to_vec());

    let conn = parser.parse_expr();

    if !parser.eat(&Comma) {
        cx.span_err(parser.span, "expected `,`");
        return DummyResult::expr(sp);
    }

    let query_expr = cx.expander().fold_expr(parser.parse_expr());
    let query = match parse_str_lit(cx, &*query_expr) {
        Some(query) => query,
        None => return DummyResult::expr(sp),
    };

    if !parser.eat(&Comma) {
        cx.span_err(parser.span, "expected `,`");
        return DummyResult::expr(sp);
    }

    let args = match parse_args(cx, &mut parser) {
        Some(args) => args,
        None => return DummyResult::expr(sp),
    };

    match parse(query.get()) {
        Ok(ref info) if info.num_params != args.len() =>
            cx.span_err(sp, format!("Expected {} query parameters but got {}",
                                    info.num_params, args.len()).as_slice()),
        Ok(_) => {}
        Err(err) => parse_error(cx, query_expr.span, err),
    }

    let ident = Ident::new(token::intern("execute"));
    let args = cx.expr_vec(sp, args);
    MacExpr::new(cx.expr_method_call(sp, conn, ident, vec![query_expr, args]))
}

fn parse_error(cx: &mut ExtCtxt, sp: Span, err: ParseError) {
    cx.span_err(sp, format!("Invalid syntax at position {}: {}",
                            err.index,
                            err.message).as_slice());
}

fn parse_str_lit(cx: &mut ExtCtxt, e: &Expr) -> Option<InternedString> {
    match e.node {
        ExprLit(ref lit) => {
            match lit.node {
                LitStr(ref s, _) => Some(s.clone()),
                _ => {
                    cx.span_err(e.span, "expected string literal");
                    None
                }
            }
        }
        _ => {
            cx.span_err(e.span, "expected string literal");
            None
        }
    }
}

fn parse_args(cx: &mut ExtCtxt, parser: &mut Parser) -> Option<Vec<P<Expr>>> {
    let mut args = Vec::new();

    while parser.token != Eof {
        args.push(parser.parse_expr());

        if !parser.eat(&Comma) && parser.token != Eof {
            cx.span_err(parser.span, "expected `,`");
            return None;
        }
    }

    Some(args)
}

fn parse(query: &str) -> Result<ParseInfo, ParseError> {
    unsafe {
        let mut result = mem::uninitialized();
        let query = CString::from_slice(query.as_bytes());
        ffi::parse_query(query.as_ptr(), &mut result);
        if result.success != 0 {
            Ok(ParseInfo {
                num_params: result.num_params as usize,
            })
        } else {
            let bytes = std::ffi::c_str_to_bytes(&result.error_message);
            Err(ParseError {
                message: str::from_utf8(bytes).unwrap().to_string(),
                index: result.index as usize,
            })
        }
    }
}
