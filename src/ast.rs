use tree_sitter::{Tree, Node};
use crate::c_format;

pub fn traverse_ast(ast: Tree, src: String) {
    let root = ast.root_node();
    for child in root.children(&mut root.walk()) {
        match child.grammar_name() {
            "preproc_include" => handle_preproc_include(child, src.clone()),
            "preproc_ifdef" => handle_preproc_ifdef(child, src.clone()),
            "preproc_if" => handle_preproc_if(child, src.clone()),
            "preproc_def" => handle_preproc_def(child, src.clone()),
            "preproc_function_def" => handle_preproc_function_def(child, src.clone()),
            "preproc_call" => handle_preproc_call(child, src.clone()),
            "type_definition" => handle_type_definition(child, src.clone()),
            "struct_specifier" => handle_struct_specifier(child, src.clone()),
            "declaration" => handle_declaration(child, src.clone()),
            ";" => handle_semicolon(child, src.clone()),
            "function_definition" => handle_function_definition(child, src.clone()),
            "expression_statement" => handle_expression_statement(child, src.clone()),
            "compound_statement" => handle_compound_statement(child, src.clone()),
            "if_statement" => handle_if_statement(child, src.clone()),
            "return_statement" => handle_return_statement(child, src.clone()),
            "comment" => handle_comment(child, src.clone()),
            "ERROR" => handle_error(child, src.clone()),
            _ => println!("Unknown grammar name."),
        }
    }
}

fn handle_preproc_include(root: Node, src: String) {
    println!("{}", root.grammar_name());
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => handle_compound_statement(node.clone(), src.clone()),
            _ => println!("{}: {}", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    println!("");
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

fn handle_declaration(root: Node, src: String) {
    println!("{}", root.grammar_name());
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => handle_compound_statement(node.clone(), src.clone()),
            _ => println!("{}: {}", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    println!("");
}

fn handle_semicolon(root: Node, src: String) {
    println!("{}", root.grammar_name());
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => handle_compound_statement(node.clone(), src.clone()),
            _ => println!("{}: {}", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    println!("");
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

fn handle_compound_statement(root: Node, src: String) {
    println!("{}", root.grammar_name());
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
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

fn handle_return_statement(root: Node, src: String) {
    println!("{}", root.grammar_name());
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => handle_compound_statement(node.clone(), src.clone()),
            _ => println!("{}: {}", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    println!("");
}

fn handle_comment(root: Node, src: String) {
    println!("{}", root.grammar_name());
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => handle_compound_statement(node.clone(), src.clone()),
            _ => println!("{}: {}", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    println!("");
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
