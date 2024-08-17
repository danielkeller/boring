use std::{collections::HashMap, fmt::Display, hash::Hash};

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

#[derive(Debug)]
pub struct BB<'a> {
    pub body: Vec<'a, Instr<'a>>,
    pub term: Terminator,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Value(pub i32);

#[derive(Debug, Clone, Copy)]
pub enum Instr<'a> {
    Lit, // Unit for now
    App { func: Value, args: &'a [Value] },
}

#[derive(Debug, Clone, Copy)]
pub enum Terminator {
    Return(Value),
    Jmp(u32),
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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Item { name, body } = self;
        write!(f, "fn {name} {{\n")?;
        for (i, bb) in body.iter().enumerate() {
            write!(f, "  _bb{i}:\n{bb}\n")?;
        }
        f.write_str("}")
    }
}

impl Display for BB<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, instr) in self.body.iter().enumerate() {
            write!(f, "    _{i} = {instr};\n")?;
        }
        match self.term {
            Terminator::Return(val) => {
                write!(f, "    return {val};\n")
            }
            Terminator::Jmp(bb) => write!(f, "    jmp _bb{bb};\n"),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0 < 0 {
            write!(f, "_a{}", -self.0)
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
        }
    }
}
