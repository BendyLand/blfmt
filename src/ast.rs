use tree_sitter::{Tree, Node};
use crate::{c_format, utils::{self, add_leading_whitespace}};

pub fn traverse_ast(ast: Tree, src: String) {
    let root = ast.root_node();
    for child in root.children(&mut root.walk()) {
        match child.grammar_name() {
            "preproc_include" => {
                let line = handle_preproc_include(child, src.clone());
                println!("Finished includes: {}\n", line);
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
                println!("Finished declaration: {}\n", line);
            },
            ";" => println!("Add a semicolon here.\n"),
            "function_definition" => handle_function_definition(child, src.clone()),
            "expression_statement" => handle_expression_statement(child, src.clone()),
            "compound_statement" => handle_compound_statement(child, src.clone()),
            "if_statement" => handle_if_statement(child, src.clone()),
            "switch_statement" => handle_switch_statement(child, src.clone()),
            "continue_statement" => handle_continue_statement(child, src.clone()),
            "break_statemet" => handle_break_statement(child, src.clone()),
            "using_declaration" => handle_using_declaration(child, src.clone()),
            "return_statement" => {
                let return_statement = handle_return_statement(child, src.clone());
                println!("Finished return: {}\n", return_statement);
            },
            "comment" => {
                let comment = handle_comment(child, src.clone());
                println!("Finished comment:\n{}\n", comment);
            },
            "ERROR" => handle_error(child, src.clone()),
            _ => println!("Unknown grammar name 1: {}\n", &child.grammar_name()),
        }
    }
}

fn handle_compound_statement(root: Node, src: String) {
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "preproc_include" => {
                let line = handle_preproc_include(node, src.clone());
                println!("Finished includes: {}\n", line);
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
                println!("Finished declaration: {}\n", line);
            },
            ";" => println!("Add a semicolon here.\n"),
            "function_definition" => handle_function_definition(node, src.clone()),
            "expression_statement" => handle_expression_statement(node, src.clone()),
            "compound_statement" => handle_compound_statement(node, src.clone()),
            "if_statement" => handle_if_statement(node, src.clone()),
            "switch_statement" => handle_switch_statement(node, src.clone()),
            "case_statement" => handle_case_statement(node, src.clone()),
            "for_statement" => handle_for_statement(node, src.clone()),
            "for_range_loop" => handle_for_range_loop(node, src.clone()),
            "while_statement" => handle_while_statement(node, src.clone()),
            "do_statement" => handle_do_statement(node, src.clone()),
            "labeled_statement" => handle_labeled_statement(node, src.clone()),
            "continue_statement" => handle_continue_statement(node, src.clone()),
            "break_statement" => handle_break_statement(node, src.clone()),
            "parameter_list" => handle_parameter_list(node, src.clone()),
            "goto_statement" => handle_goto_statement(node, src.clone()),
            "return_statement" => {
                let return_statement = handle_return_statement(node, src.clone());
                println!("Finished return: {}\n", return_statement);
            },
            "comment" => {
                let comment = handle_comment(node, src.clone());
                println!("Finished comment:\n{}", comment);
            },
            "{" => println!("Add an open brace here.\n"),
            "}" => println!("Add a closing brace here.\n"),
            "ERROR" => handle_error(node, src.clone()),
            _ => println!("Unknown grammar name 2.\n"),
        }
    }
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
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => handle_compound_statement(node.clone(), src.clone()),
            _ => println!("{}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
}

