use tree_sitter::{Tree, Node};
use crate::{c_format, utils::{self, add_all_leading_tabs, add_leading_whitespace, remove_all_spaces, remove_empty_lines, remove_unnecessary_spaces, remove_whitespace_before_commas, StringUtils}};

//todo: go through the 3 main match functions and delete anything redundant and/or unnecessary
pub fn traverse_ast(ast: Tree, src: String) {
    let root = ast.root_node();
    for child in root.children(&mut root.walk()) {
        match child.grammar_name() {
            "preproc_include" => {
                let preproc_include = handle_preproc_include(child, src.clone());
                println!("Finished includes:\n{}\n", preproc_include);
            },
            "declaration" => {
                let declaration = handle_declaration(child, src.clone());
                println!("Finished declaration:\n{}\n", declaration);
            },
            "function_definition" => {
                let function_definition = handle_function_definition(child, src.clone());
                //* this one's a little messy right now:
                // println!("Finished function definition:\n{}\n", function_definition);
            },
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
            "continue_statement" => {
                let continue_statement = "continue;\n";
            },
            "break_statement" => {
                let break_statement = "break;\n";
            },
            "return_statement" => {
                let return_statement = handle_return_statement(child, src.clone());
                println!("Finished return:\n{}\n", return_statement);
            },
            "comment" => {
                let comment = handle_comment(child, src.clone());
                println!("Finished comment:\n{}\n", comment);
            },
            ";" => (),
            // "preproc_ifdef" => handle_preproc_ifdef(child, src.clone()),
            // "preproc_if" => handle_preproc_if(child, src.clone()),
            // "preproc_def" => handle_preproc_def(child, src.clone()),
            // "preproc_function_def" => handle_preproc_function_def(child, src.clone()),
            // "preproc_call" => handle_preproc_call(child, src.clone()),
            // "type_definition" => handle_type_definition(child, src.clone()),
            // "struct_specifier" => handle_struct_specifier(child, src.clone()),
            // "sized_type_specifier" => handle_sized_type_specifier(child, src.clone()),
            // "ERROR" => handle_error(child, src.clone()),
            _ => (), // println!("Unknown grammar name 1: {}\n", &child.grammar_name()),
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
            "declaration" => {
                result = handle_declaration(node, src.clone());
            },
            "function_definition" => {
                result = handle_function_definition(node, src.clone());
            },
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
            "break_statement" => {
                result = "break;\n".to_string();
            },
            "continue_statement" => {
                result = "continue;\n".to_string();
            },
            "goto_statement" => {
                result = handle_goto_statement(node, src.clone());
            },
            "primitive_type" => {
                result = handle_primitive_type(node, src.clone());
            },
            "parenthesized_expression" => {
                result = handle_parenthesized_expression(node, src.clone());
            },
            "identifier" => {
                result = handle_identifier(node, src.clone());
            },
            "return_statement" => {
                result = handle_return_statement(node, src.clone());
            },
            "comment" => {
                result = handle_comment(node, src.clone());
            },
            ";" => (),
            "return" => (),
            "true" => (),
            "false" => (),
            "{" => (),
            "}" => (),
            // "preproc_ifdef" => handle_preproc_ifdef(node, src.clone()),
            // "preproc_if" => handle_preproc_if(node, src.clone()),
            // "preproc_def" => handle_preproc_def(node, src.clone()),
            // "preproc_function_def" => handle_preproc_function_def(node, src.clone()),
            // "preproc_call" => handle_preproc_call(node, src.clone()),
            // "type_definition" => handle_type_definition(node, src.clone()),
            // "struct_specifier" => handle_struct_specifier(node, src.clone()),
            // "for_statement" => handle_for_statement(node, src.clone()),
            // "for_range_loop" => handle_for_range_loop(node, src.clone()),
            // "while_statement" => handle_while_statement(node, src.clone()),
            // "do_statement" => handle_do_statement(node, src.clone()),
            // "labeled_statement" => handle_labeled_statement(node, src.clone()),
            // "parameter_list" => handle_parameter_list(node, src.clone()),
            // "init_declarator" => handle_init_declarator(node, src.clone()),
            // "while" => handle_while(node, src.clone()),
            // "ERROR" => handle_error(node, src.clone()),
            _ => (), // println!("Unknown grammar name 2: {}\n", node.grammar_name()),
        }
    }
    return result;
}

