use crate::loader::expression::ast::Expression;

use super::{
    environment::Env,
    value::{self, TaggedValue},
};

use anyhow::Result;

pub trait Evaluate {
    fn eval(&self, env: &Env) -> Result<TaggedValue>;
}

impl Evaluate for Expression {
    fn eval(&self, env: &Env) -> Result<TaggedValue> { 
        match self {
            Expression::Var(name) => env.get(name),
            Expression::Str(s)  => Ok(TaggedValue::Str(s.to_string())),
            _ => todo!("implement me: {:?}", self)
        }
    }
}