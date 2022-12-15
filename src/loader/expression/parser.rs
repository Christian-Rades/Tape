use std::collections::VecDeque;

use nom::{combinator::map_res, IResult};

use anyhow::{anyhow, Result};

use crate::loader::{expression::ast::FuncCall, Span};

use super::{
    ast::{Expression, Term, KeyValuePair},
    lexer::{lex_exprs, Token},
};

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
    Not,
}

pub fn parse(input: Span) -> IResult<Span, Expression> {
    map_res(lex_exprs, parse_to_expression)(input)
}

pub fn parse_to_expression(tokens: Vec<Token>) -> Result<Expression> {
    let mut tokens = VecDeque::from(tokens);
    parse_rec(&mut tokens, 0)
}

// see https://matklad.github.io/2020/04/13/simple-but-powerful-pratt-parsing.html
fn parse_rec(tokens: &mut VecDeque<Token>, min_bp: u8) -> Result<Expression> {
    let lhs = tokens.pop_front().unwrap_or(Token::Null);

    let mut lhs = match lhs {
        Token::Null => {
            return Ok(Expression::Null);
        }
        Token::Parens(par_tokens) => parse_to_expression(par_tokens)?,
        Token::Float(f) => Expression::Float(f),
        Token::Number(n) => Expression::Number(n),
        Token::Parent() => Expression::Parent,
        Token::Str(s) => Expression::Str(s),
        Token::Var(v) => Expression::Var(v),
        Token::Bool(b) => Expression::Bool(b),

        Token::Array(toks) => Expression::Array(toks
                              .into_iter()
                              .map(parse_to_expression)
                              .collect::<Result<Vec<Expression>>>()?),

        Token::HashMap(kvs) => Expression::HashMap(kvs.into_iter().map(|kv_pair| -> Result<KeyValuePair> {
            Ok(KeyValuePair{
                key: parse_to_expression(kv_pair.key)?,
                val: parse_to_expression(kv_pair.value)?
            })
        }).collect::<Result<Vec<KeyValuePair>>>()?),

        Token::FuncCall(fc) => Expression::FuncCall(FuncCall {
            name: fc.name,
            params: fc
                .params
                .into_iter()
                .map(parse_to_expression)
                .collect::<Result<Vec<Expression>>>()?,
        }),

        Token::Op(op) => {
            if let Some(bp) = op.bp_prefix() {
                Expression::Term(Term {
                    op,
                    params: vec![parse_rec(tokens, bp)?],
                })
            } else {
                return Err(anyhow!("not a prefix op: {:?}", op));
            }
        }

        _ => todo!("lhs not an atom"),
    };
    loop {
        let op = match tokens.pop_front() {
            None => break,
            Some(Token::Op(op)) => op,
            Some(x) => todo!("two atoms next to eachother {:?} {:?}", lhs, x),
        };

        let (l_bp, r_bp) = op.bp_infix();

        if l_bp < min_bp {
            tokens.push_front(Token::Op(op));
            break;
        }

        if op == Operator::Filter {
            let Some(filter) = tokens.pop_front() else {
                return Err(anyhow!("unexpected end of expression"));
            };

            lhs = match filter {
                Token::Var(name) => Expression::FilterCall(FuncCall {
                    name,
                    params: vec![lhs],
                }),
                Token::FuncCall(fc) => {
                    let super::lexer::FuncCall { name, params } = fc;
                    let mut params: Vec<Expression> = params
                        .into_iter()
                        .map(parse_to_expression)
                        .collect::<Result<Vec<Expression>>>()?;

                    params.insert(0, lhs);

                    Expression::FilterCall(FuncCall { name, params })
                }
                _ => return Err(anyhow!("illegal filter name: {:?}", filter)),
            };

            break;
        }

        let rhs = parse_rec(tokens, r_bp)?;
        lhs = Expression::Term(Term {
            op,
            params: vec![lhs, rhs],
        })
    }

    Ok(lhs)
}

trait BindingPower {
    fn bp_infix(&self) -> (u8, u8);
    fn bp_prefix(&self) -> Option<u8>;
}

