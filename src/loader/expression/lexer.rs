use nom::{IResult, multi::{separated_list1, many_till, separated_list0}, character::complete::{multispace1, digit1, one_of, multispace0}, branch::alt, bytes::complete::{tag, take_while1, take_while}, sequence::{tuple, delimited, separated_pair}, number::complete::float, combinator::{map_res, not}};

use super::parser::Operator;

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Str(String),
    Var(String),
    Number(i64),
    Float(f32),
    Array(Vec<Token>),
    HashMap(Vec<KVTokensPair>),
    Parens(Vec<Token>),
    Op(Operator),
    Parent() //TODO remove
}

#[derive(Debug, PartialEq, Clone)]
pub struct KVTokensPair {
    pub key: Vec<Token>,
    pub value: Vec<Token>,
}

fn lex_exprs(i: &str) -> IResult<&str, Vec<Token>> {
    let (rest, exprs) = separated_list1(multispace1, lex_expr)(i)?;
    Ok((rest, exprs))
}

fn lex_expr(i: &str) -> IResult<&str, Token> {
    alt((lex_parent_call, lex_hash_map, lex_parens, lex_array, lex_number, lex_float, lex_string_literal, lex_var))(i)
}

fn lex_parent_call(i: &str) -> IResult<&str, Token> {
    let (rest, _) = tag::<&str, &str, nom::error::Error<&str>>("parent()", /* value */)(i)?;
    Ok((rest, Token::Parent()))
}

fn lex_string_literal(i: &str) -> IResult<&str, Token> {
    let (rest, plain_str) = lex_quoted(i)?;
    Ok((rest, Token::Str(plain_str.to_string())))
}

fn lex_quoted(i: &str) -> IResult<&str, &str> {
    let result = alt((
            delimited(nom::character::complete::char('\''), take_while(|c| c != '\''), nom::character::complete::char('\'')),
            delimited(nom::character::complete::char('"'), take_while(|c| c != '"'), nom::character::complete::char('"')),
            ))(i)?;
    Ok(result)
}

fn lex_var(i: &str) -> IResult<&str, Token> {
    let is_identifier = |c| -> bool {
        ('a'..='z').contains(&c) || ('A'..='Z').contains(&c) || c == '_' || (0x7f as char <= c && c <= 0xff as char)
    };
    let (rest, (part1, part2)) = dbg!(tuple((take_while1(is_identifier), take_while(|c| is_identifier(c) || ('0'..='9').contains(&c) || c == '.')))(i))?;
    let mut accessor = part1.to_string();
    accessor.push_str(part2.trim());
    Ok((rest, Token::Var(accessor)))
}

fn lex_float(i: &str) -> IResult<&str, Token> {
    let (rest, f) = float(i)?;
    Ok((rest, Token::Float(f)))
}

fn lex_number(i: &str) -> IResult<&str, Token> {
    //TODO add negative numbers
    let (rest, number) = map_res(tuple((digit1, not(one_of("e.")))), |(number,..)| str::parse(number))(i)?;
    Ok((rest, Token::Number(number)))
}

fn lex_parens(i: &str) -> IResult<&str, Token> {
    let (rest, (.., (child_exprs, ..))) = tuple((nom::character::complete::char('('), many_till(lex_expr, tuple((multispace0, nom::character::complete::char(')'))))))(i)?;
    Ok((rest, Token::Parens(child_exprs)))
}

fn lex_array(i: &str) -> IResult<&str, Token> {
    let (rest, elems) = delimited(tuple((tag("["), multispace0)), separated_list0(tuple((multispace0, tag(","), multispace0)), lex_expr), tuple((multispace0,tag("]"))))(i)?;
    Ok((rest, Token::Array(elems)))
}

fn lex_hash_map(i: &str) -> IResult<&str, Token> {
    let (rest, kv_pairs) = delimited(tuple((tag("{"), multispace0)), separated_list0(tuple((multispace0,tag(","), multispace0)), lex_key_value_pair), tuple((multispace0, tag("}"))))(i)?;
    Ok((rest, Token::HashMap(kv_pairs)))
}

fn lex_key_value_pair(i: &str) -> IResult<&str, KVTokensPair> {
    let (rest, (key, value)) = separated_pair(alt((lex_parens, lex_string_literal, lex_var)), tuple((multispace0,tag(":"),multispace0)), lex_exprs)(i)?;
    //hash keys are allowed to be unqouted
    let key = match key {
        Token::Var(v) => vec![Token::Str(v)],
        Token::Str(a) => vec![Token::Str(a)],
        Token::Parens(tokens) => tokens,
        _ => todo!()
    };
    Ok((rest, KVTokensPair{key, value}))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_lex_var() {
        let var = "foo.bar";
        assert_eq!(lex_var(var), Ok(("", Token::Var("foo.bar".to_string()))))
    }

    #[test]
    fn test_lex_str()  {
        let single_quote = "'foo'";
        let double_quote = r#""foo""#;

        assert_eq!(
            lex_string_literal(single_quote),
            Ok(("", Token::Str("foo".to_string())))
        );

        assert_eq!(
            lex_string_literal(double_quote),
            Ok(("", Token::Str("foo".to_string())))
        );
    }

    #[test]
    fn test_lex_array() {
        let arr = "[ var, ',str',1]";

        assert_eq!(
            lex_array(arr),
            Ok(("", Token::Array(vec![
                Token::Var("var".to_string()),
                Token::Str(",str".to_string()),
                Token::Number(1)
            ])))
        )
    }

    #[test]
    fn test_lex_hashmap() {
        let hm = "{ key:'bar','key1' : var, (var): 1}";
        assert_eq!(
            lex_hash_map(hm),
            Ok(("", Token::HashMap(vec![
                KVTokensPair {
                    key: vec![Token::Str("key".to_string())],
                    value: vec![Token::Str("bar".to_string())]
                },
                KVTokensPair {
                    key: vec![Token::Str("key1".to_string())],
                    value: vec![Token::Var("var".to_string())]
                },
                KVTokensPair {
                    key: vec![Token::Var("var".to_string())],
                    value: vec![Token::Number(1)]
                }
            ])))
        )
    }
}