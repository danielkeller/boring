use std::{fmt::Display, hash::Hash};

use bumpalo::collections::Vec;

use crate::pretty::Commas;

#[derive(Debug, Clone, Copy)]
pub struct Module<'a> {
    pub items: &'a [Item<'a>],
}
#[derive(Debug)]
pub struct Item<'a> {
    pub name: &'a str,
    pub body: Vec<'a, BB<'a>>,
}

impl Item<'_> {
    pub fn param(&self, mut bb: usize, mut param: usize) -> Value {
        loop {
            if bb == self.body[bb].idom {
                return Value(-1i32 - i32::try_from(param).unwrap());
            }
            bb = self.body[bb].idom;
            param += self.body[bb].n_params;
        }
    }
    pub fn instr(&self, mut bb: usize, mut instr: usize) -> Value {
        loop {
            if bb == self.body[bb].idom {
                return Value(i32::try_from(instr).unwrap());
            }
            bb = self.body[bb].idom;
            instr += self.body[bb].body.len();
        }
    }
}

#[derive(Debug)]
pub struct BB<'a> {
    pub n_params: usize,
    pub body: Vec<'a, Instr<'a>>,
    pub term: Terminator<'a>,
    pub idom: usize, // Can this be computed incrementally like this?
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Value(pub i32);

#[derive(Debug, Clone, Copy)]
pub enum Instr<'a> {
    Lit, // Unit for now
    Ref(Value),
    App { func: Value, args: &'a [Value] },
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Jmp<'a> {
    pub to: usize,
    pub args: &'a [Value],
}

#[derive(Debug, Clone, Copy, Default)]
pub enum Terminator<'a> {
    #[default]
    Halt,
    Return(Value),
    Jmp(Jmp<'a>),
    Switch(Value, &'a [Jmp<'a>]),
}

impl Display for Module<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for item in self.items {
            write!(f, "{item}\n\n")?;
        }
        Ok(())
    }
}
/*
 (\x -> (\y -> E(x, y)) Yi(x)) Xi
 (\ -> (\ -> E(2, 1)) Yi(1)) Xi
*/

impl Display for Item<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Item { name, body } = self;
        write!(f, "fn {name} {{\n")?;
        for (bb, body) in body.iter().enumerate() {
            let params = Commas((0..body.n_params).map(|p| self.param(bb, p)));
            write!(f, "  _bb{bb}({params}):\n")?;
            for (i, instr) in body.body.iter().enumerate() {
                let lhs = self.instr(bb, i);
                write!(f, "    {lhs} = {instr};\n")?;
            }
            match body.term {
                Terminator::Halt => f.write_str("    hlt;\n\n"),
                Terminator::Return(val) => {
                    write!(f, "    return {val};\n\n")
                }
                Terminator::Jmp(j) => write!(f, "    jmp {j};\n\n"),
                Terminator::Switch(val, js) => {
                    write!(f, "    switch {val}: {};\n\n", Commas(js))
                }
            }?;
        }
        f.write_str("}")
    }
}

impl Display for Jmp<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "_bb{}({})", self.to, Commas(self.args))
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0 < 0 {
            write!(f, "_a{}", -self.0 - 1)
        } else {
            write!(f, "_{}", self.0)
        }
    }
}

impl Display for Instr<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Instr::Lit => f.write_str("()"),
            &Instr::App { func, args } => {
                write!(f, "{func}({})", Commas(args))
            }
            Instr::Ref(v) => write!(f, "&{v}"),
        }
    }
}
