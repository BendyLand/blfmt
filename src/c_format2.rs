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
    Block,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Token {
    Preprocessor(MacroKind),
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

// First pass will tokenize all preprocessor lines
pub fn tokenize_first_pass(line: String) -> Token {
    let token: Token;
    if line.trim_start().starts_with('#') {
        if line.contains("include") {
            token = Token::Preprocessor(MacroKind::Includes);
        }
        else {
            token = Token::Preprocessor(MacroKind::Other);
        }
    }
    else {
        token = Token::Na;
    }
    return token;
}

// Second pass will identify starts to functions and their corresponding end blocks
pub fn tokenize_second_pass(line: &String) -> Token {
    let re = Regex::new(r"^[\w\*]+\s+[\w\*]+\(.*\)\s*\{?\s*$").unwrap();
    let token: Token;
    if re.is_match(&line) {
        token = Token::Function(FunctionKind::Definition);
    }
    else if line == "}" {
        token = Token::EndBlock;
    }
    else {
        token = Token::Na;
    }
    return token;
}

//? idea: multi pass tokenization. first pass can label function starts and ends, second can do something else, third...
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