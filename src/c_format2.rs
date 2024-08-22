use std::result;

use crate::utils;
use regex::Regex;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord)]
pub enum FunctionKind {
    Prototype,
    Definition,
    Body,
    Full,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord)]
pub enum MacroKind {
    Includes,
    Other,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord)]
pub enum GlobalKind {
    Variable,
    DataStructure,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord)]
pub enum CommentKind {
    Inline,
    BlockStart,
    BlockEnd,
    BlockMiddle,
    FullBlock
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord)]
pub enum Token {
    Macro(MacroKind),
    Comment(CommentKind),
    Global(GlobalKind),
    Function(FunctionKind),
    EndBlock,
    Na,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord)]
pub enum Section {
    Macro,
    Comment,
    Function,
    Na,
}

#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord)]
pub enum LineToken {
    Var(String, String, String),
    Const(String, String, String),
    If(String, String),
    ElseIf(String, String),
    Else,
    Switch(String, String),
    For(String, String),
    While(String, String),
    Comment(String),
    FunctionCall(String),
    Return(String),
    Na(String),
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
    let mut lang_includes = Vec::<(Token, String)>::new();
    let mut custom_includes = Vec::<(Token, String)>::new();
    let mut others = Vec::<(Token, String)>::new();
    for (tok, mac) in macros {
        let line = mac.clone();
        let token_clone = tok.clone();
        match tok {
            Token::Macro(MacroKind::Includes) => {
                if mac.contains("\"") {
                    custom_includes.push((token_clone, line));
                }
                else {
                    lang_includes.push((token_clone, line));
                }
            },
            Token::Macro(MacroKind::Other) => others.push((token_clone, line)),
            _ => (),
        };
    }
    lang_includes.sort();
    custom_includes.sort();
    others.sort();
    result.extend(lang_includes);
    result.extend(custom_includes);
    result.extend(others);
    return result;
}

pub fn format_functions(functions: Vec<(Token, String)>) -> Vec<(Token, String)> {
    let mut result: Vec<(Token, String)> = Vec::new();
    let mut prototypes: Vec<(Token, String)> = functions.clone().into_iter().filter(|x| x.0 == Token::Function(FunctionKind::Prototype)).collect();
    prototypes.sort();
    result.extend(prototypes);
    for (token, function) in functions {
        match token {
            Token::Function(FunctionKind::Prototype) => continue,
            _ => {
                let f_func = format_c_function(function);
            },
        };
        println!();
    }
    return result;
}

fn format_c_function(func: String) -> String {
    let mut result = String::new();
    let lines: Vec<&str> = func.split("\n").collect();
    let mut f_lines = Vec::<String>::new();
    for (i, line) in lines.into_iter().enumerate() {
        match i {
            0 => {
                if utils::line_ends_with_curly_brace(&line.to_string()) {
                    let temp = line.trim_end_matches("{");
                    f_lines.push(temp.to_string());
                    f_lines.push("{".to_string());
                }
            },
            1 => {
                if utils::line_ends_with_curly_brace(&line.to_string()) {
                    continue;
                }
                else {
                    let line_token = tokenize_c_fn_line(&line.to_string());
                    println!("{:?}", line_token);
                }
            },
            _ => {
                let line_token = tokenize_c_fn_line(&line.to_string());
                println!("{:?}", line_token);
            },
        };
    }
    return result;
}

fn tokenize_c_fn_line(line: &String) -> LineToken {
    let mut result = LineToken::Na("".to_string());
    let pat = Regex::new(r"\((.*)\)").unwrap();
    match line.clone() {
        x if x.contains("else if") => {
            let condition = pat.captures(&line).unwrap();
            let end_token = extract_end_token(&line);
            result = LineToken::ElseIf(condition.get(1).unwrap().as_str().to_string(), end_token);
        },
        x if x.trim_start().starts_with("if") => {
            let condition = pat.captures(&line).unwrap();
            let end_token = extract_end_token(&line);
            result = LineToken::If(condition.get(1).unwrap().as_str().to_string(), end_token);
        },
        x if x.contains("else") => {
            result = LineToken::Else;
        },
        x if x.trim_start().starts_with("for") => {
            let condition = pat.captures(&line).unwrap();
            let end_token = extract_end_token(&line);
            result = LineToken::For(condition.get(1).unwrap().as_str().to_string(), end_token);
        },
        x if x.trim_start().starts_with("while") => {
            let condition = pat.captures(&line).unwrap();
            let end_token = extract_end_token(&line);
            result = LineToken::While(condition.get(1).unwrap().as_str().to_string(), end_token);
        },
        x if x.trim_start().starts_with("switch") => {
            let condition = pat.captures(&line).unwrap();
            let end_token = extract_end_token(&line);
            result = LineToken::Switch(condition.get(1).unwrap().as_str().to_string(), end_token);
        },
        x if x.trim_start().starts_with("const") => {
            let line_clone = line.clone().trim_start().to_string();
            let words: Vec::<&str> = line_clone.split(" ").collect();
            if words.len() >= 3 {
                let const_type = words[1].to_string();
                let name = words[2].to_string();
                let start_idx = line.find("=").unwrap();
                let val = line[start_idx+1..].trim_start().to_string();
                result = LineToken::Const(const_type, name, val);
            }
            else {
                dbg!("Something weird happened during parsing and I shouldn't be here!");
            }
        },
        x if x.trim_start().starts_with("return") => {
            let line_clone = line.clone().trim_start().to_string();
            let start_idx = line_clone.find(" ").unwrap();
            let value = line_clone[start_idx+1..].to_string();
            result = LineToken::Return(value);
        },
        x if x.split(" ").collect::<Vec<&str>>().len() >= 2 && 
        x.chars().filter(|x| *x == '=').count() > 0         && 
        x.trim_end().ends_with(";") => {
            let line_clone = line.clone().trim_start().to_string();
            let words: Vec::<&str> = line_clone.split(" ").collect();
            if words.len() >= 2 {
                let var_type; 
                let name; 
                let start_idx; 
                let val; 
                if words.iter().position(|x| *x == "=").unwrap_or_default() == 1 {
                    var_type = "".to_string();
                    name = words[0].to_string();
                }
                else {
                    var_type = words[0].to_string();
                    name = words[1].to_string();
                }
                start_idx = line.find("=").unwrap();
                val = line[start_idx+1..].trim_start().to_string();
                result = LineToken::Var(var_type, name, val);
            }
            else {
                dbg!("Something weird happened during parsing and I shouldn't be here!");
            }
        }
        _ => {
            result = LineToken::Na(line.clone());
        },
    }
    return result;
}

fn extract_end_token(line: &String) -> String {
    let end_token: String;
    if line.trim_end().ends_with("{") {
        end_token = "{".to_string();
    }
    else if line.trim_end().ends_with(")") {
        end_token = "".to_string();
    }
    else {
        let idx = line.rfind(")").unwrap();
        end_token = line[idx+1..].trim_start().to_string();
    }
    return end_token;
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