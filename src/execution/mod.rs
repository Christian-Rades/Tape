use std::{fmt::{Write, Display}, collections::HashMap};

use ext_php_rs::{types::Zval, convert::FromZval, flags::DataType};

use crate::loader::ast::{Contents, Template, Content, Expression, Block, BlockType, IterationType};

use anyhow::{anyhow, Result, Context};

pub fn render(tpl: &Template, env: Env) -> Result<String> {
    let mut out_buf = String::default();
    tpl.render(&mut out_buf, env)?;
    Ok(out_buf)
}

pub struct Env {
    globals: Zval,
    stack: Vec<Scope>
}

type Scope = HashMap<String, InternalValue>;

pub enum InternalValue {
    Str(String),
    Zval(Zval),
    Usize(u64)
}

impl Display for InternalValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Str(s) => write!(f, "{}", &s),
            Self::Usize(us) => write!(f, "{}", us),
            Self::Zval(zv) => {
                // TODO check if this behaivior is ok
                write!(f, "{}", zv.str().unwrap_or(""))
            }
        }
    }
}

impl Clone for InternalValue {
    fn clone(&self) -> Self {
        match self {
            Self::Str(s) => Self::Str(s.clone()),
            Self::Usize(u) => Self::Usize(*u),
            Self::Zval(zv) => Self::Zval(zv.shallow_clone())
        }
    }
}

impl From<&str> for InternalValue  {
    fn from(s: &str) -> Self {
        InternalValue::Str(s.to_string())
    }
}

impl From<String> for InternalValue  {
    fn from(s: String) -> Self {
        InternalValue::Str(s)
    }
}

impl From<u64> for InternalValue {
    fn from(u: u64) -> Self {
        InternalValue::Usize(u)
    }
}

impl FromZval<'_> for InternalValue {
    const TYPE: ext_php_rs::flags::DataType = DataType::Mixed;
    fn from_zval(zval: & Zval) -> Option<Self> {
        Some(InternalValue::Zval(zval.shallow_clone()))
    }
}

trait Renderable {
    fn render<T: Write>(&self, out: &mut T, env: Env) -> Result<Env>;
}

impl Env {
    pub fn new(globals: Zval) -> Self {
        Self { globals, stack: vec![Scope::default()]}
    }

    pub fn enter_new_scope(mut self) -> Self {
        self.stack.push(Scope::default());
        self
    }
     pub fn exit_scope(mut self) -> Self {
         self.stack.pop();
         self
     }

    pub fn set(&mut self, name: &str, val: InternalValue) {
        let scope = self.stack.last_mut().expect("env is initialised with 1 scope");
        // TODO set values in parent scope
        scope.insert(name.to_string(), val);
    }

    pub fn get(&self, accessor: &str) -> Result<InternalValue> {
        if accessor.is_empty() {
            return Err(anyhow!("empty varname"));
        }

        if let Some(val) = self.get_from_scope(accessor) {
            return Ok(val);
        }

        match Self::get_rec(&self.globals, accessor) {
            Some(zv) => Ok(InternalValue::Zval(zv.shallow_clone())),
            None => Err(anyhow!("variable {} was not found", accessor))
        }
    }

    fn get_from_scope(&self, accessor: &str) -> Option<InternalValue> {
        let (key, rest) = if accessor.contains('.') {
            accessor.split_once('.').unwrap()
        } else {
            (accessor, "")
        };


        for scope in self.stack.iter().rev() {
            if let Some(val) = scope.get(key) {
                return match val {
                    InternalValue::Zval(zv) => {
                        Self::get_rec(zv, rest).and_then(InternalValue::from_zval)
                    },
                    _ => Some(val.clone())
                }
            }
        }
        None
    }

    fn get_rec<'a>(val: &'a Zval, accessor: &'_ str) -> Option<&'a Zval> {
        if accessor.is_empty() {
            return Some(val);
        }
        let (key, rest) = if accessor.contains('.') {
            accessor.split_once('.').unwrap()
        } else {
            (accessor, "")
        };

        if val.is_array() {
            let array = val.array()?;
            return Self::get_rec(array.get(key)?, rest);
        }

        if val.is_object() {
            let obj = val.object()?;
            return Self::get_rec(obj.get_property(key).ok()?, rest)
        }
        None
    }
}

impl Renderable for Template {
    fn render<T: Write>(&self, out: &mut T, env: Env) -> Result<Env> {
        self.content.render(out, env)
    }
}

impl Renderable for Contents {
   fn render<T: Write>(&self, out: &mut T, env: Env) -> Result<Env> {
       let mut env = env;
       for c in self.iter() {
           env = c.render(out, env)?
       }
       Ok(env)
   } 
}

impl Renderable for Content {
    fn render<T: Write>(&self, out: &mut T, env: Env) -> Result<Env> {
        match self {
            Content::Text(str) => { write!(out, "{}", str)?; Ok(env)},
            Content::Print(expr) => expr.render(out, env),
            Content::Block(block) => block.render(out, env),
            Content::Statement(_) => Ok(env),
        }
    }
}

impl Renderable for Expression {
    fn render<T: Write>(&self, out: &mut T, env: Env) -> Result<Env> {
        match self {
            Expression::Str(str) => write!(out, "{}", str)?,
            Expression::Var(var_name) => write!(out, "{}", env.get(var_name)?)?,
            _ => todo!(),
        }
        Ok(env)
    }
}

impl Renderable for Block {
    fn render<T: Write>(&self, out: &mut T, env: Env) -> Result<Env> {
        let mut env = env.enter_new_scope();
        match &self.typ {
            BlockType::BlockName(_) => {
                self.contents.render(out, env).map(Env::exit_scope)
            },
            BlockType::Loop(l) => {
                let zv = if let InternalValue::Zval(zv) = env.get(&l.iterator)? {
                    zv
                } else {
                    return Err(anyhow!("variable {} is not iterable", &l.iterator))
                };
                let collection = zv.array().with_context(|| format!("variable {}, is not iterable", &l.iterator))?;

                for (idx, key, val) in collection.iter() {
                    match &l.typ {
                        IterationType::SingleVal(name) => {
                            env.set(name, InternalValue::from_zval(val).expect("php vm broke"))
                        },
                        IterationType::KeyVal((kname, vname)) => {
                            env.set(kname, key.map_or_else(|| idx.into(), InternalValue::from));
                            env.set(vname, InternalValue::from_zval(val).expect("php vm broke"));
                        }
                    };

                    env = self.contents.render(out, env)?
                };
                Ok(env)
            }
        }
    }
}