fn handle_preproc_include(root: Node, src: String) -> String {
    let result: String;
    let mut vec = Vec::<String>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            _ => vec.push(node.utf8_text(src.as_bytes()).unwrap().to_string()),
        }
    }
    result = vec.join(" ");
    return result;
}

fn handle_declaration(root: Node, src: String) -> String {
    let mut result: String;
    let mut parts = Vec::<String>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => {
                let compound_statement = handle_compound_statement(node.clone(), src.clone());
                parts.push(compound_statement);
            },
            "init_declarator" => {
                let init_declarator = handle_init_declarator(node, src.clone());
                parts.push(init_declarator);
            }
            "primitive_type" => {
                let primitive_type = handle_primitive_type(node, src.clone());
                parts.push(primitive_type);
            }
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                parts.push(identifier);
            }
            "function_declarator" => {
                let function_declarator = handle_function_declarator(node, src.clone());
                parts.push(function_declarator);
            }
            "ERROR" => {
                let error = handle_error(node, src.clone());
                parts.push(error);
            }
            "pointer_declarator" => {
                let pointer_declarator = handle_pointer_declarator(node, src.clone());
                parts.push(pointer_declarator);
            }
            ";" => parts.push(";".to_string()),
            "," => parts.push(",".to_string()),
            _ => println!("You shouldn't be here (declaration): {}\n", node.grammar_name()),
        }
    }
    result = parts.join(" ");
    if result.contains(",") { result = remove_whitespace_before_commas(result); }
    result = utils::remove_unnecessary_spaces(result);
    return result;
}

fn handle_function_definition(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => {
                let compound_statement = handle_compound_statement(node.clone(), src.clone());
                result += format!("{}\n", compound_statement).as_str();
            },
            "primitive_type" => {
                let primitive_type = handle_primitive_type(node.clone(), src.clone());
                result += format!("{} ", primitive_type).as_str();
            },
            "identifier" => {
                let identifier = handle_identifier(node.clone(), src.clone());
                result += format!("{}\n", identifier).as_str();
            },
            "function_declarator" => {
                let function_declarator = handle_function_declarator(node.clone(), src.clone());
                result += format!("{}\n", function_declarator).as_str();
            },
            "pointer_declarator" => {
                let pointer_declarator = handle_pointer_declarator(node.clone(), src.clone());
                result += format!("{}\n", pointer_declarator).as_str();
            },
            "parenthesized_declarator" => {
                let parenthesized_declarator = handle_parenthesized_declarator(node.clone(), src.clone());
                result += format!("{}\n", parenthesized_declarator).as_str();
            },
            "ERROR" => {
                let error = handle_error(node.clone(), src.clone());
                result += format!("{}\n", error).as_str();
            },
            "type_qualifier" => {
                let type_qualifier = handle_type_qualifier(node.clone(), src.clone());
                result += format!("{} ", type_qualifier).as_str();
            },
            // "storage_class_specifier" => handle_storage_class_specifier(node.clone(), src.clone()),
            // "struct_specifier" => handle_struct_specifier(node.clone(), src.clone()),
            // "sized_type_specifier" => handle_sized_type_specifier(node.clone(), src.clone()),
            _ => println!("You shouldn't be here (function_definition): {}\n", node.grammar_name()),
        }
    }
    return result;
}

fn handle_expression_statement(root: Node, src: String) -> String {
    let mut result = "".to_string();
    for node in root.children(&mut root.walk()) {
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
            "identifier" => {
                result = handle_identifier(root.clone(), src.clone());
            },
            "binary_expression" => {
                result = handle_binary_expression(root.clone(), src.clone());
            },
            ";" => (), // Handled in the functions called above.
            // "ERROR" => handle_error(root.clone(), src.clone()),
            _ => println!("You shouldn't be here (expression_statement): {}\n", node.grammar_name()),
        }
    }
    return result;
}

