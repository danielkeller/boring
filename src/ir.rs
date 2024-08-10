use bumpalo::collections::Vec;

#[derive(Debug, Clone, Copy)]
pub struct Module<'a> {
    pub items: &'a [Item<'a>],
}
#[derive(Debug)]
pub struct Item<'a> {
    pub name: &'a str,
    pub body: Vec<'a, BB<'a>>,
}

#[derive(Debug)]
pub struct BB<'a> {
    pub body: Vec<'a, &'a Instr<'a>>,
    pub term: Terminator<'a>,
}

#[derive(Debug, Clone, Copy)]
pub enum Value<'a> {
    Arg(usize),
    Instr(&'a Instr<'a>),
}

#[derive(Debug, Clone, Copy)]
pub enum Instr<'a> {
    Lit, // Unit for now
    App { func: Value<'a>, args: &'a [Value<'a>] },
}

#[derive(Debug, Clone, Copy)]
pub enum Terminator<'a> {
    Return(Value<'a>),
    Jmp(&'a BB<'a>),
}
