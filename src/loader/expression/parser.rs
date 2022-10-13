use nom::IResult;

use super::ast::Expression;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Operator {
    Ternary,
    BAnd,
    BOr,
    BXor,
    Or,
    And,
    Eq,
    Neq,
    Starship,
    Lt,
    Gt,
    Gte,
    Lte,
    In,
    Matches,
    StartsWith,
    EndsWith,
    Range,
    Add,
    Sub,
    StrConcat,
    Mul,
    Div,
    Divi,
    Modulo,
    Is,
    Exp,
    NullCoal,
    Filter,
    ArrayIndex,
    Get,
}

pub fn parse(input: &str) -> IResult<&str, Expression> {
    todo!()
}
