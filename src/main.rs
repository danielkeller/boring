#![allow(dead_code)]
mod ast;
mod ir;
mod lower;
mod pretty;

use lrlex::lrlex_mod;
use lrpar::lrpar_mod;

lrlex_mod!("boring.l");
lrpar_mod!("boring.y");

fn main() {
    let bump = &bumpalo::Bump::new();
    let lexerdef = boring_l::lexerdef();

    let input = std::fs::read_to_string("testdata/parse.bo").unwrap();
    let lexer = lexerdef.lexer(&input);
    let (res, errs) = boring_y::parse(&lexer, bump);
    for e in errs {
        println!("{}", e.pp(&lexer, &boring_y::token_epp));
    }
    let ast = res.unwrap();
    println!("{ast}");
    let ir = crate::lower::lower(ast, bump);
    println!("{ir}");
}
