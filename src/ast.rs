use tree_sitter::{Tree, Node};
use crate::{c_format, utils::{self, add_all_leading_tabs, add_leading_whitespace, remove_all_spaces, remove_empty_lines, remove_unnecessary_spaces, remove_whitespace_before_commas, StringUtils}};

pub fn traverse_ast(ast: Tree, src: String) {
    let root = ast.root_node();
    for child in root.children(&mut root.walk()) {
        match child.grammar_name() {
            "preproc_include" => {
                let line = handle_preproc_include(child, src.clone());
                println!("Finished includes:\n{}\n", line);
            },
            "preproc_ifdef" => handle_preproc_ifdef(child, src.clone()),
            "preproc_if" => handle_preproc_if(child, src.clone()),
            "preproc_def" => handle_preproc_def(child, src.clone()),
            "preproc_function_def" => handle_preproc_function_def(child, src.clone()),
            "preproc_call" => handle_preproc_call(child, src.clone()),
            "type_definition" => handle_type_definition(child, src.clone()),
            "struct_specifier" => handle_struct_specifier(child, src.clone()),
            "declaration" => { 
                let line = handle_declaration(child, src.clone());
                println!("Finished declaration:\n{}\n", line);
            },
            ";" => (),
            "function_definition" => handle_function_definition(child, src.clone()), 
            "expression_statement" => {
                let expression_statement = handle_expression_statement(child, src.clone()); 
                println!("Finished expression statement:\n{}\n", expression_statement);
            },
            "compound_statement" => {
                let compound_statement = handle_compound_statement(child, src.clone());
                println!("Finished compound statement:\n{}\n", compound_statement);
            },
            "if_statement" => {
                let if_statement = handle_if_statement(child, src.clone());
                println!("Finished if statement:\n{}\n", if_statement);
            },
            "switch_statement" => {
                let switch_statement = handle_switch_statement(child, src.clone());
                println!("Finished switch statement:\n{}\n", switch_statement);
            },
            "continue_statement" => handle_continue_statement(child, src.clone()),
            "break_statement" => {
                let break_statement = "break;";
            },
            "using_declaration" => handle_using_declaration(child, src.clone()),
            "sized_type_specifier" => handle_sized_type_specifier(child, src.clone()),
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

fn handle_compound_statement(root: Node, src: String) -> String {
    let mut result = "".to_string();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "preproc_include" => {
                result = handle_preproc_include(node, src.clone());
            },
            "preproc_ifdef" => handle_preproc_ifdef(node, src.clone()),
            "preproc_if" => handle_preproc_if(node, src.clone()),
            "preproc_def" => handle_preproc_def(node, src.clone()),
            "preproc_function_def" => handle_preproc_function_def(node, src.clone()),
            "preproc_call" => handle_preproc_call(node, src.clone()),
            "type_definition" => handle_type_definition(node, src.clone()),
            "struct_specifier" => handle_struct_specifier(node, src.clone()),
            "declaration" => {
                result = handle_declaration(node, src.clone());
            },
            ";" => (),
            "function_definition" => handle_function_definition(node, src.clone()),
            "expression_statement" => {
                result = handle_expression_statement(node, src.clone());
            },
            "compound_statement" => {
                result = handle_inner_compound_statement(node, src.clone());
            },
            "if_statement" => {
                result = handle_if_statement(node, src.clone());
            },
            "switch_statement" => {
                result = handle_switch_statement(node, src.clone());
            },
            "case_statement" => {
                result = handle_case_statement(node, src.clone());
            },
            "for_statement" => handle_for_statement(node, src.clone()),
            "for_range_loop" => handle_for_range_loop(node, src.clone()),
            "while_statement" => handle_while_statement(node, src.clone()),
            "do_statement" => handle_do_statement(node, src.clone()),
            "labeled_statement" => handle_labeled_statement(node, src.clone()),
            "continue_statement" => handle_continue_statement(node, src.clone()),
            "break_statement" => {
                result = "break;".to_string();
            },
            "parameter_list" => handle_parameter_list(node, src.clone()),
            "goto_statement" => {
                result = handle_goto_statement(node, src.clone());
            },
            "init_declarator" => handle_init_declarator(node, src.clone()),
            "primitive_type" => handle_primitive_type(node, src.clone()),
            "while" => handle_while(node, src.clone()),
            "parenthesized_expression" => {
                result = handle_parenthesized_expression(node, src.clone());
            },
            "identifier" => {
                result = handle_identifier(node, src.clone());
            },
            "return" => (),
            "true" => (),
            "false" => (),
            "return_statement" => {
                result = handle_return_statement(node, src.clone());
            },
            "comment" => {
                result = handle_comment(node, src.clone());
            },
            "{" => (),
            "}" => (),
            "ERROR" => handle_error(node, src.clone()),
            _ => println!("Unknown grammar name 2: {}\n", node.grammar_name()),
        }
    }
    return result;
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

fn handle_declaration(root: Node, src: String) -> String {
    let mut result;
    let mut vec = Vec::<String>::new();
    let mut remove_ptr_space = false;
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => {
                let compound_statement = handle_compound_statement(node.clone(), src.clone());
            },
            _ => {
                let line = node.utf8_text(src.as_bytes()).unwrap_or("");
                if line != ";" {
                    vec.push(line.to_string());
                    if line.starts_with("*") && line.trim_start_matches("*").starts_with(" ") {
                        remove_ptr_space = true;
                    }
                }
            },
        }
    }
    result = (vec.join(" ") + ";").to_string();
    if remove_ptr_space {
        let idx = result.chars().position(|x| x == '*').unwrap();
        result.remove(idx-1);
    }
    if result.contains(",") { result = remove_whitespace_before_commas(result); }
    return result;
}

