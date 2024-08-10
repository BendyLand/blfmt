use std::result;

use crate::utils;
use regex::Regex;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FunctionKind {
    Prototype,
    Definition,
    Body,
    Full,
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
    FullBlock
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
pub enum Section {
    Macro,
    Comment,
    Function,
    Na,
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
        x if x.trim_end() == "}"                  => token = Token::EndBlock,
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
        match prev_token {
            Token::Na         => result = Token::Function(FunctionKind::Body),
            Token::Comment(_) => result = Token::Function(FunctionKind::Body),
            _                 => result = prev_token,
        }
    }
    else {
        result = prev_token;
    }
    return result;
}

pub fn group_c_file_sections(token_lines: &Vec<(Token, String)>) -> Vec<(Section, Vec<(Token, String)>)> {
    let mut temp_func = String::new();
    let mut temp_comment = String::new();
    let mut macros: Vec<(Token, String)> = Vec::new();
    let mut functions: Vec<(Token, String)> = Vec::new();
    let mut comments: Vec<(Token, String)> = Vec::new();
    let mut extras: Vec<(Token, String)> = Vec::new();
    for (token, line) in token_lines {
        match token {
            Token::Macro(_) => macros.push((*token, line.clone())),
            Token::Comment(_) => {
                match token {
                    Token::Comment(CommentKind::BlockStart) => temp_comment += (line.to_string() + "\n").as_str(),
                    Token::Comment(CommentKind::BlockMiddle) => temp_comment += (line.to_string() + "\n").as_str(),
                    Token::Comment(CommentKind::BlockEnd) => {
                        temp_comment += line;
                        comments.push((Token::Comment(CommentKind::FullBlock), temp_comment));
                        temp_comment = "".to_string();
                    },
                    Token::Comment(CommentKind::Inline) => comments.push((Token::Comment(CommentKind::Inline), line.to_string())),
                    _ => (),
                };
            }
            Token::Function(_) => {
                match token {
                    Token::Function(FunctionKind::Definition) => {
                        let mut line = line.clone();
                        if line.trim_end().ends_with("{") {
                            line = line.trim_end_matches("{").to_string();
                        }
                        temp_func += (line.to_string() + "\n{\n").as_str();
                    },
                    Token::Function(FunctionKind::Body) => temp_func += (line.to_string() + "\n").as_str(),
                    Token::Function(FunctionKind::Prototype) => {
                        functions.push((Token::Function(FunctionKind::Prototype), line.to_string()));
                    },
                    _ => (),
                }
            }
            Token::EndBlock => {
                if !temp_func.is_empty() {
                    temp_func += line;
                    functions.push((Token::Function(FunctionKind::Full), temp_func));
                    temp_func = "".to_string();
                }
            },
            _ => extras.push((Token::Na, (line.to_string() + "\n"))),
        }
    }
    let mut result: Vec<(Section, Vec<(Token, String)>)> = Vec::with_capacity(4);
    result.push((Section::Macro, macros));
    result.push((Section::Comment, comments));
    result.push((Section::Function, functions));
    result.push((Section::Na, extras));
    return result;
}

pub fn format_macros(macros: Vec<(Token, String)>) -> Vec<(Token, String)> {
    let mut result: Vec<(Token, String)> = Vec::new();
    println!("Formatting macros...");
    for (tok, mac) in macros {
        println!("Token: {:?}, Macro:\n{}\n", tok, mac);
    }
    println!();
    return result;
}

pub fn format_comments(comments: Vec<(Token, String)>) -> Vec<(Token, String)> {
    let mut result: Vec<(Token, String)> = Vec::new();
    println!("Formatting comments...");
    for (token, comment) in comments {
        println!("Token: {:?}, Comment:\n{}\n", token, comment);
    }
    println!();
    return result;
}

pub fn format_functions(functions: Vec<(Token, String)>) -> Vec<(Token, String)> {
    let mut result: Vec<(Token, String)> = Vec::new();
    println!("Formatting functions...");
    for (token, function) in functions {
        println!("Token: {:?}, Function:\n{}\n", token, function);
    }
    println!();
    return result;
}

pub fn format_extras(extras: Vec<(Token, String)>) -> Vec<(Token, String)> {
    let mut result: Vec<(Token, String)> = Vec::new();
    println!("Formatting functions...");
    println!("Extras length: {}", extras.len());
    println!();
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