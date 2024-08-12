%start module
%expect-unused Unmatched "UNMATCHED"
%parse-param bump: &'input bumpalo::Bump
%%

module -> Module<'input>
    : fns { Module { items: slice($1) } }
    ;

fns -> Vec<'input, Item<'input>>
    : { Vec::new_in(bump) }
    | fns fn { $1.push($2); $1 }
    ;

fn -> Item<'input>
    : "fn" ident fn_type_params fn_params return_type block {
        Item { name: $2, ty_params: $3, params: $4, ret_ty: $5, body: $6 }
    }
    ;
fn_type_params -> &'input [TyParam<'input>]
    : { &[] }
    | "[" type_params "]" { slice($2) }
    ;
type_params -> Vec<'input, TyParam<'input>>
    : type_param { vec![in bump; $1] }
    | type_params "," type_param { $1.push($3); $1 }
    ;
type_param -> TyParam<'input>
    : lifetime { TyParam::Lifetime($1) }
    | ident { TyParam::Type($1) }
    ;
fn_params -> &'input [Param<'input>]
    : "(" ")" { &[] }
    | "(" params ")" { slice($2) }
    ;
params -> Vec<'input, Param<'input>>
    : param { vec![in bump; $1] }
    | params "," param { $1.push($3); $1 }
    ;
param -> Param<'input>
    : ident ":" type { Param { name: $1, ty: $3 } }
    ;
return_type -> Type<'input>
    : "->" type { $2 } 
    | { Type::Named("()") }
    ;

block -> Expr<'input>
    : "{" stmts "}" { Expr::Block { body: slice($2), result: None } }
    | "{" stmts expr_without_block "}" {
        Expr::Block {
            body: slice($2),
            result: Some(bump.alloc($3)),
        } 
    }
    ;
stmts -> Vec<'input, Expr<'input>>
    : { vec![in bump] }
    | stmts stmt { $1.push($2); $1 }
    ;

stmt -> Expr<'input>
    : let ";" { $1 }
    | expr_with_block { $1 }
    | expr_without_block ";" { $1 }
    ;

let -> Expr<'input>
    : "let" ident "=" expr { Expr::Let { name: $2, init: bump.alloc($4) } };

expr -> Expr<'input>
    : expr_with_block { $1 }
    | expr_without_block { $1 }
    ;

expr_with_block -> Expr<'input>
    : if_expr { $1 }
    ;

expr_without_block -> Expr<'input>
    : call_expr { $1 }
    ;

if_expr -> Expr<'input>
    : "if" expr block { 
        Expr::If { cond: bump.alloc($2), yes: bump.alloc($3), no: None }
    }
    | "if" expr block "else" else_branch {
        Expr::If { 
            cond: bump.alloc($2), 
            yes: bump.alloc($3), 
            no: Some(bump.alloc($5)) 
        }
    }
    ;

else_branch -> Expr<'input>
    : if_expr { $1 }
    | block { $1 }
    ;

call_expr -> Expr<'input>
    : prim_expr { $1 }
    | prim_expr call_args { Expr::App { func: bump.alloc($1), args: $2 } }
    ;
call_args -> &'input [Expr<'input>]
    : "(" ")" { &[] }
    | "(" args ")" { slice($2) }
    ;
args -> Vec<'input, Expr<'input>>
    : expr { vec![in bump; $1] }
    | args "," expr { $1.push($3); $1 }
    ;
prim_expr -> Expr<'input>
    : ident { Expr::Var($1) }
    | "(" expr ")" { $2 }
    | "(" ")" { Expr::Unit }
    ;

lifetime -> &'input str: "'" ident { $2 };

type -> Type<'input>
    : ident { Type::Named($1) }
    | "&" ref_props type { Type::Ref { lifetime: $2, ty: bump.alloc($3) } };

ref_props -> &'input str
    : lifetime { $1 }
    | { "_" }
    ;

ident -> &'input str
    : "identifier" { $lexer.span_str($span) }
    ;

Unmatched -> ():
    "UNMATCHED" { } ;

%%

use crate::ast::*;
use bumpalo::collections::Vec;
use bumpalo::vec;

fn slice<'bump, T>(vec: Vec<'bump, T>) -> &'bump [T] {
    vec.into_bump_slice()
}
