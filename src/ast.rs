use tree_sitter::{Tree, Node};
use crate::c_format;

pub fn traverse_ast(ast: Tree, src: String) {
    let root = ast.root_node();
    for child in root.children(&mut root.walk()) {
        match child.grammar_name() {
            "preproc_include" => {
                let line = handle_preproc_include(child, src.clone());
                println!("Includes: {}\n", line);
            },
            "preproc_ifdef" => handle_preproc_ifdef(child, src.clone()),
            "preproc_if" => handle_preproc_if(child, src.clone()),
            "preproc_def" => handle_preproc_def(child, src.clone()),
            "preproc_function_def" => handle_preproc_function_def(child, src.clone()),
            "preproc_call" => handle_preproc_call(child, src.clone()),
            "type_definition" => handle_type_definition(child, src.clone()),
            "identifier" => handle_identifier(child, src.clone()),
            "struct_specifier" => handle_struct_specifier(child, src.clone()),
            "declaration" => {
                let line = handle_declaration(child, src.clone());
                println!("Declaration: {}\n", line);
            },
            ";" => println!("Add a semicolon..."),
            "function_definition" => handle_function_definition(child, src.clone()),
            "expression_statement" => handle_expression_statement(child, src.clone()),
            "compound_statement" => handle_compound_statement(child, src.clone()),
            "if_statement" => handle_if_statement(child, src.clone()),
            "switch_statement" => handle_switch_statement(child, src.clone()),
            "continue_statement" => handle_continue_statement(child, src.clone()),
            "break_statement" => handle_break_statement(child, src.clone()),
            "return_statement" => {
                let return_statement = handle_return_statement(child, src.clone());
                println!("Return: {}\n", return_statement);
            },
            "comment" => {
                let comment = handle_comment(child, src.clone());
                println!("Comment:\n{}", comment);
            },
            "ERROR" => handle_error(child, src.clone()),
            _ => println!("Unknown grammar name 1."),
        }
    }
}

fn handle_compound_statement(root: Node, src: String) {
    for node in root.children(&mut root.walk()) {
        dbg!(&node.grammar_name());
        match node.grammar_name() {
            "preproc_include" => {
                let line = handle_preproc_include(node, src.clone());
                println!("Includes: {}\n", line);
            },
            "preproc_ifdef" => handle_preproc_ifdef(node, src.clone()),
            "preproc_if" => handle_preproc_if(node, src.clone()),
            "preproc_def" => handle_preproc_def(node, src.clone()),
            "preproc_function_def" => handle_preproc_function_def(node, src.clone()),
            "preproc_call" => handle_preproc_call(node, src.clone()),
            "type_definition" => handle_type_definition(node, src.clone()),
            "identifier" => handle_identifier(node, src.clone()),
            "struct_specifier" => handle_struct_specifier(node, src.clone()),
            "declaration" => {
                let line = handle_declaration(node, src.clone());
                println!("Declaration: {}\n", line);
            },
            ";" => println!("Add a semicolon..."),
            "function_definition" => handle_function_definition(node, src.clone()),
            "expression_statement" => handle_expression_statement(node, src.clone()),
            "compound_statement" => handle_compound_statement(node, src.clone()),
            "if_statement" => handle_if_statement(node, src.clone()),
            "switch_statement" => handle_switch_statement(node, src.clone()),
            "case_statement" => handle_case_statement(node, src.clone()),
            "for_statement" => handle_for_statement(node, src.clone()),
            "while_statement" => handle_while_statement(node, src.clone()),
            "do_statement" => handle_do_statement(node, src.clone()),
            "labeled_statement" => handle_labeled_statement(node, src.clone()),
            "continue_statement" => handle_continue_statement(node, src.clone()),
            "break_statement" => handle_break_statement(node, src.clone()),
            "goto_statement" => handle_goto_statement(node, src.clone()),
            "return_statement" => {
                let return_statement = handle_return_statement(node, src.clone());
                println!("Return: {}\n", return_statement);
            },
            "comment" => {
                let comment = handle_comment(node, src.clone());
                println!("Comment:\n{}", comment);
            },
            "{" => println!("Handle open brace..."),
            "}" => println!("Handle closing brace..."),
            "ERROR" => handle_error(node, src.clone()),
            _ => println!("Unknown grammar name 2."),
        }
    }
    println!("");
}

fn handle_preproc_include(root: Node, src: String) -> String {
    let result: String;
    let mut vec = Vec::<String>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            _ => vec.push(node.utf8_text(src.as_bytes()).unwrap_or("").to_string()),
        }
    }
    result = vec.join(" ");
    return result;
}

