use crate::loader::{expression::ast::Expression, Operator};

use super::{
    environment::Env,
    value::{self, TaggedValue},
};

use anyhow::{anyhow, Result};

pub trait Evaluate {
    fn eval(&self, env: &Env) -> Result<TaggedValue>;
}

trait Apply {
    fn apply(&self, params: Vec<TaggedValue>) -> Result<TaggedValue>;
}

impl Evaluate for Expression {
    fn eval(&self, env: &Env) -> Result<TaggedValue> {
        match self {
            Expression::Var(name) => Ok(env.get(name).unwrap_or_default()),
            Expression::Str(s) => Ok(TaggedValue::Str(s.to_string())),
            Expression::Number(n) => Ok(TaggedValue::Number(*n)),
            Expression::Float(f) => Ok(TaggedValue::Float(*f)),
            Expression::Bool(b) => Ok(TaggedValue::Bool(*b)),
            Expression::Term(term) => {
                let params: Result<Vec<TaggedValue>> =
                    term.params.iter().map(|p| p.eval(env)).collect();
                term.op.apply(params?)
            }
            _ => todo!("implement me: {:?}", self),
        }
    }
}

impl Apply for Operator {
    fn apply(&self, params: Vec<TaggedValue>) -> Result<TaggedValue> {
        match self {
            Self::Add => add(&params),
            Self::Mul => mul(&params),
            Self::Div => div(&params),
            Self::Divi => divi(&params),
            Self::And => and(&params),
            Self::Or => or(&params),
            Self::Not => not(&params),
            _ => Err(anyhow!("missing apply for operator: {:?}", self)),
        }
    }
}

fn add(params: &[TaggedValue]) -> Result<TaggedValue> {
    match params {
        [TaggedValue::Number(lhs), TaggedValue::Number(rhs)] => Ok(TaggedValue::Number(lhs + rhs)),
        [TaggedValue::Float(lhs), TaggedValue::Number(rhs)] => {
            Ok(TaggedValue::Float(lhs + *rhs as f64))
        }
        [TaggedValue::Number(lhs), TaggedValue::Float(rhs)] => {
            Ok(TaggedValue::Float(*lhs as f64 + rhs))
        }
        [TaggedValue::Float(lhs), TaggedValue::Float(rhs)] => Ok(TaggedValue::Float(lhs + rhs)),
        _ => Err(anyhow!("add not implemented for {:?}", params)),
    }
}

fn mul(params: &[TaggedValue]) -> Result<TaggedValue> {
    match params {
        [TaggedValue::Number(lhs), TaggedValue::Number(rhs)] => Ok(TaggedValue::Number(lhs * rhs)),
        [TaggedValue::Float(lhs), TaggedValue::Number(rhs)] => {
            Ok(TaggedValue::Float(lhs * *rhs as f64))
        }
        [TaggedValue::Number(lhs), TaggedValue::Float(rhs)] => {
            Ok(TaggedValue::Float(*lhs as f64 * rhs))
        }
        [TaggedValue::Float(lhs), TaggedValue::Float(rhs)] => Ok(TaggedValue::Float(lhs * rhs)),
        _ => Err(anyhow!("add not implemented for {:?}", params)),
    }
}

fn div(params: &[TaggedValue]) -> Result<TaggedValue> {
    match params {
        [TaggedValue::Number(lhs), TaggedValue::Number(rhs)] => {
            Ok(TaggedValue::Float(*lhs as f64 / *rhs as f64))
        }
        [TaggedValue::Float(lhs), TaggedValue::Number(rhs)] => {
            Ok(TaggedValue::Float(lhs / *rhs as f64))
        }
        [TaggedValue::Number(lhs), TaggedValue::Float(rhs)] => {
            Ok(TaggedValue::Float(*lhs as f64 / rhs))
        }
        [TaggedValue::Float(lhs), TaggedValue::Float(rhs)] => Ok(TaggedValue::Float(lhs / rhs)),
        _ => Err(anyhow!("add not implemented for {:?}", params)),
    }
}

fn divi(params: &[TaggedValue]) -> Result<TaggedValue> {
    match params {
        [TaggedValue::Number(lhs), TaggedValue::Number(rhs)] => Ok(TaggedValue::Number(lhs / rhs)),
        _ => Err(anyhow!("add not implemented for {:?}", params)),
    }
}

fn and(params: &[TaggedValue]) -> Result<TaggedValue> {
    match params {
        [TaggedValue::Bool(lhs), TaggedValue::Bool(rhs)] => Ok(TaggedValue::Bool(*lhs && *rhs)),
        _ => Err(anyhow!("add not implemented for {:?}", params)),
    }
}

fn or(params: &[TaggedValue]) -> Result<TaggedValue> {
    match params {
        [TaggedValue::Bool(lhs), TaggedValue::Bool(rhs)] => Ok(TaggedValue::Bool(*lhs || *rhs)),
        _ => Err(anyhow!("add not implemented for {:?}", params)),
    }
}

fn not(params: &[TaggedValue]) -> Result<TaggedValue> {
    match params {
        [TaggedValue::Bool(b)] => Ok(TaggedValue::Bool(*b == false)),
        _ => Err(anyhow!("add not implemented for {:?}", params)),
    }
}