fn handle_assignment_expression(root: Node, src: String) -> String {
    let mut parts = Vec::<String>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            ";" => parts.push(";".to_string()),
            "=" => parts.push("=".to_string()),
            "assignment_expression" => {
                let assignment_expression = handle_inner_assignment_expression(node, src.clone());
                parts.push(assignment_expression);
            },
            "call_expression" => {
                let call_expression = handle_call_expression(node, src.clone());
                parts.push(call_expression);
            },
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                parts.push(identifier);
            },
            "subscript_expression" => {
                let subscript_expression = handle_subscript_expression(node, src.clone());
                parts.push(subscript_expression);
            },
            _ => println!("You shouldn't be here (assignment_expression): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap()),
        }
    }
    let result = parts.join(" ");
    return result;
}

fn handle_inner_assignment_expression(root: Node, src: String) -> String {
    let mut parts = Vec::<String>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => parts.push(handle_identifier(node, src.clone())),
            "call_expression" => parts.push(handle_call_expression(node, src.clone())),
            "assignment_expression" => {
                let assignment_expression = handle_assignment_expression(node, src.clone());
                parts.push(assignment_expression);
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(node, src.clone());
                dbg!(parenthesized_expression);
            },
            "pointer_expression" => {
                let mut pointer_expression = handle_pointer_expression(node, src.clone());
                if pointer_expression.chars().filter(|c| *c == '*').collect::<Vec<char>>().len() > 1 {
                    pointer_expression.remove(1);
                }
                parts.push(pointer_expression);
            },
            "conditional_expression" => {
                let conditional_expression = handle_conditional_expression(node, src.clone());
                // dbg!(conditional_expression);
            },
            "number_literal" => {
                let number_literal = handle_number_literal(node, src.clone());
                // dbg!(number_literal);
            },
            "char_literal" => {
                let number_literal = handle_char_literal(node, src.clone());
                // dbg!(number_literal);
            },
            "subscript_expression" => {
                let subscript_expression = handle_subscript_expression(node, src.clone());
                // dbg!(subscript_expression);
            },
            "field_expression" => {
                let field_expression = handle_field_expression(node, src.clone());
                // dbg!(field_expression);
            },
            "binary_expression" => {
                let binary_expression = handle_binary_expression(node, src.clone());
                // dbg!(binary_expression);
            },
            "unary_expression" => {
                let unary_expression = handle_unary_expression(node, src.clone());
                // dbg!(unary_expression);
            },
            "cast_expression" => {
                let cast_expression = handle_cast_expression(node, src.clone());
                // dbg!(cast_expression);
            },
            "sizeof_expression" => {
                let sizeof_expression = handle_sizeof_expression(node, src.clone());
                // dbg!(sizeof_expression);
            },
            "true" => parts.push("true".to_string()),
            "false" => parts.push("false".to_string()),
            "null" => parts.push("NULL".to_string()),
            "=" => parts.push("=".to_string()),
            "+=" => parts.push("=".to_string()),
            "-=" => parts.push("=".to_string()),
            "*=" => parts.push("=".to_string()),
            "/=" => parts.push("=".to_string()),
            "%=" => parts.push("=".to_string()),
            "|=" => parts.push("=".to_string()),
            "&=" => parts.push("=".to_string()),
            "ERROR" => {
                dbg!(node.grammar_name());
            },
            _ => println!("You shouldn't be here (inner_assignment_expression): {}\n", node.grammar_name()),
        }
    }
    let result = parts.join(" ").to_string();
    return result;
}

fn handle_inner_compound_statement(root: Node, src: String) -> String {
    let mut result = "".to_string();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
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
            "break_statement" => {
                result += "\tbreak;\n";
            },
            "continue_statement" => {
                result += "\tcontinue;\n";
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
            "compound_statement" => {
                let compound_statement = handle_nested_inner_compound_statement(node, src.clone());
                result += "\t";
                result += (compound_statement + "\n").as_str();
            },
            "while_statement" => {
                let while_statement = handle_while_statement(node, src.clone());
            },
            // "preproc_call" => {
            //     let preproc_call = handle_preproc_call(node, src.clone());
            // },
            // "for_statement" => handle_for_statement(node, src.clone()),
            // "do_statement" => handle_do_statement(node, src.clone()),
            _ => println!("You should't be here (inner_compound_statement): {}\n", node.grammar_name()),
        }
    }
    return result;
}

