use bumpalo::collections::Vec;
use std::collections::HashMap;

use crate::{ast, ir};

pub fn lower<'a>(
    ast: ast::Module<'a>, bump: &'a bumpalo::Bump,
) -> ir::Module<'a> {
    let iter = ast.items.iter().map(|i| lower_item(*i, bump));
    ir::Module { items: bump.alloc_slice_fill_iter(iter) }
}

// Is it possible that definite initialization analysis is the same as the
// dominance frontier algorithm?

#[derive(Debug, Default)]
struct Env<'e, 'a> {
    items: HashMap<&'a str, ir::Value>,
    parent: Option<&'e Env<'e, 'a>>,
}

impl<'e, 'a> Env<'e, 'a> {
    fn insert(&mut self, name: &'a str, value: ir::Value) {
        self.items.insert(name, value);
    }
    fn get(&self, name: &'a str) -> Option<ir::Value> {
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
        n_params: ast.params.len(),
        body: Vec::new_in(bump),
        term: ir::Terminator::Halt,
        idom: 0,
    });
    let mut to = ir::Item { name: &ast.name, body };
    for (i, param) in ast.params.iter().enumerate() {
        env.insert(param.name, ir::Value(-((i + 1) as i32)));
    }

    let ret = lower_expr(&mut to, &mut env, ast.body, bump);
    to.body.last_mut().unwrap().term = ir::Terminator::Return(ret);
    to
}

fn push<'a>(to: &'_ mut ir::Item<'a>, instr: ir::Instr<'a>) -> ir::Value {
    let bb = label(to);
    to.body[bb].body.push(instr);
    to.instr(bb, to.body[bb].body.len() - 1)
}

fn label<'a>(to: &'_ ir::Item<'a>) -> usize {
    to.body.len() - 1
}

fn new_bb<'a>(
    to: &'_ mut ir::Item<'a>, n_params: usize, idom: usize,
    bump: &'a bumpalo::Bump,
) {
    to.body.push(ir::BB {
        n_params,
        body: Vec::new_in(bump),
        term: ir::Terminator::Halt,
        idom,
    });
}

fn lower_expr<'a>(
    to: &mut ir::Item<'a>, env: &mut Env<'_, 'a>, ast: ast::Expr<'a>,
    bump: &'a bumpalo::Bump,
) -> ir::Value {
    match ast {
        ast::Expr::Unit => lower_unit(to),
        ast::Expr::Stmt(expr) => {
            lower_expr(to, env, *expr, bump);
            lower_unit(to)
        }
        ast::Expr::Block(body) => lower_block(to, env, body, bump),
        ast::Expr::Let { name, init } => lower_let(to, env, name, *init, bump),
        ast::Expr::Var(name) => env.get(name).expect("undeclared variable"),
        ast::Expr::App { func, args } => lower_app(to, env, *func, args, bump),
        ast::Expr::If { cond, yes, no } => {
            lower_if(to, env, cond, yes, no, bump)
        }
    }
}

fn lower_unit<'a>(to: &'_ mut ir::Item<'a>) -> ir::Value {
    push(to, ir::Instr::Lit)
}

fn lower_let<'a>(
    to: &'_ mut ir::Item<'a>, env: &mut Env<'_, 'a>, name: &'a str,
    init: ast::Expr<'a>, bump: &'a bumpalo::Bump,
) -> ir::Value {
    let value = lower_expr(to, env, init, bump);
    env.insert(name, value);
    value
}

fn lower_app<'a>(
    to: &'_ mut ir::Item<'a>, env: &mut Env<'_, 'a>, func: ast::Expr<'a>,
    args: &'a [ast::Expr<'a>], bump: &'a bumpalo::Bump,
) -> ir::Value {
    let func = lower_expr(to, env, func, bump);
    let args = &*bump.alloc_slice_fill_iter(
        args.iter().map(|&arg| lower_expr(to, env, arg, bump)),
    );
    push(to, ir::Instr::App { func, args })
}

fn lower_if<'a>(
    to: &'_ mut ir::Item<'a>, env: &mut Env<'_, 'a>, cond: &'a ast::Expr,
    yes: &'a ast::Expr, no: Option<&'a ast::Expr>, bump: &'a bumpalo::Bump,
) -> ir::Value {
    let cond = lower_expr(to, env, *cond, bump);
    let cond_exit = label(to);
    if let Some(no) = no {
        new_bb(to, 0, cond_exit, bump);
        let yes_enter = label(to);
        let yes = lower_expr(to, env, *yes, bump);
        let yes_exit = label(to);
        new_bb(to, 0, cond_exit, bump);
        let no_enter = label(to);
        let no = lower_expr(to, env, *no, bump);
        let no_exit = label(to);
        new_bb(to, 1, cond_exit, bump);
        let exit = label(to);

        let yes_jmp = ir::Jmp { to: yes_enter, args: &[] };
        let no_jmp = ir::Jmp { to: no_enter, args: &[] };
        to.body[cond_exit].term = ir::Terminator::Switch(
            cond,
            bump.alloc_slice_copy(&[no_jmp, yes_jmp]),
        );
        to.body[yes_exit].term = ir::Terminator::Jmp(ir::Jmp {
            to: exit,
            args: bump.alloc_slice_copy(&[yes]),
        });
        to.body[no_exit].term = ir::Terminator::Jmp(ir::Jmp {
            to: exit,
            args: bump.alloc_slice_copy(&[no]),
        });
        to.param(exit, 0)
    } else {
        new_bb(to, 0, cond_exit, bump);
        let yes_enter = label(to);
        lower_expr(to, env, *yes, bump);
        let yes_exit = label(to);
        new_bb(to, 1, cond_exit, bump);
        let exit = label(to);

        let yes_jmp = ir::Jmp { to: yes_enter, args: &[] };
        let no_jmp = ir::Jmp { to: exit, args: &[] };
        to.body[cond_exit].term = ir::Terminator::Switch(
            cond,
            bump.alloc_slice_copy(&[no_jmp, yes_jmp]),
        );
        to.body[yes_exit].term =
            ir::Terminator::Jmp(ir::Jmp { to: exit, args: &[] });

        lower_unit(to)
    }
}

fn lower_block<'a>(
    to: &'_ mut ir::Item<'a>, env: &mut Env<'_, 'a>, body: &'a [ast::Expr<'a>],
    bump: &'a bumpalo::Bump,
) -> ir::Value {
    let mut env = env.push();
    if let [body @ .., result] = body {
        for &expr in body {
            lower_expr(to, &mut env, expr, bump);
        }
        lower_expr(to, &mut env, *result, bump)
    } else {
        lower_unit(to)
    }
}
