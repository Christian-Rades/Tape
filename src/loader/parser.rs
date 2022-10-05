use super::ast::{Template, Module, Extension, Content, Expression, BlockType, Block, Stmt, IterationType, Loop, Setter, get_blocks, KeyValuePair};

use std::collections::HashMap;

use anyhow::{anyhow, Result};
use nom::{IResult, sequence::{tuple, delimited, separated_pair}, character::complete::{multispace0, multispace1, line_ending, digit1, space0, one_of}, branch::alt, bytes::{complete::{take_while, tag, take_till, take_while1}, streaming::take_until}, multi::{many_till, separated_list0}, combinator::{eof, opt, map_res, not}, number::complete::float};

pub fn parse(name: String, input: &str) -> Result<Module> {
    if let Ok((rest, parent)) = parse_extends(input) {
        match parse_contents(rest) {
            Ok((_, content)) => {
                let ext = Extension {name, parent, blocks: get_blocks(content, HashMap::default())};
                Ok(Module::Extension(ext))
            }, 
            Err(err) => Err(anyhow!("error parsing {}: {}", name, err))
        }
    } else {
        match parse_contents(input) {
            Ok((_, content)) => Ok(Module::Template(Template {name, content})),
            Err(err) => Err(anyhow!("error parsing {}: {}", name, err))
        }
    }
}

fn parse_extends(i: &str) -> IResult<&str, String> {
    let (rest, (.., parent)) = delimited(parse_block_tag_l, tuple((tag("extends"), multispace1, parse_quoted)), parse_block_tag_r)(i)?;
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
            delimited(nom::character::complete::char('\''), take_while(|c| c != '\''), nom::character::complete::char('\'')),
            delimited(nom::character::complete::char('"'), take_while(|c| c != '"'), nom::character::complete::char('"')),
            ))(i)?;
    Ok(result)
}

fn parse_contents(i: &str) -> IResult<&str, Vec<Content>> {
    let (_, (contents, _)) = many_till(parse_content, eof)(i)?;
    Ok(("", contents))
}

fn parse_content(i: &str) -> IResult<&str, Content> {
    alt((
            parse_print,
            parse_statement,
            parse_block,
            parse_text,
        ))(i)
}

fn parse_text(i: &str) -> IResult<&str, Content> {
    let (rest, text) = take_while1(|c| c != '{')(i)?;
    Ok((rest, Content::Text(text.to_string())))
}

fn parse_print(i: &str) -> IResult<&str, Content> {
    let (rest, expr) = delimited(parse_print_tag_l, parse_expr, parse_print_tag_r)(i)?;
    Ok((rest, Content::Print(expr)))
}

fn parse_print_tag_l(i: &str) -> IResult<&str, ()> {
    let (rest, _) = tuple((tag("{{"), multispace1))(i)?;
    Ok((rest, ()))
}


fn parse_print_tag_r(i: &str) -> IResult<&str, ()> {
    let (rest, _) = tuple((multispace0, tag("}}")))(i)?;
    Ok((rest, ()))
}


fn parse_expr(i: &str) -> IResult<&str, Expression> {
    if let Ok((rest, _)) = tag::<&str, &str, nom::error::Error<&str>>("parent()")(i) {
        return Ok((rest, Expression::Parent()))
    }
    let (rest, (_, expr)) = tuple((multispace0, alt((parse_hash_map, parse_parens, parse_array, parse_number, parse_float, parse_string_literal, parse_var))))(i)?;
    Ok((rest, expr))
}

fn parse_string_literal(i: &str) -> IResult<&str, Expression> {
    let (rest, plain_str) = parse_quoted(i)?;
    Ok((rest, Expression::Str(plain_str.to_string())))
}

fn parse_var(i: &str) -> IResult<&str, Expression> {
    let is_identifier = |c| -> bool {
        ('a'..='z').contains(&c) || ('A'..='Z').contains(&c) || c == '_' || (0x7f as char <= c && c <= 0xff as char)
    };
    let (rest, (part1, part2)) = tuple((take_while1(is_identifier), take_while(|c| is_identifier(c) || ('0'..='9').contains(&c) || c == '.')))(i)?;
    let mut accessor = part1.to_string();
    accessor.push_str(part2.trim());
    Ok((rest, Expression::Var(accessor)))
}

