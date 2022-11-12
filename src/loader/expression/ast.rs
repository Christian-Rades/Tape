use super::parser::Operator;

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Term(Term),
    Str(String),
    Var(String),
    Number(i64),
    Float(f64),
    Bool(bool),
    Null,
    Array(Vec<Expression>),
    FuncCall(FuncCall),
    FilterCall(FuncCall),
    HashMap(Vec<KeyValuePair>),
    Parent,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Term {
    pub op: Operator,
    pub params: Vec<Expression>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct FuncCall {
    pub name: String,
    pub params: Vec<Expression>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct KeyValuePair {
    pub key: Expression,
    pub val: Expression,
}
