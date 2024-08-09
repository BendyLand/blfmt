use crate::utils;
use regex::Regex;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FunctionKind {
    Prototype,
    Definition,
    Body,
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
pub enum CommentKind {
    Inline,
    BlockStart,
    BlockEnd,
    BlockMiddle,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Token {
    Macro(MacroKind),
    Comment(CommentKind),
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

// First pass will tokenize all macro lines.
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
// and single-line comments or block comment starts and ends.
pub fn tokenize_second_pass(line: &String, prev_token: Token) -> Token {
    let re = Regex::new(r"^([\w\*]+\s+)+[\w\*]+\(.*\)\s*\{?\s*$").unwrap();
    let token: Token;
    match line.as_str() {
        x if re.is_match(&x)                  => token = Token::Function(FunctionKind::Definition),
        "}"                                   => token = Token::EndBlock,
        x if x.trim_start().starts_with("/*") => token = Token::Comment(CommentKind::BlockStart),
        x if x.contains("*/")                 => token = Token::Comment(CommentKind::BlockEnd),
        x if x.trim_start().starts_with("//") => token = Token::Comment(CommentKind::Inline),
        _ => token = prev_token,
    }
    return token;
}

// Third pass will identify comment block contents and function prototypes.
pub fn tokenize_third_pass(line: &String, in_func: bool, in_comment: bool, prev_token: Token) -> Token {
    let re = Regex::new(r"^([\w\*]+\s+)+[\w\*]+\(.*\)\s*;\s*$").unwrap();
    let token: Token;
    if in_comment {
        match prev_token {
            Token::Na => token = Token::Comment(CommentKind::BlockMiddle),
            _         => token = prev_token,
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

// Fourth pass will identify function bodies. 
pub fn tokenize_fourth_pass(line: &String, in_func: bool, in_comment: bool, prev_token: Token) -> Token {
    if line.trim() == "{" { return Token::Na; }
    let result: Token;
    if in_func {
        if !in_comment {
            match prev_token {
                Token::Na => result = Token::Function(FunctionKind::Body),
                _         => result = prev_token,
            }
        }
        else {
            match prev_token {
                Token::Comment(_) => result = prev_token,
                _                 => result = Token::Na,
            }
        }
    }
    else {
        result = prev_token;
    }
    return result;
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