fn handle_update_expression(root: Node, src: String) -> String {
    let mut result = {
        root
            .clone()
            .utf8_text(src.as_bytes())
            .unwrap()
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
                let mut temp = String::new();
                for subnode in node.children(&mut node.walk()) {
                    match subnode.grammar_name() {
                        "identifier" => {
                            let identifier = handle_identifier(subnode, src.clone());
                            temp += identifier.as_str();
                        },
                        "argument_list" => {
                            let argument_list = handle_argument_list(subnode, src.clone());
                            temp += argument_list.as_str();
                        },
                        _ => println!("You shouldn't be here (middle of call_expression): {}\n", subnode.grammar_name()),
                    }
                }
                result += temp.as_str();
            },
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                result += identifier.as_str();
            },
            "argument_list" => {
                let argument_list = handle_argument_list(node, src.clone());
                result += argument_list.as_str();
            },
            ";" => result += ";",
            _ => println!("You shouldn't be here (call_expression): {}\n", node.grammar_name()),
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
                        let text = subnode.utf8_text(src.as_bytes()).unwrap();
                        temp += text;
                    }
                    else {
                        //* macros have an extra space `EXAMPLE (arg1, arg2)`
                        let mut expr_parts = Vec::<String>::new();
                        for inner_subnode in subnode.children(&mut subnode.walk()) {
                            match inner_subnode.grammar_name() {
                                "call_expression" => {
                                    let call_expression = handle_call_expression(inner_subnode, src.clone());
                                    expr_parts.push(call_expression);
                                },
                                _ => {
                                    expr_parts.push(inner_subnode.utf8_text(src.as_bytes()).unwrap().to_string());
                                },
                            }
                        }
                        let mut content = expr_parts.join(" ");
                        temp += remove_unnecessary_spaces(content).as_str();
                    }
                }
                parts.push(temp);
            },
            "compound_statement" => {
                let inner_compound_statement = handle_inner_compound_statement(node, src.clone());
                parts.push(inner_compound_statement);
            },
            "return_statement" => {
                let return_statement = handle_return_statement(node, src.clone());
                parts.push(return_statement);
            },
            "expression_statement" => {
                let expression_statement = handle_expression_statement(node, src.clone());
                parts.push(expression_statement);
            },
            "if_statement" => {
                let if_statement = handle_if_statement(node, src.clone());
                parts.push(if_statement);
            }
            "continue_statement" => {
                parts.push("continue;".to_string());
            }
            "comment" => {
                let comment = handle_comment(node, src.clone());
                parts.push(comment);
            }
            "else_clause" => {
                let else_clause = handle_else_clause(node, src.clone());
                parts.push(else_clause);
            },
            "goto_statement" => {
                let goto_statement = handle_goto_statement(node, src.clone());
                parts.push(goto_statement);
            },
            _ => println!("You shouldn't be here (if_statement): {}\n", node.grammar_name()),
        }
    }
    let result = construct_conditional(parts.clone());
    return result;
}

fn handle_else_clause(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => {
                let inner_compound_statement = handle_inner_compound_statement(node, src.clone());
                result += format!("\t{}\n", inner_compound_statement).as_str();
            }
            "else" => {
                result += "else {\n";
            },
            "switch_statement" => {
                let switch_statement = handle_switch_statement(node, src.clone());
                result += format!("\t{}\n", switch_statement).as_str();
            },
            "if_statement" => {
                //? This is where the `else if` branches go
                let if_statement = handle_if_statement(node, src.clone());
                let mut line = format!("else {}\n", if_statement);
                line = utils::remove_unnecessary_spaces(line);
                result += format!("\t{}\n", line).as_str();
            },
            "return_statement" => {
                let return_statement = handle_return_statement(node, src.clone());
                result += format!("\t{}\n", return_statement).as_str();
            },
            "expression_statement" => {
                let expression_statement = handle_expression_statement(node, src.clone());
                result += format!("\t{}\n", expression_statement).as_str();
            },
            "for_statement" => {
                let _ = handle_for_statement(node, src.clone());
            },
            _ => println!("You shouldn't be here (else_clause): {}\n", node.grammar_name()),
        }
    }
    return result;
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
                result += compound_statement.as_str();
            },
            "switch" => {
                result += "switch ";
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(node, src.clone());
                result += parenthesized_expression.as_str();
            },
            _ => println!("You shouldn't be here (switch_statement): {}\n", node.grammar_name()),
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
                let number_literal = node.utf8_text(src.as_bytes()).unwrap();
                result += number_literal;
            },
            "char_literal" => {
                let char_literal = node.utf8_text(src.as_bytes()).unwrap();
                result += char_literal;
            },
            ":" => {
                result = result.trim_end().to_string();
                result += ":\n"
            },
            _ => println!("You shouldn't be here (case_statement): {}\n", node.grammar_name()),
        }
    }
    return result;
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
                result += (node.utf8_text(src.as_bytes()).unwrap().to_string() + " ").as_str();
            },
        }
    }
    return result;
}