fn handle_parameter_list(root: Node, src: String) {
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => handle_compound_statement(node.clone(), src.clone()),
            _ => println!("{}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
}

fn handle_preproc_if(root: Node, src: String) {
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => handle_compound_statement(node.clone(), src.clone()),
            _ => println!("{}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
}

fn handle_preproc_def(root: Node, src: String) {
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => handle_compound_statement(node.clone(), src.clone()),
            _ => println!("{}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
}

fn handle_preproc_function_def(root: Node, src: String) {
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => handle_compound_statement(node.clone(), src.clone()),
            _ => println!("{}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
}

fn handle_preproc_call(root: Node, src: String) {
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => handle_compound_statement(node.clone(), src.clone()),
            _ => println!("{}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
}

fn handle_type_definition(root: Node, src: String) {
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => handle_compound_statement(node.clone(), src.clone()),
            _ => println!("{}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
}

fn handle_identifier(root: Node, src: String) {
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => handle_compound_statement(node.clone(), src.clone()),
            _ => println!("{}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
}

fn handle_using_declaration(root: Node, src: String) {
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => handle_compound_statement(node.clone(), src.clone()),
            _ => println!("{}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
}

fn handle_struct_specifier(root: Node, src: String) {
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => handle_compound_statement(node.clone(), src.clone()),
            _ => println!("{}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
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
    for node in root.children(&mut root.walk()) {
        // All branches accounted for.
        match node.grammar_name() {
            "compound_statement" => handle_compound_statement(node.clone(), src.clone()),
            "primitive_type" => handle_primitive_type(node.clone(), src.clone()),
            "function_declarator" => handle_function_declarator(node.clone(), src.clone()),
            "pointer_declarator" => handle_pointer_declarator(node.clone(), src.clone()),
            "identifier" => {
                handle_identifier(node.clone(), src.clone());
            },
            _ => { dbg!("You shouldn't be here.\n"); },
        }
    }
}

fn handle_primitive_type(root: Node, src: String) {
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => handle_compound_statement(node.clone(), src.clone()),
            _ => println!("{}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
}

fn handle_function_declarator(root: Node, src: String) {
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => handle_compound_statement(node.clone(), src.clone()),
            _ => println!("{}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
}

fn handle_pointer_declarator(root: Node, src: String) {
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => handle_compound_statement(node.clone(), src.clone()),
            _ => println!("{}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
}

fn handle_expression_statement(root: Node, src: String) {
    for node in root.children(&mut root.walk()) {
        // All branches accounted for.
        match node.grammar_name() {
            "assignment_expression" => handle_assignment_expression(root.clone(), src.clone()), 
            "update_expression" => handle_update_expression(root.clone(), src.clone()),
            "call_expression" => handle_call_expression(root.clone(), src.clone()),
            ";" => println!("Add a semicolon...\n"),
            _ => { dbg!("You shouldn't be here.\n"); },
        }
    }
}

fn handle_assignment_expression(root: Node, src: String) {
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => handle_compound_statement(node.clone(), src.clone()),
            _ => println!("{}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
}

fn handle_update_expression(root: Node, src: String) {
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => handle_compound_statement(node.clone(), src.clone()),
            _ => println!("{}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
}

fn handle_call_expression(root: Node, src: String) {
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => handle_compound_statement(node.clone(), src.clone()),
            _ => println!("{}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
}

fn handle_if_statement(root: Node, src: String) {
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => handle_compound_statement(node.clone(), src.clone()),
            _ => println!("{}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
}

fn handle_switch_statement(root: Node, src: String) {
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => handle_compound_statement(node.clone(), src.clone()),
            _ => println!("{}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
}

fn handle_case_statement(root: Node, src: String) {
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => handle_compound_statement(node.clone(), src.clone()),
            _ => println!("{}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
}

fn handle_for_statement(root: Node, src: String) {
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => handle_compound_statement(node.clone(), src.clone()),
            _ => println!("{}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
}

fn handle_for_range_loop(root: Node, src: String) {
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => handle_compound_statement(node.clone(), src.clone()),
            _ => println!("{}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
}

fn handle_labeled_statement(root: Node, src: String) {
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => handle_compound_statement(node.clone(), src.clone()),
            _ => println!("{}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
}

fn handle_goto_statement(root: Node, src: String) {
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => handle_compound_statement(node.clone(), src.clone()),
            _ => println!("{}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
}

fn handle_continue_statement(root: Node, src: String) {
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => handle_compound_statement(node.clone(), src.clone()),
            _ => println!("{}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
}

fn handle_break_statement(root: Node, src: String) {
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => handle_compound_statement(node.clone(), src.clone()),
            _ => println!("{}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
}

fn handle_while_statement(root: Node, src: String) {
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => handle_compound_statement(node.clone(), src.clone()),
            "while" => handle_while(node.clone(), src.clone()),
            _ => println!("{}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
}

fn handle_while(root: Node, src: String) {
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => handle_compound_statement(node.clone(), src.clone()),
            "while" => {
                println!("Handling while keyword.\n");
            },
            _ => println!("{}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
}

fn handle_do_statement(root: Node, src: String) {
    println!("{}", root.grammar_name());
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => handle_compound_statement(node.clone(), src.clone()),
            _ => println!("{}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
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
    let mut result = root.utf8_text(src.as_bytes()).unwrap().to_string();
    let mut lines: Vec<&str> = result.split("\n").collect();
    if lines.len() > 1 {
        if lines[1].starts_with(" ") && !lines[0].starts_with(" ") {
            let num_whitespace = utils::count_leading_whitespace(lines[1].to_string(), ' ');
            let temp = utils::add_leading_whitespace(lines[0].to_string(), num_whitespace);
            lines[0] = temp.as_str();
            result = lines.join("\n");
        }
    }
    return result;
}

fn handle_error(root: Node, src: String) {
    println!("{}", root.grammar_name());
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => handle_compound_statement(node.clone(), src.clone()),
            _ => println!("{}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
}