impl BindingPower for Operator {
    fn bp_infix(&self) -> (u8, u8) {
        match self {
            Self::Get => (63, 62),
            &Self::ArrayIndex => unreachable!("operator is postfix"),
            Self::Filter => (61, 60),
            Self::NullCoal => (59, 58),
            Self::Exp => (57, 56),
            Self::Is => (55, 54),
            Self::Modulo => (53, 52),
            Self::Divi => (51, 50),
            Self::Div => (49, 48),
            Self::Mul => (47, 46),
            Self::StrConcat => (45, 44),
            Self::Sub => (43, 42),
            Self::Add => (41, 40),
            Self::Range => (39, 38),
            Self::EndsWith => (37, 36),
            Self::StartsWith => (35, 34),
            Self::Matches => (33, 32),
            Self::In => (31, 30),
            Self::Lte => (27, 26),
            Self::Gte => (25, 24),
            Self::Gt => (23, 22),
            Self::Lt => (21, 20),
            Self::Starship => (19, 18),
            Self::Not => unreachable!("operator is prefix"),
            Self::Neq => (15, 14),
            Self::Eq => (13, 12),
            Self::And => (11, 10),
            Self::Or => (9, 8),
            Self::BOr => (7, 6),
            Self::BXor => (5, 4),
            Self::BAnd => (3, 2),
            Self::Ternary => todo!("ternary not yet supported"),
        }
    }

    fn bp_prefix(&self) -> Option<u8> {
        match self {
            Self::Not => Some(16),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::loader::expression::{ast::FuncCall, lexer};

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_infix_arithmetic() {
        let tokens = vec![
            Token::Number(1),
            Token::Op(Operator::Add),
            Token::Number(2),
            Token::Op(Operator::Mul),
            Token::Number(3),
        ];

        assert_eq!(
            parse_to_expression(tokens).unwrap(),
            Expression::Term(Term {
                op: Operator::Add,
                params: vec![
                    Expression::Number(1),
                    Expression::Term(Term {
                        op: Operator::Mul,
                        params: vec![Expression::Number(2), Expression::Number(3),]
                    })
                ]
            })
        )
    }

    #[test]
    fn test_parenthesis() {
        let tokens = vec![
            Token::Number(1),
            Token::Op(Operator::Mul),
            Token::Parens(vec![
                Token::Number(2),
                Token::Op(Operator::Add),
                Token::Number(3),
            ]),
        ];

        assert_eq!(
            parse_to_expression(tokens).unwrap(),
            Expression::Term(Term {
                op: Operator::Mul,
                params: vec![
                    Expression::Number(1),
                    Expression::Term(Term {
                        op: Operator::Add,
                        params: vec![Expression::Number(2), Expression::Number(3)]
                    })
                ]
            })
        )
    }

    #[test]
    fn test_function_call() {
        let tokens = vec![Token::FuncCall(lexer::FuncCall {
            name: "foo".to_string(),
            params: vec![vec![
                Token::Number(1),
                Token::Op(Operator::Add),
                Token::Number(2),
            ]],
        })];

        assert_eq!(
            parse_to_expression(tokens).unwrap(),
            Expression::FuncCall(FuncCall {
                name: "foo".to_string(),
                params: vec![Expression::Term(Term {
                    op: Operator::Add,
                    params: vec![Expression::Number(1), Expression::Number(2)]
                })]
            })
        )
    }

    #[test]
    fn test_not() {
        let tokens = vec![
            Token::Op(Operator::Not),
            Token::Number(2),
            Token::Op(Operator::Lte),
            Token::Number(3),
            Token::Op(Operator::And),
            Token::Number(4),
            Token::Op(Operator::Gte),
            Token::Number(5),
        ];

        assert_eq!(
            parse_to_expression(tokens).unwrap(),
            Expression::Term(Term {
                op: Operator::And,
                params: vec![
                    Expression::Term(Term {
                        op: Operator::Not,
                        params: vec![Expression::Term(Term {
                            op: Operator::Lte,
                            params: vec![Expression::Number(2), Expression::Number(3),]
                        }),]
                    }),
                    Expression::Term(Term {
                        op: Operator::Gte,
                        params: vec![Expression::Number(4), Expression::Number(5)]
                    })
                ]
            })
        )
    }
}
