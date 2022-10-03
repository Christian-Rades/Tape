use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum Module {
    Template(Template),
    Extension(Extension)
}

#[derive(Debug, PartialEq)]
pub struct Template {
    pub name: String,
    pub content: Contents
}

#[derive(Debug, PartialEq)]
pub struct Extension {
    pub name: String,
    pub parent: String,
    pub blocks: HashMap<String, Box<Block>>
}

pub type Contents = Vec<Content>;

#[derive(Debug, PartialEq)]
pub enum Content {
    Text(String),
    Print(Expression),
    Block(Box<Block>),
    Statement(Stmt),
}

#[derive(Debug, PartialEq)]
pub enum Expression {
    Str(String),
    Var(String),
    Parent()
}

#[derive(Debug, PartialEq)]
pub struct Block {
    pub typ: BlockType,
    pub contents: Contents
}

#[derive(Debug, PartialEq)]
pub enum BlockType {
    BlockName(String),
    Loop(Loop),
}

#[derive(Debug, PartialEq)]
pub struct Loop {
    pub typ: IterationType,
    pub iterator: String
}

#[derive(Debug, PartialEq)]
pub enum IterationType {
    SingleVal(String),
    KeyVal((String, String))
}

#[derive(Debug, PartialEq)]
pub enum Stmt {
    Set(Setter),
    Include(String),
}

#[derive(Debug, PartialEq)]
pub struct Setter {
    pub target: String,
    pub value: Expression
}

impl Template {
    pub fn replace_includes(mut self, replace: &mut dyn FnMut(Content)-> Content) -> Template {
        self.content = self.content.into_iter().map(|c| replace_includes(c, replace)).collect();
        self
    }
    
    pub fn into_block(self) -> Content {
        let Self { name, content } = self;
        Content::Block(Box::new(Block { typ: BlockType::BlockName(name), contents: content}))
    }
}

fn replace_includes(content: Content, replace: &mut dyn FnMut(Content) -> Content) -> Content {
    match content {
        Content::Statement(Stmt::Include(_)) => replace(content),
        Content::Block(mut block) => {
            block.contents = block.contents.into_iter().map(|c| replace_includes(c, replace)).collect();
            Content::Block(block)
        }
        _ => content
    }
}
