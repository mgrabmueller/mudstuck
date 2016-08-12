// Copyright 2016 Martin Grabmueller. See the LICENSE file at the
// top-level directory of this distribution for license information.

//! Templating language for Mudstuck.

use super::scanner;
use super::scanner::Scanner;

#[derive(Debug, Clone)]
pub enum Ast {
    Empty,
    Seq(Box<Ast>, Box<Ast>),
    Chr(char),
    Str(String),
    Id(String),
    Call(Box<Ast>, Vec<Ast>),
}

fn parse_ident(s: &mut Scanner) -> Result<Ast, String> {
    let mut ret = String::new();
    match s.current() {
        None =>
            Err("identifier expected".to_string()),
        Some(c) if (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '_' => {
            s.next();
            ret.push(c);
            loop {
                match s.current() {
                    None =>
                        return Ok(Ast::Id(ret)),
                    Some(c) if (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') ||
                                c == '_' || c == '.' || (c >= '0' && c <= '9') => {
                        s.next();
                        ret.push(c);
                    },
                    Some(_) =>
                        return Ok(Ast::Id(ret)),
                }
            }
        },
        Some(_) =>
            Err("identifier expected".to_string())
    }
}

fn parse_call(s: &mut Scanner) -> Result<Ast, String> {
    scanner::skip_ws(s);
    match parse_ident(s) {
        Err(e) => Err(e),
        Ok(id) => {
            let mut args = Vec::new();
            scanner::skip_ws(s);
            loop {
                match s.current() {
                    None => return Err("unexpected end of string in call expression".to_string()),
                    Some(c) if c == ')' => {
                        s.next();
                        return Ok(Ast::Call(Box::new(id), args))
                    },
                    Some(_) => {
                        let a = try!(parse_expr(s));
                        args.push(a);
                    }
                }
            }
        }
    }
}

fn parse_string(quote: char, s: &mut Scanner) -> Result<Ast, String> {
    let mut res = String::new();
    loop {
        match s.current() {
            None =>
                return Err("unexpected end of string in string literal".to_string()),
            Some(c) if c == quote => {
                s.next();
                return Ok(Ast::Str(res));
            },
            Some(c) => {
                s.next();
                res.push(c)
            },
        }
    }
}

fn parse_expr(s: &mut Scanner) -> Result<Ast, String> {
    scanner::skip_ws(s);
    match s.current() {
        None =>
            Err("unexpected end of string in expression".to_string()),
        Some(c) if c == '(' => {
            s.next();
            parse_call(s)
        },
        Some(c) if c == '\'' || c == '"' => {
            s.next();
            parse_string(c, s)
        },
        Some(c) if (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '_' => {
            parse_ident(s)
        }
        Some(c) =>
            Err(format!("unexpected character in expression: {}", c)),
    }
}

pub fn parse(txt: &str) -> Result<Ast, String> {
    let mut s = Scanner::new(txt);
    let mut ret = Ast::Empty;
    loop {
        match s.current() {
            None =>
                break,
            Some(c) if c == '#' => {
                s.next();
                let a = try!(parse_expr(&mut s));
                ret = Ast::Seq(Box::new(ret), Box::new(a))
            }
            Some(c) => {
                let mut acc = String::new();
                acc.push(c);
                s.next();
                loop {
                    match s.current() {
                        None => {
                            ret = Ast::Seq(Box::new(ret), Box::new(Ast::Str(acc)));
                            break
                        },
                        Some(c) if c == '#' => {
                            ret = Ast::Seq(Box::new(ret), Box::new(Ast::Str(acc)));
                            break
                        },
                        Some(c) => {
                            acc.push(c);
                            s.next();
                        }
                    }
                }
            }
        }
    }
    return Ok(ret);
}