fn handle_function_definition(root: Node, src: String) {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        // All branches accounted for.
        match node.grammar_name() {
            "compound_statement" => {
                let compound_statement = handle_compound_statement(node.clone(), src.clone());
                result += (compound_statement + "\n").as_str();
            },
            "primitive_type" => handle_primitive_type(node.clone(), src.clone()),
            "function_declarator" => handle_function_declarator(node.clone(), src.clone()),
            "pointer_declarator" => handle_pointer_declarator(node.clone(), src.clone()),
            "identifier" => {
                let identifier = handle_identifier(node.clone(), src.clone());
                result += (identifier + "\n").as_str();
            },
            "storage_class_specifier" => handle_storage_class_specifier(node.clone(), src.clone()),
            "parenthesized_declarator" => handle_parenthesized_declarator(node.clone(), src.clone()),
            "struct_specifier" => handle_struct_specifier(node.clone(), src.clone()),
            "sized_type_specifier" => handle_sized_type_specifier(node.clone(), src.clone()),
            "type_qualifier" => handle_type_qualifier(node.clone(), src.clone()),
            "ERROR" => handle_error(node.clone(), src.clone()),
            _ => println!("You shouldn't be here 1: {}\n", node.grammar_name()),
        }
    }
    // println!("Finished function definition:\n{}\n", result);
}

fn handle_expression_statement(root: Node, src: String) -> String {
    let mut result = "".to_string();
    for node in root.children(&mut root.walk()) {
        // All branches accounted for.
        match node.grammar_name() {
            "assignment_expression" => {
                result = handle_assignment_expression(root.clone(), src.clone());
            },
            "update_expression" => {
                result = handle_update_expression(root.clone(), src.clone());
            },
            "call_expression" => {
                result = handle_call_expression(root.clone(), src.clone());
            },
            "binary_expression" => handle_binary_expression(root.clone(), src.clone()),
            "ERROR" => handle_error(root.clone(), src.clone()),
            "identifier" => {
                result = handle_identifier(root.clone(), src.clone());
            },
            ";" => (), // Handled in the functions called above.
            _ => println!("You shouldn't be here 2: {}\n", node.grammar_name()),
        }
    }
    return result;
}

fn handle_assignment_expression(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            ";" => result += ";",
            _ => {
                let parts: Vec<&str> = {
                    node
                        .utf8_text(src.as_bytes())
                        .unwrap_or("")
                        .split(" ")
                        .filter(|x| !x.is_empty())
                        .collect()
                };
                result += parts.join(" ").as_str();
            },
        }
    }
    return result;
}

