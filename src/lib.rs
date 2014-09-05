#![crate_name="postgres_macros"]
#![crate_type="dylib"]
#![feature(plugin_registrar)]

extern crate libc;
extern crate rustc;
extern crate syntax;

use rustc::plugin::Registry;
use std::c_str::CString;
use std::mem;
use std::gc::Gc;
use syntax::ast::{TokenTree, ExprLit, LitStr, Expr, Ident};
use syntax::codemap::Span;
use syntax::ext::base::{ExtCtxt, MacResult, MacExpr, DummyResult};
use syntax::ext::build::AstBuilder;
use syntax::fold::Folder;
use syntax::parse::token;
use syntax::parse::token::{InternedString, COMMA, EOF};
use syntax::parse;
use syntax::parse::parser::Parser;

mod ffi {
    use libc::{c_char, c_int};

    #[repr(C)]
    pub struct ParseResult {
        pub success: c_int,
        pub error_message: *const c_char,
        pub index: c_int,
        pub num_params: c_int,
    }

    #[link(name="parser", kind="static")]
    extern {
        pub fn init_parser();
        pub fn parse_query(query: *const c_char, result: *mut ParseResult);
    }
}

struct ParseInfo {
    num_params: uint,
}

struct ParseError {
    message: CString,
    index: uint,
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
                                                Vec::from_slice(tts));

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
                                                Vec::from_slice(tts));

    let conn = parser.parse_expr();

    if !parser.eat(&COMMA) {
        cx.span_err(parser.span, "expected `,`");
        return DummyResult::expr(sp);
    }

    let query_expr = cx.expander().fold_expr(parser.parse_expr());
    let query = match parse_str_lit(cx, &*query_expr) {
        Some(query) => query,
        None => return DummyResult::expr(sp),
    };

    if !parser.eat(&COMMA) {
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
                            err.message.as_str().unwrap()).as_slice());
}

fn parse_str_lit(cx: &mut ExtCtxt, e: &Expr) -> Option<InternedString> {
    match e.node {
        ExprLit(lit) => {
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

fn parse_args(cx: &mut ExtCtxt, parser: &mut Parser) -> Option<Vec<Gc<Expr>>> {
    let mut args = Vec::new();

    while parser.token != EOF {
        args.push(parser.parse_expr());

        if !parser.eat(&COMMA) && parser.token != EOF {
            cx.span_err(parser.span, "expected `,`");
            return None;
        }
    }

    Some(args)
}

fn parse(query: &str) -> Result<ParseInfo, ParseError> {
    unsafe {
        ffi::init_parser();
        let mut result = mem::uninitialized();
        query.with_c_str(|query| {
            ffi::parse_query(query, &mut result);
        });
        match result.success != 0 {
            true => Ok(ParseInfo {
                num_params: result.num_params as uint,
            }),
            false => Err(ParseError {
                message: CString::new(result.error_message, true),
                index: result.index as uint,
            }),
        }
    }
}
