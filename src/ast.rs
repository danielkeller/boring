use bumpalo::collections::Vec;

#[derive(Debug)]
pub struct Module<'a> {
    pub items: Vec<'a, Item<'a>>,
}

#[derive(Debug)]
pub struct Item<'a> {
    pub name: &'a str,
    pub ty_params: Vec<'a, TyParam<'a>>,
    pub params: Vec<'a, Param<'a>>,
    pub ret_ty: Type<'a>,
    pub body: Expr<'a>,
}

#[derive(Debug)]
pub struct Param<'a> {
    pub name: &'a str,
    pub ty: Type<'a>,
}

#[derive(Debug)]
pub enum TyParam<'a> {
    Lifetime(&'a str),
    Type(&'a str),
}

#[derive(Debug)]
pub enum Type<'a> {
    Ref { lifetime: &'a str, ty: &'a Type<'a> },
    Named(&'a str),
}

#[derive(Debug)]
pub enum Expr<'a> {
    Unit,
    Block(Vec<'a, Expr<'a>>),
    Let { name: &'a str, init: &'a Expr<'a> },
    Var(&'a str),
    App { func: &'a Expr<'a>, args: Vec<'a, Expr<'a>> },
}