fn handle_inner_compound_statement(root: Node, src: String) -> String {
    let mut result = "".to_string();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "preproc_call" => {
                //? doesn't do anything yet
                let preproc_call = handle_preproc_call(node, src.clone());
            },
            "compound_statement" => {
                let compound_statement = handle_compound_statement(node, src.clone());
                println!("COMPOUND:\n{}\n", compound_statement);
                // result += "\t";
                // result += (compound_statement + "\n").as_str();
            },
            "expression_statement" => {
                let expression_statement = handle_expression_statement(node, src.clone());
                result += "\t";
                result += (expression_statement + "\n").as_str();
            },
            "return_statement" => {
                let return_statement = handle_return_statement(node, src.clone());
                result += "\t";
                result += (return_statement + "\n").as_str();
            },
            "goto_statement" => {
                let goto_statement = handle_goto_statement(node, src.clone());
                result += "\t";
                result += (goto_statement + "\n").as_str();
            },
            "if_statement" => {
                let if_statement = handle_if_statement(node, src.clone());
                result += "\t";
                result += (if_statement + "\n\t").as_str();
            },
            "switch_statement" => {
                let switch_statement = handle_switch_statement(node, src.clone());
                result += "\t";
                result += (switch_statement + "\n").as_str();
            },
            "for_statement" => handle_for_statement(node, src.clone()),
            "while_statement" => handle_while_statement(node, src.clone()),
            "do_statement" => handle_do_statement(node, src.clone()),
            "break_statement" => {
                result += "\tbreak;";
            },
            "continue_statement" => {
                result += "\tcontinue;";
            },
            "case_statement" => {
                let mut case_statement = handle_case_statement(node, src.clone());
                case_statement = add_all_leading_tabs(case_statement);
                case_statement = remove_empty_lines(case_statement.split("\n").collect::<Vec<&str>>());
                result += (case_statement + "\n").as_str();
            },
            "declaration" => {
                let declaration = handle_declaration(node, src.clone());
                result += "\t";
                result += (declaration + "\n").as_str();
            },
            "comment" => {
                let comment = handle_comment(node, src.clone());
                result += "\t";
                result += (comment + "\n").as_str();
            },
            "{" => result += "{\n",
            "}" => result += "}",
            _ => println!("You should't be here 4: {}", node.grammar_name()),
        }
    }
    return result;
}

fn handle_update_expression(root: Node, src: String) -> String {
    let mut result = {
        root
            .clone()
            .utf8_text(src.as_bytes())
            .unwrap_or("UNABLE TO UNWRAP update_expression")
            .to_string()
    };
    result = remove_all_spaces(result);
    return result.to_string();
}

fn handle_call_expression(root: Node, src: String) -> String {
    let mut result = "".to_string();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "call_expression" => {
                for subnode in node.children(&mut node.walk()) {
                    match subnode.grammar_name() {
                        "identifier" => {
                            let identifier = handle_identifier(subnode, src.clone());
                            println!("Finished identifier: {}\n", identifier);
                        },
                        "argument_list" => {
                            let argument_list = handle_argument_list(subnode, src.clone());
                            println!("Finished argument list: {}\n", argument_list);
                        },
                        _ => println!("You shouldn't be here 8: {}\n", subnode.grammar_name()),
                    }
                }
                result += node.utf8_text(src.as_bytes()).unwrap_or("");
            },
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                dbg!(identifier);
            },
            "argument_list" => {
                let argument_list = handle_argument_list(node, src.clone());
            },
            ";" => result += ";",
            _ => println!("You shouldn't be here 3: {}", node.grammar_name()),
        }
    }
    return result;
}

