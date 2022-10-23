use crate::loader::expression::ast::Expression;

use super::{
    environment::Env,
    value::{self, TaggedValue},
};

use anyhow::Result;

fn eval_expr(expr: Expression, env: Env) -> Result<TaggedValue> {
    Ok(TaggedValue::Str("foo".to_string()))
}