//? I actually implemented this already, but I think the result was actually the last line of the code inside the while statement.
//todo: fix that
fn handle_while_statement(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => {
                let compound_statement = handle_compound_statement(node.clone(), src.clone());
                dbg!(compound_statement);
            },
            "while" => {
                let while_result = handle_while(node.clone(), src.clone());
                dbg!(while_result);
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(node, src.clone());
                dbg!(parenthesized_expression);
            },
            "expression_statement" => {
                let expression_statement = handle_expression_statement(node, src.clone());
                dbg!(expression_statement);
            },
            _ => println!("You shouldn't be here (while_statement): {}: {}", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_while(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => {
                let compound_statement = handle_inner_compound_statement(node.clone(), src.clone());
                result += compound_statement.as_str();
            }
            "while" => {
                result += "while ";
            },
            _ => println!("You shouldn't be here (while): {}\n", node.grammar_name()),
        }
    }
    return result;
}

fn handle_argument_list(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "," => result += ", ",
            _ => result += node.utf8_text(src.as_bytes()).unwrap(),
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

fn handle_primitive_type(root: Node, src: String) -> String {
    let mut result = root.utf8_text(src.as_bytes()).unwrap();
    return result.to_string();
}

fn handle_function_declarator(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        if node.child_count() == 0 {
            result += node.utf8_text(src.as_bytes()).unwrap();
        }
        else {
            let mut temp = String::new();
            for subnode in node.children(&mut node.walk()) {
                let text = subnode.utf8_text(src.as_bytes()).unwrap();
                match subnode.grammar_name() {
                    "," => temp += ", ",
                    "(" => temp += "(",
                    ")" => temp += ")",
                    "parameter_declaration" => {
                        let parameter_declaration = handle_parameter_declaration(subnode, src.clone());
                        temp += parameter_declaration.as_str();
                    },
                    "variadic_parameter" => {
                        let variadic_parameter = handle_variadic_parameter(subnode, src.clone());
                        // dbg!(parameter_declaration)
                    },
                    _ => println!("You shouldn't be here (function_declarator): {}\n", subnode.grammar_name()),
                }
            }
            result += temp.as_str();
        }
    }
    return result;
}

fn handle_pointer_declarator(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "*" => {
                result += "* ";
            },
            "function_declarator" => {
                let function_declarator = handle_function_declarator(node, src.clone());
                result += function_declarator.as_str();
            },
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                result += identifier.as_str();
            }
            "pointer_declarator" => {
                let mut temp = String::new();
                for subnode in node.children(&mut node.walk()) {
                    let text = subnode.utf8_text(src.as_bytes()).unwrap();
                    match subnode.grammar_name() {
                        "*" => temp += "* ",
                        "function_declarator" => {
                            let function_declarator = handle_function_declarator(subnode, src.clone());
                            temp += function_declarator.as_str();
                        },
                        "identifier" => {
                            if subnode.child_count() == 0 {
                                temp += text;
                            }
                            else {
                                let mut temp = String::new();
                                for inner_subnode in subnode.children(&mut subnode.walk()) {
                                    match inner_subnode.grammar_name() {
                                        "identifier" => {
                                            let identifier = handle_identifier(inner_subnode, src.clone());
                                            temp += identifier.as_str();
                                        },
                                        "parameter_list" => {
                                            let parameter_list = handle_parameter_list(inner_subnode, src.clone());
                                            temp += parameter_list.as_str();
                                        },
                                        _ => println!("You shouldn't be here (inside of pointer_declarator): {}\n", inner_subnode.grammar_name()),
                                    }
                                }
                            }
                        },
                        _ => println!("You shouldn't be here (middle of pointer_declarator): {}\n", subnode.grammar_name()),
                    }
                }
            },
            _ => println!("You shouldn't be here (pointer_declarator): {}\n", node.grammar_name()),
        }
    }
    return result;
}

