use std::collections::VecDeque;

use super::ast::{ExpressionToken, Expression, ExpressionAtom, OperatorToken, Term};


pub fn parse_to_expression(tokens: Vec<ExpressionToken>) -> Expression {
    let mut tokens = VecDeque::from(tokens);
    parse_rec(&mut tokens, 0)
}

// see https://matklad.github.io/2020/04/13/simple-but-powerful-pratt-parsing.html
fn parse_rec(tokens: &mut VecDeque<ExpressionToken>, min_bp: u8) -> Expression {
    let lhs = tokens.pop_front().unwrap_or_else(|| todo!("empty expr"));

    let mut lhs = match lhs {
        ExpressionToken::Atom(ExpressionAtom::Parens(par_tokens)) => parse_to_expression(par_tokens),
        ExpressionToken::Atom(a) => Expression::Atom(a),
        _ => todo!("lhs not an atom"),
    };

    loop {
        let op = match tokens.pop_front() {
            None => break,
            Some(ExpressionToken::Op(op)) => op,
            x => todo!("two atoms next to eachother {:?}", x)
        };

        let (l_bp, r_bp) = op.bp_infix();

        if l_bp  < min_bp {
            break;
        }

        let rhs = parse_rec(tokens, r_bp);
        lhs = Expression::Term(Term{
            op,
            params: vec![lhs, rhs]
        })
    }

    lhs
}

trait BindingPower {
    fn bp_infix (&self) -> (u8, u8);
}

impl BindingPower for OperatorToken {
    fn bp_infix (&self) -> (u8, u8) {
        match self {
            Self::Get =>        (55,54),
            Self::Filter =>     (53,52),
            Self::Exp =>        (51,50),
            Self::Is =>         (49,48),
            Self::Modulo =>     (47,46),
            Self::Divi=>        (45,44),
            Self::Div =>        (43,42),
            Self::Mul =>        (41,40),
            Self::StrConcat =>  (39,38),
            Self::Sub =>        (37,36),
            Self::Add =>        (35,34),
            Self::Range =>      (33,32),
            Self::EndWith =>    (31,30),
            Self::StartsWith => (25,24),
            Self::Matches =>    (29,28),
            Self::In =>         (27,26),
            Self::Lte =>        (23,22),
            Self::Gte =>        (21,20),
            Self::Gt =>         (19,18),
            Self::Lt =>         (17,16),
            Self::Starship =>   (15,14),
            Self::Neq =>        (11,10),
            Self::Eq =>         (13,12),
            Self::And =>        (9,8),
            Self::Or =>         (7,6),
            Self::BOr =>        (5,4),
            Self::BXor =>       (3,2),
            Self::BAnd =>       (1,0),
            _ => unreachable!()
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::loader::ast::{OperatorToken, Term};

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_parse_single_atom() {
        for atom  in (vec![
                      ExpressionAtom::Str("foo".to_string()),
                      ExpressionAtom::Number(1),
                      ExpressionAtom::Float(1.0),
                      ExpressionAtom::Parent(),
                      ExpressionAtom::Var("foo".to_string())
        ]).into_iter() {
            assert_eq!(parse_to_expression(vec![ExpressionToken::Atom(atom.clone())]), Expression::Atom(atom))
        }
    }

    #[test]
    fn test_infix_arithmetic() {
        let tokens = vec![
            t_atom(ExpressionAtom::Number(1)),
            t_op(OperatorToken::Add),
            t_atom(ExpressionAtom::Number(2)),
            t_op(OperatorToken::Mul),
            t_atom(ExpressionAtom::Number(3))
        ];

        assert_eq!(parse_to_expression(tokens), Expression::Term(Term{
            op: OperatorToken::Add,
            params: vec![
                Expression::Atom(ExpressionAtom::Number(1)),
                Expression::Term(Term{
                    op: OperatorToken::Mul,
                    params: vec![
                        Expression::Atom(ExpressionAtom::Number(2)),
                        Expression::Atom(ExpressionAtom::Number(3)),
                    ]
                })
            ]
        }))
    }

    #[test]
    fn test_parenthesis() {
        let tokens = vec![
            t_atom(ExpressionAtom::Number(1)),
            t_op(OperatorToken::Mul),
            t_atom(ExpressionAtom::Parens(vec![
                t_atom(ExpressionAtom::Number(2)),
                t_op(OperatorToken::Add),
                t_atom(ExpressionAtom::Number(3))
            ])
            )
        ];

        assert_eq!(parse_to_expression(tokens), Expression::Term(Term {
            op: OperatorToken::Mul,
            params: vec![
                Expression::Atom(ExpressionAtom::Number(1)),
                Expression::Term(Term {
                    op: OperatorToken::Add,
                    params: vec![
                        Expression::Atom(ExpressionAtom::Number(2)),
                        Expression::Atom(ExpressionAtom::Number(3))
                    ]
                })
            ]
        }))
    }

    fn t_atom(a: ExpressionAtom) -> ExpressionToken {
        ExpressionToken::Atom(a)
    }
    fn t_op(o: OperatorToken) -> ExpressionToken {
        ExpressionToken::Op(o)
    }
}
