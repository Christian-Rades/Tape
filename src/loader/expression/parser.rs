use std::collections::VecDeque;

use nom::{IResult, combinator::map_res};

use anyhow::Result;

use super::{ast::{Expression, Term}, lexer::{Token, lex_exprs}};

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

pub fn parse(input: &str) -> IResult<&str, Expression> {
    map_res(lex_exprs, parse_to_expression)(input)
}

pub fn parse_to_expression(tokens: Vec<Token>) -> Result<Expression> {
    let mut tokens = VecDeque::from(tokens);
    parse_rec(&mut tokens, 0)
}

// see https://matklad.github.io/2020/04/13/simple-but-powerful-pratt-parsing.html
fn parse_rec(tokens: &mut VecDeque<Token>, min_bp: u8) -> Result<Expression> {
    let lhs = tokens.pop_front().unwrap_or_else(|| todo!("empty expr"));

    let mut lhs = match lhs {
        Token::Parens(par_tokens) => parse_to_expression(par_tokens)?,
        Token::Float(f) => Expression::Float(f),
        Token::Number(n) => Expression::Number(n),
        Token::Parent() => Expression::Parent,
        Token::Str(s) => Expression::Str(s),
        Token::Var(v) => Expression::Var(v),
        _ => todo!("lhs not an atom"),
    };

    loop {
        let op = match tokens.pop_front() {
            None => break,
            Some(Token::Op(op)) => op,
            x => todo!("two atoms next to eachother {:?}", x)
        };

        let (l_bp, r_bp) = op.bp_infix();

        if l_bp  < min_bp {
            break;
        }

        let rhs = parse_rec(tokens, r_bp)?;
        lhs = Expression::Term(Term{
            op,
            params: vec![lhs, rhs]
        })
    }

    Ok(lhs)
}

trait BindingPower {
    fn bp_infix (&self) -> (u8, u8);
}

impl BindingPower for Operator {
    fn bp_infix (&self) -> (u8, u8) {
        match self {
            Self::Get =>         (61, 60),
            &Self::ArrayIndex => unreachable!("operator is postfix"),
            Self::Filter =>      (59, 58),
            Self::NullCoal =>    (57, 56),
            Self::Exp =>         (55, 54),
            Self::Is =>          (53, 52),
            Self::Modulo =>      (51, 50),
            Self::Divi=>         (49, 48),
            Self::Div =>         (47, 46),
            Self::Mul =>         (45, 44),
            Self::StrConcat =>   (43, 42),
            Self::Sub =>         (41, 40),
            Self::Add =>         (39, 38),
            Self::Range =>       (37, 36),
            Self::EndsWith =>     (35, 34),
            Self::StartsWith =>  (33, 32),
            Self::Matches =>     (31, 30),
            Self::In =>          (29, 28),
            Self::Lte =>         (25, 24),
            Self::Gte =>         (23, 22),
            Self::Gt =>          (21, 20),
            Self::Lt =>          (19, 18),
            Self::Starship =>    (17, 16),
            Self::Neq =>         (15, 14),
            Self::Eq =>          (13, 12),
            Self::And =>         (11, 10),
            Self::Or =>          (9, 8),
            Self::BOr =>         (7, 6),
            Self::BXor =>        (5, 4),
            Self::BAnd =>        (3, 2),
            Self::Ternary =>     todo!("ternary not yet supported"),
            &Self::Not =>        unreachable!("operator is prefix")
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_infix_arithmetic() {
        let tokens = vec![
            Token::Number(1),
            Token::Op(Operator::Add),
            Token::Number(2),
            Token::Op(Operator::Mul),
            Token::Number(3)
        ];

        assert_eq!(parse_to_expression(tokens).unwrap(), Expression::Term(Term{
            op: Operator::Add,
            params: vec![
                Expression::Number(1),
                Expression::Term(Term{
                    op: Operator::Mul,
                    params: vec![
                        Expression::Number(2),
                        Expression::Number(3),
                    ]
                })
            ]
        }))
    }

    #[test]
    fn test_parenthesis() {
        let tokens = vec![
            Token::Number(1),
            Token::Op(Operator::Mul),
            Token::Parens(vec![
                Token::Number(2),
                Token::Op(Operator::Add),
                Token::Number(3)
            ])
        ];

        assert_eq!(parse_to_expression(tokens).unwrap(), Expression::Term(Term {
            op: Operator::Mul,
            params: vec![
                Expression::Number(1),
                Expression::Term(Term {
                    op: Operator::Add,
                    params: vec![
                        Expression::Number(2),
                        Expression::Number(3)
                    ]
                })
            ]
        }))
    }
}