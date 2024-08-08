use cfgrammar::yacc::YaccKind;
use lrlex::CTLexerBuilder;

fn main() {
    CTLexerBuilder::new()
        .lrpar_config(|ctp| {
            ctp.yacckind(YaccKind::Grmtools)
                .grammar_in_src_dir("boring.y")
                .unwrap()
        })
        .lexer_in_src_dir("boring.l")
        .unwrap()
        .build()
        .unwrap();
    println!("cargo::rerun-if-changed=src/boring.l");
    println!("cargo::rerun-if-changed=src/boring.y");
}
