%start module
%expect-unused Unmatched "UNMATCHED"
%parse-param bump: &'input bumpalo::Bump
%%

module -> Module<'input>
    : { Module { items: Vec::new_in(bump) } }
    | module fn { $1.items.push($2); $1 }
    ;

fn -> Item<'input>
    : "fn" ident fn_type_params "(" params ")" return_type block {
        Item { name: $2, ty_params: $3, params: $5, ret_ty: $7, body: $8 }
    }
    ;
fn_type_params -> Vec<'input, TyParam<'input>>
    : "[" type_params "]" { $2 }
    | { Vec::new_in(bump) } ;
type_params -> Vec<'input, TyParam<'input>>
    : type_param { vec![in bump; $1] }
    | type_params "," type_param { $1.push($3); $1 }
    | type_params "," { $1 }
    ;
type_param -> TyParam<'input>
    : lifetime { TyParam::Lifetime($1) }
    | ident { TyParam::Type($1) }
    ;
params -> Vec<'input, Param<'input>>
    : param { vec![in bump; $1] }
    | params "," param { $1.push($3); $1 }
    | params "," { $1 }
    ;
param -> Param<'input>
    : ident ":" type {
        Param { name: $1, ty: $3 }
    }
    ;
return_type -> Type<'input>
    : "->" type { $2 } 
    | { Type::Named("()") }
    ;

block -> Expr<'input>
    : "{" stmts "}" { Expr::Block($2) };
stmts -> Vec<'input, Expr<'input>>
    : stmt { vec![in bump; $1] }
    | stmts ";" stmt { $1.push($3); $1 }
    | stmts ";" { $1.push(Expr::Unit); $1 };

stmt -> Expr<'input>
    : let { $1 }
    | expr { $1 }
    ;

let -> Expr<'input>
    : "let" ident "=" expr { Expr::Let { name: $2, init: bump.alloc($4) } };

expr -> Expr<'input>: call_expr { $1 };
call_expr -> Expr<'input>
    : prim_expr { $1 }
    | prim_expr "(" args ")" { Expr::App { func: bump.alloc($1), args: $3 } }
    ;
args -> Vec<'input, Expr<'input>>
    : expr { vec![in bump; $1] }
    | args "," expr { $1.push($3); $1 }
    | args "," { $1 }
    ;
prim_expr -> Expr<'input>: ident { Expr::Var($1) };

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
