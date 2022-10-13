use super::{
    ast::{
        get_blocks, Block, BlockType, Content, Extension, IterationType, Loop, Module, Setter,
        Stmt, Template,
    },
    expression,
};

use std::collections::HashMap;

use anyhow::{anyhow, Result};
use nom::{
    branch::alt,
    bytes::{
        complete::{tag, take_till, take_while, take_while1},
        streaming::take_until,
    },
    character::complete::{digit1, line_ending, multispace0, multispace1, one_of, space0},
    combinator::{eof, map_res, not, opt},
    multi::{many_till, separated_list0, separated_list1},
    number::complete::float,
    sequence::{delimited, separated_pair, tuple},
    IResult,
};

pub fn parse(name: String, input: &str) -> Result<Module> {
    if let Ok((rest, parent)) = parse_extends(input) {
        match parse_contents(rest) {
            Ok((_, content)) => {
                let ext = Extension {
                    name,
                    parent,
                    blocks: get_blocks(content, HashMap::default()),
                };
                Ok(Module::Extension(ext))
            }
            Err(err) => Err(anyhow!("error parsing {}: {}", name, err)),
        }
    } else {
        match parse_contents(input) {
            Ok((_, content)) => Ok(Module::Template(Template { name, content })),
            Err(err) => Err(anyhow!("error parsing {}: {}", name, err)),
        }
    }
}

fn parse_extends(i: &str) -> IResult<&str, String> {
    let (rest, (.., parent)) = delimited(
        parse_block_tag_l,
        tuple((tag("extends"), multispace1, parse_quoted)),
        parse_block_tag_r,
    )(i)?;
    Ok((rest, parent.to_string()))
}

fn parse_block_tag_l(i: &str) -> IResult<&str, ()> {
    let (rest, _) = tuple((space0, tag("{%"), multispace1))(i)?;
    Ok((rest, ()))
}

fn parse_block_tag_r(i: &str) -> IResult<&str, ()> {
    let (rest, _) = tuple((multispace0, tag("%}"), opt(line_ending)))(i)?;
    Ok((rest, ()))
}

fn parse_quoted(i: &str) -> IResult<&str, &str> {
    let result = alt((
        delimited(
            nom::character::complete::char('\''),
            take_while(|c| c != '\''),
            nom::character::complete::char('\''),
        ),
        delimited(
            nom::character::complete::char('"'),
            take_while(|c| c != '"'),
            nom::character::complete::char('"'),
        ),
    ))(i)?;
    Ok(result)
}

fn parse_contents(i: &str) -> IResult<&str, Vec<Content>> {
    let (_, (contents, _)) = many_till(parse_content, eof)(i)?;
    Ok(("", contents))
}

fn parse_content(i: &str) -> IResult<&str, Content> {
    alt((parse_print, parse_statement, parse_block, parse_text))(i)
}

fn parse_text(i: &str) -> IResult<&str, Content> {
    let (rest, text) = take_while1(|c| c != '{')(i)?;
    Ok((rest, Content::Text(text.to_string())))
}

fn parse_print(i: &str) -> IResult<&str, Content> {
    let (rest, exprs) = delimited(parse_print_tag_l, expression::parse, parse_print_tag_r)(i)?;
    Ok((rest, Content::Print(exprs)))
}

fn parse_print_tag_l(i: &str) -> IResult<&str, ()> {
    let (rest, _) = tuple((tag("{{"), multispace1))(i)?;
    Ok((rest, ()))
}

fn parse_print_tag_r(i: &str) -> IResult<&str, ()> {
    let (rest, _) = tuple((multispace0, tag("}}")))(i)?;
    Ok((rest, ()))
}

fn parse_statement(i: &str) -> IResult<&str, Content> {
    let (rest, statement) = delimited(
        parse_block_tag_l,
        alt((parse_set_statement, parse_include_statement)),
        parse_block_tag_r,
    )(i)?;
    Ok((rest, Content::Statement(statement)))
}

fn parse_set_statement(i: &str) -> IResult<&str, Stmt> {
    let (rest, (.., target, _, _, expr)) = tuple((
        tag("set"),
        multispace1,
        take_till(|c| c == '='),
        nom::character::complete::char('='),
        multispace0,
        expression::parse,
    ))(i)?;
    Ok((
        rest,
        Stmt::Set(Setter {
            target: target.trim().to_string(),
            value: expr,
        }),
    ))
}

fn parse_include_statement(i: &str) -> IResult<&str, Stmt> {
    let (rest, (.., target)) = tuple((tag("include"), multispace1, parse_quoted))(i)?;
    Ok((rest, Stmt::Include(target.to_string())))
}

fn parse_block(i: &str) -> IResult<&str, Content> {
    let (rest, typ) = parse_block_type(i)?;
    match typ {
        BlockType::BlockName(_) => {
            let (rest, (contents, _)) = many_till(
                parse_content,
                tuple((tag("{% endblock %}"), opt(line_ending))),
            )(rest)?;
            Ok((rest, Content::Block(Box::new(Block { typ, contents }))))
        }
        BlockType::Loop(_) => {
            let (rest, (contents, _)) = many_till(
                parse_content,
                tuple((tag("{% endfor %}"), opt(line_ending))),
            )(rest)?;
            Ok((rest, Content::Block(Box::new(Block { typ, contents }))))
        }
    }
}

fn parse_block_type(i: &str) -> IResult<&str, BlockType> {
    delimited(
        parse_block_tag_l,
        alt((parse_block_name, parse_loop)),
        parse_block_tag_r,
    )(i)
}

fn parse_block_name(i: &str) -> IResult<&str, BlockType> {
    let (rest, (.., name)) = tuple((tag("block"), multispace1, (take_till(|c| c == ' '))))(i)?;
    Ok((rest, BlockType::BlockName(name.to_string())))
}

fn parse_loop(i: &str) -> IResult<&str, BlockType> {
    let (rest, (.., iter_type, _, _, iterator)) = tuple((
        tag("for"),
        multispace1,
        alt((parse_key_value, parse_single_var)),
        tag("in"),
        multispace1,
        take_till(|c| c == ' '),
    ))(i)?;
    Ok((
        rest,
        BlockType::Loop(Loop {
            typ: iter_type,
            iterator: iterator.to_string(),
        }),
    ))
}

fn parse_single_var(i: &str) -> IResult<&str, IterationType> {
    let (rest, varname) = take_until("in")(i)?;
    Ok((rest, IterationType::SingleVal(varname.trim().to_string())))
}

fn parse_key_value(i: &str) -> IResult<&str, IterationType> {
    let (rest, (keyname, .., valname)) = tuple((
        take_till(|c| c == ',' || c == '%'),
        nom::character::complete::char(','),
        multispace0,
        take_until("in"),
    ))(i)?;
    Ok((
        rest,
        IterationType::KeyVal((keyname.trim().to_string(), valname.trim().to_string())),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_parse_quotes() {
        let single = r#"'foo'"#;
        let double = r#""foo""#;
        assert_eq!(parse_quoted(single), Ok(("", "foo")));
        assert_eq!(parse_quoted(double), Ok(("", "foo")));
    }

    #[test]
    fn test_parse_extends() {
        let extends = "{% extends 'parent.html.twig' %}";
        assert_eq!(
            parse_extends(extends),
            Ok(("", "parent.html.twig".to_string()))
        )
    }

    #[test]
    fn test_parse_text() {
        let input = r#"first{# comment #}"#;
        assert_eq!(
            parse_text(input),
            Ok(("{# comment #}", Content::Text("first".to_string())))
        )
    }
}
