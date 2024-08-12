use std::fmt::Display;

use crate::pretty::Commas;

#[derive(Debug, Clone, Copy)]
pub struct Module<'a> {
    pub items: &'a [Item<'a>],
}

#[derive(Debug, Clone, Copy)]
pub struct Item<'a> {
    pub name: &'a str,
    pub ty_params: &'a [TyParam<'a>],
    pub params: &'a [Param<'a>],
    pub ret_ty: Type<'a>,
    pub body: Expr<'a>,
}

#[derive(Debug, Clone, Copy)]
pub struct Param<'a> {
    pub name: &'a str,
    pub ty: Type<'a>,
}

#[derive(Debug, Clone, Copy)]
pub enum TyParam<'a> {
    Lifetime(&'a str),
    Type(&'a str),
}

#[derive(Debug, Clone, Copy)]
pub enum Type<'a> {
    Ref { lifetime: &'a str, ty: &'a Type<'a> },
    Named(&'a str),
}

#[derive(Debug, Clone, Copy)]
pub enum Expr<'a> {
    Unit,
    Block { body: &'a [Expr<'a>], result: Option<&'a Expr<'a>> },
    Let { name: &'a str, init: &'a Expr<'a> },
    Var(&'a str),
    App { func: &'a Expr<'a>, args: &'a [Expr<'a>] },
}

impl Display for Module<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for item in self.items {
            write!(f, "{item}\n\n")?;
        }
        Ok(())
    }
}

impl Display for Item<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let Item { name, ty_params, params, ret_ty, body } = self;
        let ty_params = Commas(*ty_params);
        let params = Commas(*params);
        write!(f, "fn {name}[{ty_params}]({params}) -> {ret_ty} {body}")
    }
}

impl Display for Param<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let Param { name, ty } = self;
        write!(f, "{name}: {ty}")
    }
}

impl Display for TyParam<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TyParam::Lifetime(name) => write!(f, "'{name}"),
            TyParam::Type(name) => write!(f, "{name}"),
        }
    }
}

impl Display for Type<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Type::Ref { lifetime, ty } => write!(f, "&'{lifetime} {ty}"),
            Type::Named(name) => f.write_str(name),
        }
    }
}

impl Display for Expr<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Expr::Unit => f.write_str("()"),
            Expr::Block { body, result: None } => {
                f.write_str("{\n")?;
                write_block_body(body, f)?;
                f.write_str("}")
            }
            Expr::Block { body, result: Some(result) } => {
                f.write_str("{\n")?;
                write_block_body(body, f)?;
                write!(f, "    {result}\n}}")
            }
            Expr::Let { name, init } => write!(f, "let {name} = {init}"),
            Expr::Var(name) => f.write_str(name),
            Expr::App { func, args } => write!(f, "{func}({})", Commas(*args)),
        }
    }
}

fn write_block_body(
    body: &[Expr], f: &mut std::fmt::Formatter,
) -> std::fmt::Result {
    for stmt in body {
        write!(f, "    {stmt};\n")?;
    }
    Ok(())
}