fn handle_if_statement(root: Node, src: String) -> String {
    let mut parts = Vec::<String>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "if" => parts.push("if".to_string()),
            "parenthesized_expression" => {
                let mut temp = String::new();
                for (i, subnode) in node.children(&mut node.walk()).enumerate() {
                    if subnode.child_count() == 0 {
                        let text = subnode.utf8_text(src.as_bytes()).unwrap_or("UNABLE TO UNWRAP parenthesized_expression IN if_statement 1");
                        temp += text;
                    }
                    else {
                        //* macros have an extra space `EXAMPLE (arg1, arg2)`
                        // dbg!(&subnode.grammar_name();
                        let mut expr_parts = Vec::<String>::new();
                        for inner_subnode in subnode.children(&mut subnode.walk()) {
                            match inner_subnode.grammar_name() {
                                "call_expression" => {
                                    let call_expression = handle_call_expression(inner_subnode, src.clone());
                                    dbg!(call_expression);
                                },
                                _ => {
                                    expr_parts.push(inner_subnode.utf8_text(src.as_bytes()).unwrap_or("UNABLE TO UNWRAP parenthesized_expression IN if_statement 2").to_string());
                                },
                            }
                        }
                        let mut content = expr_parts.join(" ");
                        temp += remove_unnecessary_spaces(content).as_str();
                    }
                }
                dbg!(&temp);
                parts.push(temp);
            },
            "compound_statement" => {
                let inner_compound_statement = handle_inner_compound_statement(node, src.clone());
                parts.push(inner_compound_statement);
            },
            "else_clause" => handle_else_clause(node, src.clone()),
            _ => println!("{}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let result = construct_conditional(parts.clone());
    // println!("Finished if statement:\n{}\n", &result);
    return result;
}

fn handle_else_clause(root: Node, src: String) {
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => {
                let inner_compound_statement = handle_inner_compound_statement(node, src.clone());
                println!("Else clause inner compound statement:\n{}\n", inner_compound_statement);
            }
            "else" => {
                //? contents are always "else"
            },
            "switch_statement" => {
                let switch_statement = handle_switch_statement(node, src.clone());
                println!("Else clause switch statement: {}\n", switch_statement);
            },
            "for_statement" => handle_for_statement(node, src.clone()),
            "if_statement" => {
                //? This is where the `else if` branches go
                let if_statement = handle_if_statement(node, src.clone());
                let mut line = "else ".to_string() + if_statement.as_str() + "\n";
                line = utils::remove_unnecessary_spaces(line);
                println!("Else if: {}\n", line);
            },
            "return_statement" => {
                let return_statement = handle_return_statement(node, src.clone());
                println!("Else clause return: {}\n", return_statement);
            },
            "expression_statement" => {
                let expression_statement = handle_expression_statement(node, src.clone());
                println!("Else clause expression statement: {}\n", expression_statement);
            },
            _ => println!("You shouldn't be here 5: {}", node.grammar_name()),
        }
    }
}

//? This will eventually be where curly brace placement can happen
fn construct_conditional(parts: Vec<String>) -> String {
    let mut result = Vec::<String>::new();
    for (i, part) in parts.into_iter().enumerate() {
        match i {
            _ => {
                result.push(part)
            }
        }
    }
    return result.join(" ");
}

fn handle_switch_statement(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => {
                let compound_statement = handle_inner_compound_statement(node, src.clone());
                println!("Compound statement:\n{}", compound_statement);
            },
            "switch" => {
                result += "switch ";
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(node, src.clone());
                result += parenthesized_expression.as_str();
            },
            _ => println!("Switch: {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_case_statement(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "expression_statement" => {
                let expression_statement = handle_expression_statement(node, src.clone());
                result += "\t";
                result += (expression_statement + "\n").as_str();
            },
            "compound_statement" => {
                let compound_statement = handle_inner_compound_statement(node, src.clone());
                result += "\t";
                result += (compound_statement + "\n").as_str();
            },
            "case" => {
                result += "case ";
            },
            "break_statement" => {
                result += "\tbreak;";
            },
            "declaration" => {
                let declaration = handle_declaration(node, src.clone());
                result += (declaration + "\n").as_str();
            },
            "return_statement" => {
                let return_statement = handle_return_statement(node, src.clone());
                result += (return_statement + "\n").as_str();
            },
            "if_statement" => {
                let if_statement = handle_if_statement(node, src.clone());
                result += (if_statement + "\n").as_str();
            },
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                result += identifier.as_str();
            },
            "switch_statement" => {
                let switch_statement = handle_switch_statement(node, src.clone());
                result += (switch_statement + "\n").as_str();
            },
            "default" => {
                result += "default"
            },
            "number_literal" => {
                let number_literal = node.utf8_text(src.as_bytes()).unwrap_or("UNABLE TO UNWRAP number_literal IN case_statement");
                result += number_literal;
            },
            "char_literal" => {
                let char_literal = node.utf8_text(src.as_bytes()).unwrap_or("UNABLE TO UNWRAP char_literal IN case_statement");
                result += char_literal;
            },
            ":" => {
                result = result.trim_end().to_string();
                result += ":\n"
            },
            _ => println!("You shouldn't be here 6: {}\n", node.grammar_name()),
        }
    }
    return result;
}