fn handle_identifier(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                result = root.utf8_text(src.as_bytes()).unwrap().to_string();
            },
            "expression_statement" => {
                result = handle_expression_statement(root, src.clone());
            },
            ";" => {
                result = ";".to_string();
            }
            _ => println!("You shouldn't be here (identifier): {}\n", root.grammar_name()),
        }
    }
    if result.len() == 0 {
        result = root.utf8_text(src.as_bytes()).unwrap().to_string();
    }
    return result;
}

fn handle_nested_inner_compound_statement(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "expression_statement" => {
                let expression_statement = handle_expression_statement(node, src.clone());
                result += format!("\t{}\n", expression_statement).as_str();
            }
            "if_statement" => {
                let if_statement = handle_if_statement(node, src.clone());
                result += format!("\t{}\n", if_statement).as_str();
            }
            "{" => result += "{\n",
            "}" => result += "}",
            _ => println!("Nested inner compound statement: {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    println!("Result:\n{}\n", &result);
    return result;
}

fn handle_binary_expression(root: Node, src: String) -> String {
    let mut parts = Vec::<String>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                parts.push(identifier);
            }
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(node, src.clone());
                parts.push(parenthesized_expression);
            }
            "field_expression" => {
                let field_expression = handle_field_expression(node, src.clone());
                parts.push(field_expression);
            }
            "number_literal" => {
                let number_literal = handle_number_literal(node, src.clone());
                parts.push(number_literal);
            }
            "sizeof_expression" => {
                let sizeof_expression = handle_sizeof_expression(node, src.clone());
                parts.push(sizeof_expression);
            }
            "!=" => parts.push("!=".to_string()),
            "==" => parts.push("==".to_string()),
            "+" => parts.push("+".to_string()),
            "-" => parts.push("-".to_string()),
            "*" => parts.push("*".to_string()),
            "/" => parts.push("/".to_string()),
            "%" => parts.push("%".to_string()),
            ">>" => parts.push("%".to_string()),
            "<<" => parts.push("%".to_string()),
            "|" => parts.push("%".to_string()),
            _ => println!("You shouldn't be here (binary_expression): {}\n", node.grammar_name()),
        }
    }
    let result = parts.join(" ");
    return result;
}

fn handle_parenthesized_declarator(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                result += identifier.as_str();
            },
            "(" => result += "(",
            ")" => result += ")",
            _ => println!("You shouldn't be here (parenthesized_declarator): {}\n", node.grammar_name()),
        }
    }
    return result;
}

fn handle_parenthesized_expression(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "call_expression" => {
                let call_expression = handle_call_expression(node, src.clone());
                result += call_expression.as_str();
            },
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                result += identifier.as_str();
            },
            "binary_expression" => {
                let binary_expression = handle_binary_expression(node, src.clone());
                result += binary_expression.as_str();
            },
            "assignment_expression" => {
                let assignment_expression = handle_assignment_expression(node, src.clone());
                result += assignment_expression.as_str();
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(node, src.clone());
                result += parenthesized_expression.as_str();
            },
            "pointer_expression" => {
                let pointer_expression = handle_pointer_expression(node, src.clone());
                result += pointer_expression.as_str();
            },
            "cast_expression" => {
                let cast_expression = handle_cast_expression(node, src.clone());
                result += cast_expression.as_str();
            },
            "(" => result += "(",
            ")" => result += ")",
            _ => println!("You shouldn't be here parenthesized_expression: {}\n", node.grammar_name()),
        }
    }
    return result;
}

fn handle_pointer_expression(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                result += identifier.as_str();
            }
            "pointer_expression" => {
                let pointer_expression = handle_inner_pointer_expression(node, src.clone());
                result += pointer_expression.as_str();
            }
            "*" => result += "* ",
            _ => println!("You shouldn't be here (pointer_expression): {}\n", node.grammar_name()),
        }
    }
    return result;
}

