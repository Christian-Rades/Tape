pub mod ast;
mod lexer;
mod parser;

pub use parser::{parse, Operator};