fn handle_for_statement(root: Node, src: String) {
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            _ => (), // println!("{}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
}

fn handle_number_literal(root: Node, src: String) {
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            _ => (), // println!("{}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
}

fn handle_for_range_loop(root: Node, src: String) {
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            _ => (), // println!("{}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
}

fn handle_labeled_statement(root: Node, src: String) {
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            _ => (), // println!("{}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
}

fn handle_goto_statement(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            ";" => {
                result = result.trim_end().to_string();
                result += ";";
            },
            _ => {
                result += (node.utf8_text(src.as_bytes()).unwrap_or("UNABLE TO UNWRAP goto_statement").to_string() + " ").as_str();
            },
        }
    }
    return result;
}

fn handle_continue_statement(root: Node, src: String) {
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            _ => (), // println!("{}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
}

fn handle_while_statement(root: Node, src: String) {
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => {
                let compound_statement = handle_compound_statement(node.clone(), src.clone());
            },
            "while" => handle_while(node.clone(), src.clone()),
            _ => (), // println!("{}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
}

fn handle_while(root: Node, src: String) {
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => {
                let compound_statement = handle_compound_statement(node.clone(), src.clone());
            }
            "while" => {
                println!("Handling while keyword.\n");
            },
            _ => (), // println!("{}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
}

fn handle_do_statement(root: Node, src: String) {
    println!("{}", root.grammar_name());
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            _ => (), // println!("{}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
}

fn handle_argument_list(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "," => result += ", ",
            _ => result += node.utf8_text(src.as_bytes()).unwrap_or("UNABLE TO UNWRAP argument_list"),
        }
    }
    return result;
}

fn handle_return_statement(root: Node, src: String) -> String {
    let result: String;
    let mut vec = Vec::<String>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => {
                let compound_statement = handle_compound_statement(node.clone(), src.clone());
            },
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
            _ => println!("Handling ERROR:\nname: {}\ntext: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
}

fn handle_storage_class_specifier(root: Node, src: String) {
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            _ => (), // println!("{}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
}

fn handle_parenthesized_declarator(root: Node, src: String) {
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            _ => (), // println!("{}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
}

fn handle_parenthesized_expression(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            _ => (), // println!("{}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_init_declarator(root: Node, src: String) {
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            _ => (), // println!("{}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
}

fn handle_primitive_type(root: Node, src: String) {
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            _ => (), // println!("{}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
}

fn handle_function_declarator(root: Node, src: String) {
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            _ => (), // println!("{}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
}

fn handle_pointer_declarator(root: Node, src: String) {
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            _ => (), // println!("{}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
}

fn handle_binary_expression(root: Node, src: String) {
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            _ => (), // println!("{}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
}

fn handle_preproc_ifdef(root: Node, src: String) {
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            _ => (), // println!("{}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
}

fn handle_type_qualifier(root: Node, src: String) {
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            _ => (), // println!("{}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
}

fn handle_sized_type_specifier(root: Node, src: String) {
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            _ => (), // println!("{}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
}

fn handle_parameter_list(root: Node, src: String) {
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            _ => (), // println!("{}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
}

fn handle_preproc_if(root: Node, src: String) {
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            _ => (), // println!("{}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
}

fn handle_preproc_def(root: Node, src: String) {
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            _ => (), // println!("{}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
}

fn handle_preproc_function_def(root: Node, src: String) {
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            _ =>  (), // println!("{}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
}

fn handle_preproc_call(root: Node, src: String) {
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            _ => (), // println!("{}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
}

fn handle_type_definition(root: Node, src: String) {
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            _ => (), // println!("{}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
}

fn handle_identifier(root: Node, src: String) -> String {
    let mut result = String::new();
    match root.grammar_name() {
        "identifier" => {
            result = root.utf8_text(src.as_bytes()).unwrap_or("UNABLE TO UNWRAP identifier").to_string();  
        },
        "expression_statement" => {
            result = handle_expression_statement(root, src.clone());
        },
        ";" => {
            result = ";".to_string();
        }
        _ => println!("You shouldn't be here 7: {}", root.grammar_name()),
    }
    return result;
}

fn handle_using_declaration(root: Node, src: String) {
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            _ => (), // println!("{}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
}

fn handle_struct_specifier(root: Node, src: String) {
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            _ => (), // println!("{}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
}