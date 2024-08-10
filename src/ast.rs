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
