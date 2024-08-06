use crate::utils;
use regex::Regex;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum FunctionKind {
    Prototype,
    Definition,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum GlobalKind {
    Variable,
    DataStructure,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum CommentType {
    Inline,
    Block,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Token {
    Preprocessor,
    Comment(CommentType),
    Global(GlobalKind),
    Function(FunctionKind),
    EndBlock,
    Na,
}

pub fn identify_line_token(line: String, prev: &Token) -> (Token, String) {
    return (Token::Na, "".to_string());
}