fn handle_inner_pointer_expression(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "*" => result += "* ",
            "identifier" => result += node.utf8_text(src.as_bytes()).unwrap(),
            _ => println!("You shouldn't be here (inner_pointer_expression): {}\n", node.grammar_name()),
        }
    }
    return result;
}

fn handle_parameter_declaration(root: Node, src: String) -> String {
    let mut parts = Vec::<String>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                parts.push(identifier);
            },
            "pointer_declarator" => {
                let pointer_declarator = handle_pointer_declarator(node, src.clone());
                parts.push(pointer_declarator);
            },
            "primitive_type" => {
                let primitive_type = handle_primitive_type(node, src.clone());
                parts.push(primitive_type);
            },
            "type_qualifier" => {
                let type_qualifier = handle_type_qualifier(node, src.clone());
                parts.push(type_qualifier);
            },
            "struct_specifier" => {
                let struct_specifier = handle_struct_specifier(node, src.clone());
                parts.push(struct_specifier);
            },
            _ => println!("You shouldn't be here (parameter_declaration): {}\n", node.grammar_name()),
        }
    }
    let mut result = parts.join(" ");
    result = utils::remove_unnecessary_spaces(result);
    return result;
}

fn handle_subscript_expression(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "[" => result += "[",
            "]" => result += "]",
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                result += identifier.as_str();
            }
            "update_expression" => {
                let update_expression = handle_update_expression(node, src.clone());
                result += update_expression.as_str();
            }
            "binary_expression" => {
                let binary_expression = handle_binary_expression(node, src.clone());
                result += binary_expression.as_str();
            }
            "number_literal" => {
                let number_literal = handle_number_literal(node, src.clone());
                result += number_literal.as_str();
            }
            "field_expression" => {
                let field_expression = handle_field_expression(node, src.clone());
                result += field_expression.as_str();
            }
            _ => println!("You shouldn't be here (subscript_expression): {}\n", node.grammar_name()),
        }
    }
    result = utils::remove_unnecessary_spaces(result);
    return result;
}

fn handle_field_expression(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                result += identifier.as_str();
            },
            "->" => result += "->",
            "." => result += ".",
            _ => println!("You shouldn't be here (field_expression): {}\n", node.grammar_name()),
        }
    }
    return result;
}

fn handle_error(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                result = handle_identifier(node, src.clone());
            },
            "{" => result = "{".to_string(),
            _ => println!("You shouldn't be here (ERROR): {}\n", node.grammar_name()),
        }
    }
    return result;
}

fn handle_cast_expression(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "(" => result += "(",
            ")" => result += ")",
            "type_descriptor" => {
                let type_descriptor = handle_type_descriptor(node, src.clone());
                result += type_descriptor.as_str();
            },
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                result += identifier.as_str();
            },
            "call_expression" => {
                let call_expression = handle_call_expression(node, src.clone());
                result += call_expression.as_str();
            },
            "pointer_expression" => {
                let pointer_expression = handle_pointer_expression(node, src.clone());
                result += pointer_expression.as_str();
            },
            "subscript_expression" => {
                let subscript_expression = handle_subscript_expression(node, src.clone());
                result += subscript_expression.as_str();
            },
            _ => println!("You shouldn't be here (cast_expression): {}\n", node.grammar_name()),
        }
    }
    return result;
}