fn handle_preproc_ifdef(root: Node, src: String) {
    println!("{}", root.grammar_name());
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => handle_compound_statement(node.clone(), src.clone()),
            _ => println!("{}: {}", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    println!("");
}

fn handle_preproc_if(root: Node, src: String) {
    println!("{}", root.grammar_name());
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => handle_compound_statement(node.clone(), src.clone()),
            _ => println!("{}: {}", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    println!("");
}

fn handle_preproc_def(root: Node, src: String) {
    println!("{}", root.grammar_name());
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => handle_compound_statement(node.clone(), src.clone()),
            _ => println!("{}: {}", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    println!("");
}

fn handle_preproc_function_def(root: Node, src: String) {
    println!("{}", root.grammar_name());
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => handle_compound_statement(node.clone(), src.clone()),
            _ => println!("{}: {}", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    println!("");
}

fn handle_preproc_call(root: Node, src: String) {
    println!("{}", root.grammar_name());
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => handle_compound_statement(node.clone(), src.clone()),
            _ => println!("{}: {}", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    println!("");
}

fn handle_type_definition(root: Node, src: String) {
    println!("{}", root.grammar_name());
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => handle_compound_statement(node.clone(), src.clone()),
            _ => println!("{}: {}", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    println!("");
}

fn handle_identifier(root: Node, src: String) {
    println!("{}", root.grammar_name());
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => handle_compound_statement(node.clone(), src.clone()),
            _ => println!("{}: {}", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    println!("");
}

fn handle_struct_specifier(root: Node, src: String) {
    println!("{}", root.grammar_name());
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => handle_compound_statement(node.clone(), src.clone()),
            _ => println!("{}: {}", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    println!("");
}

fn handle_declaration(root: Node, src: String) -> String {
    let mut result;
    let mut vec = Vec::<String>::new();
    let mut remove_space = false;
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => handle_compound_statement(node.clone(), src.clone()),
            _ => {
                let line = node.utf8_text(src.as_bytes()).unwrap_or("");
                if line != ";" {
                    vec.push(line.to_string());
                    if line.starts_with("*") {
                        remove_space = true;
                    }
                }
            },
        }
    }
    result = (vec.join(" ") + ";").to_string();
    if remove_space {
        let idx = result.chars().position(|x| x == ' ').unwrap();
        result.remove(idx);
    }
    return result;
}

fn handle_function_definition(root: Node, src: String) {
    println!("{}", root.grammar_name());
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => handle_compound_statement(node.clone(), src.clone()),
            _ => println!("{}: {}", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    println!("");
}

fn handle_expression_statement(root: Node, src: String) {
    println!("{}", root.grammar_name());
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => handle_compound_statement(node.clone(), src.clone()),
            _ => println!("{}: {}", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    println!("");
}

fn handle_if_statement(root: Node, src: String) {
    println!("{}", root.grammar_name());
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => handle_compound_statement(node.clone(), src.clone()),
            _ => println!("{}: {}", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    println!("");
}

fn handle_switch_statement(root: Node, src: String) {
    println!("{}", root.grammar_name());
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => handle_compound_statement(node.clone(), src.clone()),
            _ => println!("{}: {}", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    println!("");
}

fn handle_case_statement(root: Node, src: String) {
    println!("{}", root.grammar_name());
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => handle_compound_statement(node.clone(), src.clone()),
            _ => println!("{}: {}", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    println!("");
}

fn handle_for_statement(root: Node, src: String) {
    println!("{}", root.grammar_name());
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => handle_compound_statement(node.clone(), src.clone()),
            _ => println!("{}: {}", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    println!("");
}

fn handle_labeled_statement(root: Node, src: String) {
    println!("{}", root.grammar_name());
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => handle_compound_statement(node.clone(), src.clone()),
            _ => println!("{}: {}", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    println!("");
}

fn handle_goto_statement(root: Node, src: String) {
    println!("{}", root.grammar_name());
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => handle_compound_statement(node.clone(), src.clone()),
            _ => println!("{}: {}", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    println!("");
}

fn handle_continue_statement(root: Node, src: String) {
    println!("{}", root.grammar_name());
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => handle_compound_statement(node.clone(), src.clone()),
            _ => println!("{}: {}", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    println!("");
}

fn handle_break_statement(root: Node, src: String) {
    println!("{}", root.grammar_name());
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => handle_compound_statement(node.clone(), src.clone()),
            _ => println!("{}: {}", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    println!("");
}

fn handle_while_statement(root: Node, src: String) {
    println!("{}", root.grammar_name());
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => handle_compound_statement(node.clone(), src.clone()),
            _ => println!("{}: {}", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    println!("");
}

fn handle_do_statement(root: Node, src: String) {
    println!("{}", root.grammar_name());
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => handle_compound_statement(node.clone(), src.clone()),
            _ => println!("{}: {}", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    println!("");
}

fn handle_return_statement(root: Node, src: String) -> String {
    let result: String;
    let mut vec = Vec::<String>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => handle_compound_statement(node.clone(), src.clone()),
            _ => {
                let line = node.utf8_text(src.as_bytes()).unwrap_or("");
                if line != ";" { vec.push(line.to_string()); }
            },
        }
    }
    result = (vec.join(" ") + ";").to_string();
    return result;
}

fn handle_comment(root: Node, src: String) -> String {
    let result = root.utf8_text(src.as_bytes()).unwrap();
    return result.to_string();
}

fn handle_error(root: Node, src: String) {
    println!("{}", root.grammar_name());
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => handle_compound_statement(node.clone(), src.clone()),
            _ => println!("{}: {}", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    println!("");
}
