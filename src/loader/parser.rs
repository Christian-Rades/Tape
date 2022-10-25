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
    bytes::complete::{tag, take_till, take_until, take_while, take_while1},
    character::complete::{line_ending, multispace0, multispace1, space0},
    combinator::{eof, opt},
    multi::many_till,
    sequence::{delimited, tuple},
    IResult,
};

use nom_locate::LocatedSpan;
pub type Span<'a> = LocatedSpan<&'a str>;

pub fn parse(name: String, input: &str) -> Result<Module> {
    let input = Span::new(input);
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

fn parse_extends(i: Span) -> IResult<Span, String> {
    let (rest, (.., parent)) = delimited(
        parse_block_tag_l,
        tuple((tag("extends"), multispace1, parse_quoted)),
        parse_block_tag_r,
    )(i)?;
    Ok((rest, parent.to_string()))
}

fn parse_block_tag_l(i: Span) -> IResult<Span, ()> {
    let (rest, _) = tuple((space0, tag("{%"), multispace1))(i)?;
    Ok((rest, ()))
}

fn parse_block_tag_r(i: Span) -> IResult<Span, ()> {
    let (rest, _) = tuple((multispace0, tag("%}"), opt(line_ending)))(i)?;
    Ok((rest, ()))
}

fn parse_quoted(i: Span) -> IResult<Span, Span> {
    alt((
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
    ))(i)
}

fn parse_contents(i: Span) -> IResult<Span, Vec<Content>> {
    let (_, (contents, _)) = many_till(parse_content, eof)(i)?;
    Ok((Span::new(""), contents))
}

fn parse_content(i: Span) -> IResult<Span, Content> {
    alt((parse_print, parse_statement, parse_block, parse_text))(i)
}

fn parse_text(i: Span) -> IResult<Span, Content> {
    let (rest, text) = take_while1(|c| c != '{')(i)?;
    Ok((rest, Content::Text(text.to_string())))
}

fn parse_print(i: Span) -> IResult<Span, Content> {
    let (rest, expr) = delimited(parse_print_tag_l, take_until("}}"), parse_print_tag_r)(i)?;
    let (_, expr) = expression::parse(expr)?;
    Ok((rest, Content::Print(expr)))
}

fn parse_print_tag_l(i: Span) -> IResult<Span, ()> {
    let (rest, _) = tuple((tag("{{"), multispace1))(i)?;
    Ok((rest, ()))
}

fn parse_print_tag_r(i: Span) -> IResult<Span, ()> {
    let (rest, _) = tuple((multispace0, tag("}}")))(i)?;
    Ok((rest, ()))
}

fn parse_statement(i: Span) -> IResult<Span, Content> {
    let (rest, statement) = delimited(
        parse_block_tag_l,
        alt((parse_set_statement, parse_include_statement)),
        parse_block_tag_r,
    )(i)?;
    Ok((rest, Content::Statement(statement)))
}

fn parse_set_statement(i: Span) -> IResult<Span, Stmt> {
    let (rest, (.., target, _, _, expr)) = tuple((
        tag("set"),
        multispace1,
        take_till(|c| c == '='),
        nom::character::complete::char('='),
        multispace0,
        take_until("%}"),
    ))(i)?;
    let (_, expr) = expression::parse(expr)?;
    Ok((
        rest,
        Stmt::Set(Setter {
            target: target.trim().to_string(),
            value: expr,
        }),
    ))
}

fn parse_include_statement(i: Span) -> IResult<Span, Stmt> {
    let (rest, (.., target)) = tuple((tag("include"), multispace1, parse_quoted))(i)?;
    Ok((rest, Stmt::Include(target.to_string())))
}

fn parse_block(i: Span) -> IResult<Span, Content> {
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

fn parse_block_type(i: Span) -> IResult<Span, BlockType> {
    delimited(
        parse_block_tag_l,
        alt((parse_block_name, parse_loop)),
        parse_block_tag_r,
    )(i)
}

fn parse_block_name(i: Span) -> IResult<Span, BlockType> {
    let (rest, (.., name)) = tuple((tag("block"), multispace1, (take_till(|c| c == ' '))))(i)?;
    Ok((rest, BlockType::BlockName(name.to_string())))
}

fn parse_loop(i: Span) -> IResult<Span, BlockType> {
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

fn parse_single_var(i: Span) -> IResult<Span, IterationType> {
    let (rest, varname) = take_until("in")(i)?;
    Ok((rest, IterationType::SingleVal(varname.trim().to_string())))
}

fn parse_key_value(i: Span) -> IResult<Span, IterationType> {
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
        let single = Span::new(r#"'foo'"#);
        let double = Span::new(r#""foo""#);
        assert_eq!(unspan_twice(parse_quoted(single)), ("", "foo"));
        assert_eq!(unspan_twice(parse_quoted(double)), ("", "foo"));
    }

    #[test]
    fn test_parse_extends() {
        let extends = Span::new("{% extends 'parent.html.twig' %}");
        assert_eq!(
            unspan(parse_extends(extends)),
            ("", "parent.html.twig".to_string())
        )
    }

    #[test]
    fn test_parse_text() {
        let input = Span::new(r#"first{# comment #}"#);
        assert_eq!(
            unspan(parse_text(input)),
            ("{# comment #}", Content::Text("first".to_string()))
        )
    }

    fn unspan<O>(span: IResult<Span, O>) -> (&str, O) {
        let (rest, out) = span.unwrap();
        (rest.fragment(), out)
    }

    fn unspan_twice<'a, 'b>(span: IResult<Span<'a>, Span<'b>>) -> (&'a str, &'b str) {
        let (rest, out) = span.unwrap();
        (rest.fragment(), out.fragment())
    }
}
