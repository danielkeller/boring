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

impl PartialEq for Value<'_> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Arg(ln), Self::Arg(rn)) => ln == rn,
            (Self::Instr(li), Self::Instr(ri)) => std::ptr::eq(*li, *ri),
            _ => false,
        }
    }
}

impl Eq for Value<'_> {}

impl Hash for Value<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Value::Arg(n) => n.hash(state),
            Value::Instr(i) => std::ptr::hash(*i, state),
        }
    }
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
        for bb in body {
            write!(f, "{bb}\n")?;
        }
        f.write_str("}")
    }
}

impl Display for BB<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut ssa = HashMap::new();
        f.write_str("    {\n")?;
        for (i, &instr) in self.body.iter().enumerate() {
            write!(f, "        _{i} = ")?;
            fmt_instr(instr, &ssa, f)?;
            write!(f, ";\n")?;
            ssa.insert(Value::Instr(instr), i);
        }
        match self.term {
            Terminator::Return(val) => {
                write!(f, "        return {};\n    }}", fmt_val(&val, &ssa))
            }
            Terminator::Jmp(_) => write!(f, "        jmp...\n    }}"),
        }
    }
}

fn fmt_val(value: &Value, ssa: &HashMap<Value, usize>) -> String {
    match value {
        Value::Arg(n) => format!("_a{n}"),
        i => format!("_{}", ssa.get(&i).unwrap_or(&9999)),
    }
}

fn fmt_instr(
    instr: &Instr, ssa: &HashMap<Value, usize>, f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    let fmt_val = |val| fmt_val(val, ssa);
    match instr {
        Instr::Lit => f.write_str("()"),
        Instr::App { func, args } => {
            let func = fmt_val(func);
            let args = args.iter().map(fmt_val);
            write!(f, "{}({})", func, Commas(args))
        }
    }
}
