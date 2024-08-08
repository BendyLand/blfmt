use crate::utils;
use regex::Regex;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FunctionKind {
    Prototype,
    Definition,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum MacroKind {
    Includes,
    Other,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum GlobalKind {
    Variable,
    DataStructure,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CommentType {
    Inline,
    BlockStart,
    BlockEnd,
    BlockMiddle,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Token {
    Macro(MacroKind),
    Comment(CommentType),
    Global(GlobalKind),
    Function(FunctionKind),
    EndBlock,
    Na,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct LocationState {
    pub prev: Token,
    pub current: Token,
}

// First pass will tokenize all macro lines
pub fn tokenize_first_pass(line: String) -> Token {
    let token: Token;
    if line.trim_start().starts_with('#') {
        if line.contains("include") {
            token = Token::Macro(MacroKind::Includes);
        }
        else {
            token = Token::Macro(MacroKind::Other);
        }
    }
    else {
        token = Token::Na;
    }
    return token;
}

// Second pass will identify function starts and their corresponding end blocks, 
// and single-line comments or block comment starts and ends
pub fn tokenize_second_pass(line: &String, prev_token: Token) -> Token {
    let re = Regex::new(r"^([\w\*]+\s+)+[\w\*]+\(.*\)\s*\{?\s*$").unwrap();
    let token: Token;
    if re.is_match(&line) {
        token = Token::Function(FunctionKind::Definition);
    }
    else if line == "}" {
        token = Token::EndBlock;
    }
    else if line.trim_start().starts_with("/*") {
        token = Token::Comment(CommentType::BlockStart);
    }
    else if line.contains("*/") {
        token = Token::Comment(CommentType::BlockEnd);
    }
    else if line.trim_start().starts_with("//") {
        token = Token::Comment(CommentType::Inline);
    }
    else {
        token = prev_token;
    }
    return token;
}

pub fn tokenize_third_pass(line: &String, in_func: bool, in_comment: bool, prev_token: Token) -> Token {
    let re = Regex::new(r"^([\w\*]+\s+)+[\w\*]+\(.*\)\s*;\s*$").unwrap();
    let token: Token;
    if in_comment {
        if prev_token == Token::Na {
            token = Token::Comment(CommentType::BlockMiddle);
        }
        else {
            token = prev_token;
        }
    }
    else if re.is_match(line) {
        if !in_func {
            token = Token::Function(FunctionKind::Prototype);
        }
        else {
            token = Token::Na;
        }
    }
    else {
        token = prev_token;
    }
    return token;
}

/*  
States:
preprocessor - starts with '#'
comment - line contains // or /+* and comment delim isn't in a string.
global - 
    var - not in function, line contains '=' and ';', and (opt?) line starts with data type.
    data - ", and line starts with data structure name or "typedef".
function -
    def - not already in function, line has '(' and ')', line doesn't have ';', and (opt?) line starts with data type.
    prot - ", ", line ends with ';', ".
end_block - line only contains '}'.
na - not any of the others
*/