pub mod environment;
mod value;
use std::{fmt::Write, collections::HashMap};

use ext_php_rs::convert::FromZval;

use crate::loader::{ast::{Contents, Template, Content, ExpressionAtom, Block, BlockType, IterationType, Stmt, Expression}, Module, Extension};

use anyhow::{anyhow, Result, Context};

use self::environment::{Env};
use self::value::TaggedValue;

pub fn render(mut tpl: Module, mut env: Env) -> Result<String> {

    let mut block_extensions: HashMap<String, Box<Block>> = HashMap::default();

    while let Module::Extension(Extension{parent, blocks, ..}) = tpl {
        for (name, block) in blocks.into_iter() {
            match block_extensions.get_mut(&name) {
                None => {block_extensions.insert(name, block);},
                Some(child_block) =>  {
                    child_block.set_parents(block)
                }
            }
        }
        tpl = env.load_file(parent)?;
    }

    match tpl {
        Module::Template(mut base) => {
            let mut out_buf = String::default();
            base.apply_extensions(block_extensions);
            base.render(&mut out_buf, env)?;
            Ok(out_buf)
        },
        _ => unreachable!()
    }
 
}



trait Renderable {
    fn render<T: Write>(&self, out: &mut T, env: Env) -> Result<Env>;
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
    fn render<T: Write>(&self, out: &mut T,mut  env: Env) -> Result<Env> {
        match self {
            Content::Text(str) => { write!(out, "{}", str)?; Ok(env)},
            Content::Print(expr) => expr.render(out, env),
            Content::Block(block) => block.render(out, env),
            Content::Statement(Stmt::Set(setter)) =>{
                env.apply_setter(setter);
                Ok(env)
            },
            Content::Statement(Setter) => Ok(env),
        }
    }
}

impl Renderable for ExpressionAtom {
    fn render<T: Write>(&self, out: &mut T, env: Env) -> Result<Env> {
        match self {
            ExpressionAtom::Str(str) => write!(out, "{}", str)?,
            ExpressionAtom::Var(var_name) => write!(out, "{}", env.get(var_name).unwrap_or_default())?,
            _ => todo!(),
        }
        Ok(env)
    }
}

impl Renderable for Expression {
    fn render<T: Write>(&self, out: &mut T, env: Env) -> Result<Env> {
        match self {
            Self::Atom(a) => a.render(out, env),
            _ => todo!()
        }
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
                let zv = if let TaggedValue::Zval(zv) = env.get(&l.iterator)? {
                    zv
                } else {
                    return Err(anyhow!("variable {} is not iterable", &l.iterator))
                };
                let collection = zv.array().with_context(|| format!("variable {}, is not iterable", &l.iterator))?;

                for (idx, key, val) in collection.iter() {
                    match &l.typ {
                        IterationType::SingleVal(name) => {
                            env.set(name, TaggedValue::from_zval(val).expect("php vm broke"))
                        },
                        IterationType::KeyVal((kname, vname)) => {
                            env.set(kname, key.map_or_else(|| idx.into(), TaggedValue::from));
                            env.set(vname, TaggedValue::from_zval(val).expect("php vm broke"));
                        }
                    };

                    env = self.contents.render(out, env)?
                };
                Ok(env)
            }
        }
    }
}