fn handle_init_declarator(root: Node, src: String) -> String {
    let mut parts = Vec::<String>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                parts.push(identifier);
            },
            "number_literal" => {
                let number_literal = handle_number_literal(node, src.clone());
                parts.push(number_literal);
            },
            "call_expression" => {
                let call_expression = handle_call_expression(node, src.clone());
                parts.push(call_expression);
            },
            "field_expression" => {
                let field_expression = handle_field_expression(node, src.clone());
                parts.push(field_expression);
            },
            "subscript_expression" => {
                let subscript_expression = handle_subscript_expression(node, src.clone());
                parts.push(subscript_expression);
            },
            "pointer_declarator" => {
                let pointer_declarator = handle_pointer_declarator(node, src.clone());
                parts.push(pointer_declarator);
            },
            "pointer_expression" => {
                let pointer_expression = handle_pointer_expression(node, src.clone());
                parts.push(pointer_expression);
            },
            "cast_expression" => {
                let cast_expression = handle_cast_expression(node, src.clone());
                parts.push(cast_expression);
            },
            "binary_expression" => {
                let binary_expression = handle_binary_expression(node, src.clone());
                parts.push(binary_expression);
            },
            "initializer_list" => {
                let initializer_list = handle_initializer_list(node, src.clone());
                println!("Finished initializer list: {}\n", initializer_list);
                // parts.push(initializer_list);
            },
            "conditional_expression" => {
                let conditional_expression = handle_conditional_expression(node, src.clone());
                // parts.push(conditional_expression);
            },
            "array_declarator" => {
                let array_declarator = handle_array_declarator(node, src.clone());
                // parts.push(array_declarator);
            },
            "=" => parts.push("=".to_string()),
            "false" => parts.push("false".to_string()),
            "true" => parts.push("true".to_string()),
            "null" => parts.push("NULL".to_string()),
            _ => println!("Init declarator: {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let result = parts.join(" ");
    return result;
}

fn handle_type_descriptor(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                result += identifier.as_str();
            }
            "primitive_type" => {
                let primitive_type = handle_primitive_type(node, src.clone());
                result += primitive_type.as_str();
            }
            "abstract_pointer_declarator" => {
                let abstract_pointer_declaration = handle_abstract_pointer_declarator(node, src.clone());
                result += format!("{} ", abstract_pointer_declaration).as_str();
            }
            _ => println!("You shouldn't be here (type_descriptor): {}", node.grammar_name()),
        }
    }
    return result;
}

fn handle_initializer_list(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                result += identifier.as_str();
            },
            "number_literal" => {
                let number_literal = handle_number_literal(node, src.clone());
                result += format!("{} ", number_literal).as_str();
            },
            "{" => result += "{",
            "}" => result += "}",
            "," => result += ", ",
            _ => println!("You shouldn't be here (initializer_list): {}\n", node.grammar_name()),
        }
    }
    return result;
}

fn handle_array_declarator(root: Node, src: String) {
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            _ => (), // println!("{}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
}

fn handle_sizeof_expression(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "sizeof" => result += "sizeof",
            "(" => result += "(",
            ")" => result += ")",
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(node, src.clone());
                result += parenthesized_expression.as_str();
            },
            "type_descriptor" => {
                let type_descriptor = handle_type_descriptor(node, src.clone());
                result += type_descriptor.as_str();
            },
            _ => println!("Sizeof expression: {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_char_literal(root: Node, src: String) {
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            _ => (), // println!("{}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
}

fn handle_variadic_parameter(root: Node, src: String) {
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            _ => (), // println!("{}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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

fn handle_using_declaration(root: Node, src: String) {
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            _ => (), // println!("{}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
}

fn handle_for_statement(root: Node, src: String) {
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            _ => (), // println!("For statement: {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
}

fn handle_number_literal(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            _ => (), // println!("{}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
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

fn handle_continue_statement(root: Node, src: String) {
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            _ => (), // println!("{}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
}

fn handle_unary_expression(root: Node, src: String) {
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

fn handle_type_qualifier(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "const" => {
                result = "const".to_string();
            }
            _ => println!("You shouldn't be here (type_qualifier): {}\n", node.grammar_name()),
        }
    }
    return result;
}

fn handle_sized_type_specifier(root: Node, src: String) {
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            _ => (), // println!("{}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
}

fn handle_parameter_list(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            _ => (), // println!("Parameter list: {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
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

fn handle_do_statement(root: Node, src: String) {
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            _ => (), // println!("{}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
}

fn handle_conditional_expression(root: Node, src: String) {
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            _ => (), // println!("{}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
}

fn handle_struct_specifier(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            _ => (), // println!("{}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_abstract_pointer_declarator(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "abstract_pointer_declarator" => result = "**".to_string(),
            "*" => result = "*".to_string(),
            _ => println!("You shouldn't be here (abstract_pointer_declarator): {}\n", node.grammar_name()), 
        }
    }
    return result;
}