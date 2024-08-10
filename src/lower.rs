use bumpalo::collections::Vec;
use std::collections::HashMap;

use crate::{ast, ir};

pub fn lower<'a>(
    ast: ast::Module<'a>, bump: &'a bumpalo::Bump,
) -> ir::Module<'a> {
    let iter = ast.items.iter().map(|i| lower_item(*i, bump));
    ir::Module { items: bump.alloc_slice_fill_iter(iter) }
}

#[derive(Debug, Default)]
struct Env<'e, 'a> {
    items: HashMap<&'a str, ir::Value<'a>>,
    parent: Option<&'e Env<'e, 'a>>,
}

impl<'e, 'a> Env<'e, 'a> {
    fn insert(&mut self, name: &'a str, value: ir::Value<'a>) {
        self.items.insert(name, value);
    }
    fn get(&self, name: &'a str) -> Option<ir::Value<'a>> {
        match (self.items.get(name), self.parent) {
            (Some(value), _) => Some(*value),
            (_, Some(parent)) => parent.get(name),
            _ => None,
        }
    }
    fn push(&self) -> Env<'_, 'a> {
        Env { items: HashMap::new(), parent: Some(self) }
    }
}

fn lower_item<'a>(ast: ast::Item<'a>, bump: &'a bumpalo::Bump) -> ir::Item<'a> {
    let mut env: Env = Default::default();

    let mut body = Vec::new_in(bump);
    body.push(ir::BB {
        body: Vec::new_in(bump),
        term: ir::Terminator::Return(ir::Value::Arg(0)),
    });
    let mut to = ir::Item { name: &ast.name, body };
    for (i, param) in ast.params.iter().enumerate() {
        env.insert(param.name, ir::Value::Arg(i));
    }

    let ret = lower_expr(&mut to, &mut env, ast.body, bump);
    to.body.last_mut().unwrap().term = ir::Terminator::Return(ret);
    to
}

fn lower_expr<'a>(
    to: &mut ir::Item<'a>, env: &mut Env<'_, 'a>, ast: ast::Expr<'a>,
    bump: &'a bumpalo::Bump,
) -> ir::Value<'a> {
    match ast {
        ast::Expr::Unit => lower_unit(to, bump),
        ast::Expr::Block { body, result } => {
            lower_block(to, env, body, result, bump)
        }
        ast::Expr::Let { name, init } => lower_let(to, env, name, *init, bump),
        ast::Expr::Var(name) => env.get(name).expect("undeclared variable"),
        ast::Expr::App { func, args } => lower_app(to, env, *func, args, bump),
    }
}

fn lower_unit<'a>(
    to: &'_ mut ir::Item<'a>, bump: &'a bumpalo::Bump,
) -> ir::Value<'a> {
    let instr = &*bump.alloc(ir::Instr::Lit);
    to.body.last_mut().unwrap().body.push(instr);
    ir::Value::Instr(instr)
}

fn lower_let<'a>(
    to: &'_ mut ir::Item<'a>, env: &mut Env<'_, 'a>, name: &'a str,
    init: ast::Expr<'a>, bump: &'a bumpalo::Bump,
) -> ir::Value<'a> {
    let value = lower_expr(to, env, init, bump);
    env.insert(name, value);
    value
}

fn lower_app<'a>(
    to: &'_ mut ir::Item<'a>, env: &mut Env<'_, 'a>, func: ast::Expr<'a>,
    args: &'a [ast::Expr<'a>], bump: &'a bumpalo::Bump,
) -> ir::Value<'a> {
    let func = lower_expr(to, env, func, bump);
    let args = &*bump.alloc_slice_fill_iter(
        args.iter().map(|&arg| lower_expr(to, env, arg, bump)),
    );
    let instr = &*bump.alloc(ir::Instr::App { func, args });
    to.body.last_mut().unwrap().body.push(instr);
    ir::Value::Instr(instr)
}

fn lower_block<'a>(
    to: &'_ mut ir::Item<'a>, env: &mut Env<'_, 'a>, body: &'a [ast::Expr<'a>],
    result: Option<&'a ast::Expr<'a>>, bump: &'a bumpalo::Bump,
) -> ir::Value<'a> {
    let mut env = env.push();
    for &expr in body {
        lower_expr(to, &mut env, expr, bump);
    }

    if let Some(&result) = result {
        lower_expr(to, &mut env, result, bump)
    } else {
        lower_unit(to, bump)
    }
}