fn parse_float(i: &str) -> IResult<&str, Expression> {
    let (rest, f) = float(i)?;
    Ok((rest, Expression::Float(f)))
}

fn parse_number(i: &str) -> IResult<&str, Expression> {
    //TODO add negative numbers
    let (rest, number) = map_res(tuple((digit1, not(one_of("e.")))), |(number,..)| str::parse(number))(i)?;
    Ok((rest, Expression::Number(number)))
}

fn parse_parens(i: &str) -> IResult<&str, Expression> {
    let (rest, (.., (child_exprs, ..))) = tuple((nom::character::complete::char('('), many_till(parse_expr, tuple((multispace0, nom::character::complete::char(')'))))))(i)?;
    Ok((rest, Expression::Parens(child_exprs)))
}

fn parse_array(i: &str) -> IResult<&str, Expression> {
    let (rest, elems) = delimited(tag("["), separated_list0(tag(","), parse_expr), tuple((multispace0,tag("]"))))(i)?;
    Ok((rest, Expression::Array(elems)))
}

fn parse_hash_map(i: &str) -> IResult<&str, Expression> {
    let (rest, kv_pairs) = delimited(tuple((tag("{"), multispace0)), separated_list0(tuple((multispace0,tag(","), multispace0)), parse_key_value_pair), tuple((multispace0, tag("}"))))(i)?;
    Ok((rest, Expression::HashMap(kv_pairs)))
}

fn parse_key_value_pair(i: &str) -> IResult<&str, KeyValuePair> {
    let (rest, (key, value)) = separated_pair(alt((parse_parens, parse_string_literal, parse_var)), tuple((multispace0,tag(":"),multispace0)), parse_expr)(i)?;
    //hash keys are allowed to be unqouted
    let key = match key {
        Expression::Var(v) => Expression::Str(v),
        _ => key
    };
    Ok((rest, KeyValuePair{key, value}))
}

fn parse_statement(i: &str) -> IResult<&str, Content> {
    let (rest, statement) = delimited(parse_block_tag_l, alt((parse_set_statement, parse_include_statement)),parse_block_tag_r)(i)?;
    Ok((rest, Content::Statement(statement)))
}

fn parse_set_statement(i: &str) -> IResult<&str, Stmt> {
    let (rest, (.., target, _, _, expr)) = tuple((tag("set"), multispace1, take_till(|c| c == '='), nom::character::complete::char('='), multispace0, parse_expr))(i)?;
    Ok((rest, Stmt::Set(Setter{target: target.trim().to_string(), value: expr})))
}

fn parse_include_statement(i: &str) -> IResult<&str, Stmt> {
    let (rest, (..,target)) = tuple((tag("include"), multispace1, parse_quoted))(i)?;
    Ok((rest, Stmt::Include(target.to_string())))
}

fn parse_block(i: &str) -> IResult<&str, Content> {
   let (rest, typ) = parse_block_type(i)?;
   match typ {
       BlockType::BlockName(_) => {
           let (rest, (contents, _)) = many_till(parse_content, tuple((tag("{% endblock %}"), opt(line_ending))))(rest)?;
           Ok((rest, Content::Block(Box::new(Block{typ, contents}))))
       }
       BlockType::Loop(_) => {
           let (rest, (contents, _)) = many_till(parse_content, tuple((tag("{% endfor %}"), opt(line_ending))))(rest)?;
           Ok((rest, Content::Block(Box::new(Block{typ, contents}))))
       }
   }
}

fn parse_block_type(i: &str) -> IResult<&str, BlockType> {
    delimited(parse_block_tag_l, alt((parse_block_name, parse_loop)),parse_block_tag_r)(i)
}

fn parse_block_name(i: &str) -> IResult<&str, BlockType> {
    let (rest, (.., name)) = tuple((tag("block"), multispace1, (take_till(|c| c == ' '))))(i)?;
    Ok((rest, BlockType::BlockName(name.to_string())))
}

fn parse_loop(i: &str) -> IResult<&str, BlockType> {
    let (rest, (.., iter_type, _, _, iterator)) = tuple((tag("for"), multispace1, alt((parse_key_value, parse_single_var)), tag("in"),multispace1, take_till(|c| c == ' ')))(i)?;
    Ok((rest, BlockType::Loop(Loop{typ: iter_type, iterator: iterator.to_string()})))
}

