#![feature(plugin_registrar, rustc_private, libc)]

extern crate libc;
extern crate rustc;
extern crate syntax;

use rustc::plugin::Registry;
use std::ffi::{CStr, CString};
use std::mem;
use std::str;
use syntax::ast::{TokenTree, ExprLit, LitStr, Expr, Ident};
use syntax::codemap::Span;
use syntax::ext::base::{ExtCtxt, MacResult, MacEager, DummyResult};
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
    num_params: Option<usize>,
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

    let query_expr = cx.expander().fold_expr(parser.parse_expr().unwrap());
    let query = match parse_str_lit(cx, &*query_expr) {
        Some(query) => query,
        None => return DummyResult::expr(sp)
    };

    match parse(&query) {
        Ok(_) => {}
        Err(err) => parse_error(cx, query_expr.span, err),
    }

    MacEager::expr(query_expr)
}

fn expand_execute(cx: &mut ExtCtxt, sp: Span, tts: &[TokenTree])
                  -> Box<MacResult+'static> {
    let mut parser = parse::new_parser_from_tts(cx.parse_sess(), cx.cfg(),
                                                tts.to_vec());

    let conn = parser.parse_expr().unwrap();

    if !parser.eat(&Comma).ok().unwrap() {
        cx.span_err(parser.span, "expected `,`");
        return DummyResult::expr(sp);
    }

    let query_expr = cx.expander().fold_expr(parser.parse_expr().unwrap());
    let query = match parse_str_lit(cx, &*query_expr) {
        Some(query) => query,
        None => return DummyResult::expr(sp),
    };

    if parser.token != Eof && !parser.eat(&Comma).ok().unwrap() {
        cx.span_err(parser.span, "expected `,`");
        return DummyResult::expr(sp);
    }

    let args = match parse_args(cx, &mut parser) {
        Some(args) => args,
        None => return DummyResult::expr(sp),
    };

    match parse(&query) {
        Ok(ParseInfo { num_params: None }) => {
            cx.span_warn(sp, "unable to verify the number of query parameters");
        }
        Ok(ParseInfo { num_params: Some(num_params) }) if num_params != args.len() => {
            cx.span_err(sp, &format!("Expected {} query parameters but got {}",
                                     num_params, args.len()));
        }
        Ok(_) => {}
        Err(err) => parse_error(cx, query_expr.span, err),
    }

    let ident = Ident::with_empty_ctxt(token::intern("execute"));
    let args = cx.expr_vec(sp, args);
    let args = cx.expr_addr_of(sp, args);
    MacEager::expr(cx.expr_method_call(sp, conn, ident, vec![query_expr, args]))
}

fn parse_error(cx: &mut ExtCtxt, sp: Span, err: ParseError) {
    cx.span_err(sp, &format!("Invalid syntax at position {}: {}", err.index, err.message));
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
        args.push(parser.parse_expr().unwrap());

        if !parser.eat(&Comma).ok().unwrap() && parser.token != Eof {
            cx.span_err(parser.span, "expected `,`");
            return None;
        }
    }

    Some(args)
}

fn parse(query: &str) -> Result<ParseInfo, ParseError> {
    unsafe {
        let mut result = mem::uninitialized();
        let query = CString::new(query.as_bytes()).unwrap();
        ffi::parse_query(query.as_ptr(), &mut result);
        if result.success != 0 {
            let num_params = if result.num_params < 0 {
                None
            } else {
                Some(result.num_params as usize)
            };
            Ok(ParseInfo {
                num_params: num_params,
            })
        } else {
            let bytes = CStr::from_ptr(result.error_message).to_bytes();
            Err(ParseError {
                message: str::from_utf8(bytes).unwrap().to_string(),
                index: result.index as usize,
            })
        }
    }
}