fn parse_single_var(i: &str) -> IResult<&str, IterationType> {
    let (rest, varname) = take_until("in")(i)?;
    Ok((rest, IterationType::SingleVal(varname.trim().to_string())))
}

fn parse_key_value(i: &str) -> IResult<&str, IterationType> {
    let (rest, (keyname, .., valname)) = tuple((take_till(|c| c==',' || c == '%'), nom::character::complete::char(','), multispace0, take_until("in")))(i)?;
    Ok((rest, IterationType::KeyVal((keyname.trim().to_string(), valname.trim().to_string()))))
}

#[cfg(test)]
mod tests {
    use crate::loader::ast::Setter;

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
       assert_eq!(parse_extends(extends), Ok(("", "parent.html.twig".to_string())))
   } 

   #[test]
   fn test_parse_text() {
       let input = r#"first{# comment #}"#;
       assert_eq!(parse_text(input), Ok(("{# comment #}", Content::Text("first".to_string()))))
   }

   #[test]
   fn test_parse_print_block() {
       let parent = r#"{{ parent() }}"#;
       assert_eq!(parse_print(parent), Ok(("", Content::Print(Expression::Parent()))));
       let plain_str = r#"{{ 'foo' }}"#;
       assert_eq!(parse_print(plain_str), Ok(("", Content::Print(Expression::Str("foo".to_string())))));
       let var_acces = r#"{{ foo.baz_foo }}"#;
       assert_eq!(parse_print(var_acces), Ok(("", Content::Print(Expression::Var("foo.baz_foo".to_string())))));
   }

   #[test]
   fn test_parse_set() {
       let set = r#"set var = 'bar'"#;
       assert_eq!(parse_set_statement(set),
       Ok(("", 
           Stmt::Set(
               Setter{
                   target: "var".to_string(), 
                   value: Expression::Str("bar".to_string())
               })))
       );

       let set_var = r#"set var = bar"#;
       assert_eq!(parse_set_statement(set_var),
       Ok(("", 
           Stmt::Set(
               Setter{
                   target: "var".to_string(), 
                   value: Expression::Var("bar".to_string())
               })))
       )
   }

   #[test]
   fn test_parse_general() {
       let test_tpl = r#"{% block base_doctype %}
<!DOCTYPE html>
{% endblock %}

{% for test in coll %}
pre
{{ test }}
post
{% endfor %}
{% set v = { 'bar': [1,1.5], (v1): foo } %}

include:
{% include 'foo.html.twig' %}

<h1>HELLO {{ foo.name }}</h1>
"#;
    let module = parse("foo".to_string(), test_tpl).expect("parsing didn't work");
    assert_eq!(module, Module::Template(Template{
        name: "foo".to_string(),
        content: vec![
            Content::Block(Box::new(Block{
                typ: BlockType::BlockName("base_doctype".to_string()),
                contents: vec![
                    Content::Text("<!DOCTYPE html>\n".to_string())
                ]
            })),
            Content::Text("\n".to_string()),
            Content::Block(Box::new(Block {
                typ: BlockType::Loop(Loop {
                    typ: IterationType::SingleVal("test".to_string()),
                    iterator: "coll".to_string()
                }),
                contents: vec![
                    Content::Text("pre\n".to_string()),
                    Content::Print(Expression::Var("test".to_string())),
                    Content::Text("\npost\n".to_string())
                ]
            })),
            Content::Statement(Stmt::Set(Setter {
                target: "v".to_string(),
                value: Expression::HashMap(vec![
                                           KeyValuePair {
                                               key: Expression::Str("bar".to_string()), 
                                               value: Expression::Array(vec![
                                                                        Expression::Number(1),
                                                                        Expression::Float(1.5)
                                               ])},
                                            KeyValuePair {
                                                key: Expression::Parens(vec![Expression::Var("v1".to_string())]),
                                                value: Expression::Var("foo".to_string())
                                            }
                ])
            })),
            Content::Text("\ninclude:\n".to_string()),
            Content::Statement(Stmt::Include("foo.html.twig".to_string())),
            Content::Text("\n<h1>HELLO ".to_string()),
            Content::Print(Expression::Var("foo.name".to_string())),
            Content::Text("</h1>\n".to_string())
        ]
    }))
   }
}
