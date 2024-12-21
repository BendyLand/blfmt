use tree_sitter::{Tree, Node};
use crate::{c_format, utils};

pub fn traverse_c_ast(ast: Tree, src: String, style: utils::Style) -> String {
    let root = ast.root_node();
    let mut result = String::new();
    let mut last_group_kind = "";
    for child in root.children(&mut root.walk()) {
        match child.grammar_name() {
            "preproc_include" => {
                if last_group_kind != "preproc_include" { result += "\n"; }
                let preproc_include = handle_preproc_include(child, src.clone());
                result += format!("{}\n", preproc_include).as_str();
                last_group_kind = "preproc_include";
            },
            "declaration" => {
                if last_group_kind.contains("preproc") { result += "\n"; }
                let mut declaration = handle_declaration(child, src.clone());
                if declaration.contains("*") { declaration = utils::remove_pointer_spaces(declaration); }
                result += format!("{}\n", declaration).as_str();
                last_group_kind = "declaration";
            },
            "function_definition" => {
                if last_group_kind != "function_definition".to_string() { result += "\n"; }
                let function_definition = handle_function_definition(child, src.clone());
                result += format!("{}\n\n", function_definition).as_str();
                last_group_kind = "function_definition";
            },
            "expression_statement" => {
                let expression_statement = handle_expression_statement(child, src.clone());
                result += format!("{}\n\n", expression_statement).as_str();
                last_group_kind = "expression_statement";
            },
            "compound_statement" => {
                let compound_statement = handle_compound_statement(child, src.clone());
                result += format!("{}\n\n", compound_statement).as_str();
                last_group_kind = "compound_statement";
            },
            "if_statement" => {
                let if_statement = handle_if_statement(child, src.clone());
                result += format!("{}\n\n", if_statement).as_str();
                last_group_kind = "if_statement";
            },
            "switch_statement" => {
                let switch_statement = handle_switch_statement(child, src.clone());
                result += format!("{}\n\n", switch_statement).as_str();
                last_group_kind = "switch_statement";
            },
            "continue_statement" => {
                let continue_statement = "continue;\n";
                result += continue_statement;
                last_group_kind = "continue_statement";
            },
            "break_statement" => {
                let break_statement = "break;\n";
                result += break_statement;
                last_group_kind = "break_statement";
            },
            "return_statement" => {
                let return_statement = handle_return_statement(child, src.clone());
                result += format!("{}\n\n", return_statement).as_str();
                last_group_kind = "return_statement";
            },
            "comment" => {
                if last_group_kind == "declaration" || 
                last_group_kind == "preproc_include" { result += "\n"; }
                let comment = handle_comment(child, src.clone());
                result += format!("{}\n", comment).as_str();
                last_group_kind = "comment";
            },
            "ERROR" => {
                let error = handle_error(child, src.clone());
                result += format!("{}\n\n", error).as_str();
                last_group_kind = "ERROR";
            },
            "type_definition" => {
                let mut type_definition = handle_type_definition(child, src.clone());
                result += format!("{}\n\n", type_definition).as_str();
                last_group_kind = "type_definition";
            },
            "struct_specifier" => {
                let struct_specifier = handle_struct_specifier(child, src.clone());
                result += format!("{}\n\n", struct_specifier).as_str();
                last_group_kind = "struct_specifier";
            },
            "preproc_def" => {
                if last_group_kind != "preproc_def" { result += "\n"; }
                let preproc_def = handle_preproc_def(child, src.clone());
                result += format!("{}\n", preproc_def).as_str();
                last_group_kind = "preproc_def";
            },
            "preproc_ifdef" => {
                let preproc_ifdef = handle_preproc_ifdef(child, src.clone());
                result += format!("{}\n\n", preproc_ifdef).as_str();
                last_group_kind = "preproc_ifdef";
            },
            "preproc_if" => {
                let preproc_if = handle_preproc_if(child, src.clone());
                result += format!("{}\n\n", preproc_if).as_str();
                last_group_kind = "preproc_if";
            },
            "preproc_function_def" => {
                let preproc_function_def = handle_preproc_function_def(child, src.clone());
                result += format!("{}\n\n", preproc_function_def).as_str();
                last_group_kind = "preproc_function_def";
            },
            "preproc_call" => {
                let preproc_call = handle_preproc_call(child, src.clone());
                result += format!("{}\n\n", preproc_call).as_str();
                last_group_kind = "preproc_call";
            },
            "sized_type_specifier" => {
                let sized_type_specifier = handle_sized_type_specifier(child, src.clone());
                result += format!("{}\n\n", sized_type_specifier).as_str();
                last_group_kind = "sized_type_specifier";
            },
            ";" => (), // handled in functions above
            _ => println!("Unknown grammar name 1: {}\n", &child.grammar_name()),
        }
    }
    result = utils::sort_include_groups(result);
    utils::format_else_lines(&mut result, &style);
    result = result.trim_start().to_string();
    return result;
}

fn handle_compound_statement(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "preproc_include" => {
                let preproc_include = handle_preproc_include(node, src.clone());
                result += format!("\t{}\n", preproc_include).as_str();
            },
            "declaration" => {
                let declaration = handle_declaration(node, src.clone());
                result += format!("\t{}\n", declaration).as_str();
            },
            "function_definition" => {
                let function_definition = handle_function_definition(node, src.clone());
                result += format!("\t{}\n", function_definition).as_str();
            },
            "expression_statement" => {
                let expression_statement = handle_expression_statement(node, src.clone());
                result += format!("\t{}\n", expression_statement).as_str();
            },
            "compound_statement" => {
                let compound_statement = handle_inner_compound_statement(node, src.clone());
                result += format!("\t{}\n", compound_statement).as_str();
            },
            "if_statement" => {
                let mut if_statement = handle_if_statement(node, src.clone());
                if_statement = utils::add_all_leading_tabs(if_statement);
                result += format!("{}\n", if_statement).as_str();
            },
            "switch_statement" => {
                let mut switch_statement = handle_switch_statement(node, src.clone());
                switch_statement = utils::remove_blank_lines(switch_statement.split("\n").collect::<Vec<&str>>());
                switch_statement = utils::add_all_leading_tabs(switch_statement);
                result += format!("{}\n", switch_statement).as_str();
            },
            "case_statement" => {
                let case_statement = handle_case_statement(node, src.clone());
                result += format!("\t{}\n", case_statement).as_str();
            },
            "break_statement" => {
                let break_statement = "break;\n".to_string();
                result += format!("\t{}\n", break_statement).as_str();
            },
            "continue_statement" => {
                let continue_statement = "continue;\n".to_string();
                result += format!("\t{}\n", continue_statement).as_str();
            },
            "goto_statement" => {
                let goto_statement = handle_goto_statement(node, src.clone());
                result += format!("\t{}\n", goto_statement).as_str();
            },
            "primitive_type" => {
                let primitive_type = handle_primitive_type(node, src.clone());
                result += format!("\t{}\n", primitive_type).as_str();
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(node, src.clone());
                result += format!("\t{}\n", parenthesized_expression).as_str();
            },
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                result += format!("\t{}\n", identifier).as_str();
            },
            "return_statement" => {
                let return_statement = handle_return_statement(node, src.clone());
                result += format!("\t{}\n", return_statement).as_str();
            },
            "comment" => {
                let comment = handle_comment(node, src.clone());
                if comment.starts_with(" ") { result += format!("{}\n", comment).as_str(); }
                else { result += format!("\t{}\n", comment).as_str(); }
            },
            "labeled_statement" => {
                let labeled_statement = handle_labeled_statement(node, src.clone());
                result += format!("\t{}\n", labeled_statement).as_str();
            },
            "while_statement" => {
                let mut while_statement = handle_while_statement(node, src.clone());
                while_statement = utils::add_all_leading_tabs(while_statement);
                result += format!("{}\n", while_statement).as_str();
            },
            "for_statement" => {
                let mut for_statement = handle_for_statement(node, src.clone());
                for_statement = utils::add_all_leading_tabs(for_statement);
                result += format!("{}\n", for_statement).as_str();
            },
            "preproc_def" => {
                let mut preproc_def = handle_preproc_def(node, src.clone());
                preproc_def = utils::add_all_leading_tabs(preproc_def);
                result += format!("{}\n", preproc_def).as_str();
            },
            "type_definition" => {
                let type_definition = handle_type_definition(node, src.clone());
                result += format!("\t{}\n", type_definition).as_str();
            },
            "struct_specifier" => {
                let struct_specifier = handle_struct_specifier(node, src.clone());
                result += format!("\t{}\n", struct_specifier).as_str();
            },
            "init_declarator" => {
                let init_declarator = handle_init_declarator(node, src.clone());
                result += format!("\t{}\n", init_declarator).as_str();
            },
            "ERROR" => {
                let error = handle_error(node, src.clone());
                result += format!("\t{}\n", error).as_str();
            },
            "parameter_list" => {
                dbg!("Here (parameter_list)");
                let parameter_list = handle_parameter_list(node, src.clone());
                result += format!("\t{}\n", parameter_list).as_str();
            },
            "preproc_ifdef" => {
                let preproc_ifdef = handle_preproc_ifdef(node, src.clone());
                result += format!("\t{}\n", preproc_ifdef).as_str();
            },
            "preproc_if" => {
                let mut preproc_if = handle_preproc_if(node, src.clone());
                preproc_if = utils::add_all_leading_tabs(preproc_if);
                result += format!("{}\n", preproc_if).as_str();
            },
            "preproc_function_def" => {
                let mut preproc_function_def = handle_preproc_function_def(node, src.clone());
                preproc_function_def = utils::add_all_leading_tabs(preproc_function_def);
                result += format!("{}\n", preproc_function_def).as_str();
            },
            "preproc_call" => {
                let preproc_call = handle_preproc_call(node, src.clone());
                result += format!("\t{}\n", preproc_call).as_str();
            },
            "do_statement" => {
                let mut do_statement = handle_do_statement(node, src.clone());
                do_statement = utils::add_all_leading_tabs(do_statement);
                result += format!("{}\n", do_statement).as_str();
            },
            "return" => {
                dbg!(node.utf8_text(src.as_bytes()).unwrap());
            },
            "true" => {
                dbg!(node.utf8_text(src.as_bytes()).unwrap());
            },
            "false" => {
                dbg!(node.utf8_text(src.as_bytes()).unwrap());
            },
            ";" => result += ";",
            "{" => result += "{\n",
            "}" => result += "}",
            _ => println!("Unknown grammar name 2: {}\n", node.grammar_name()),
        }
    }
    return result;
}

fn handle_preproc_include(root: Node, src: String) -> String {
    let result: String;
    let mut vec = Vec::<String>::new();
    for node in root.children(&mut root.walk()) {
        vec.push(node.utf8_text(src.as_bytes()).unwrap().to_string());
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
            },
            "primitive_type" => {
                let primitive_type = handle_primitive_type(node, src.clone());
                parts.push(primitive_type);
            },
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                parts.push(identifier);
            },
            "function_declarator" => {
                let function_declarator = handle_function_declarator(node, src.clone());
                parts.push(function_declarator);
            },
            "type_qualifier" => {
                let type_qualifier = handle_type_qualifier(node, src.clone());
                parts.push(type_qualifier);
            },
            "ERROR" => {
                let error = handle_error(node, src.clone());
                parts.push(error);
            },
            "pointer_declarator" => {
                let pointer_declarator = handle_pointer_declarator(node, src.clone());
                parts.push(pointer_declarator);
            },
            "array_declarator" => {
                let array_declarator = handle_array_declarator(node, src.clone());
                parts.push(array_declarator);
            },
            "storage_class_specifier" => {
                let storage_class_specifier = handle_storage_class_specifier(node, src.clone());
                parts.push(storage_class_specifier);
            },
            "struct_specifier" => {
                let struct_specifier = handle_struct_specifier(node, src.clone());
                parts.push(struct_specifier);
            },
            "sized_type_specifier" => {
                let sized_type_specifier = handle_sized_type_specifier(node, src.clone());
                parts.push(sized_type_specifier);
            },
            ";" => parts.push(";".to_string()),
            "," => parts.push(",".to_string()),
            _ => println!("You shouldn't be here (declaration): {}\n", node.grammar_name()),
        }
    }
    result = parts.join(" ");
    if result.contains(",") { result = utils::remove_whitespace_before_commas(result); }
    result = utils::remove_unnecessary_spaces(result);
    if result.contains("**") { result = utils::remove_pointer_spaces(result); }
    return result;
}

fn handle_function_definition(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => {
                let compound_statement = handle_compound_statement(node, src.clone());
                result += format!("{}\n", compound_statement).as_str();
            },
            "primitive_type" => {
                let primitive_type = handle_primitive_type(node, src.clone());
                result += format!("{} ", primitive_type).as_str();
            },
            "type_qualifier" => {
                let type_qualifier = handle_type_qualifier(node, src.clone());
                result += format!("{} ", type_qualifier).as_str();
            },
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                result += format!("{} ", identifier).as_str();
            },
            "ERROR" => {
                // Usually an unknown specifier, like a custom alias for `static`.
                let error = handle_error(node, src.clone());
                result += format!("{} ", error).as_str();
            },
            "function_declarator" => {
                let function_declarator = handle_function_declarator(node, src.clone());
                result += format!("{}\n", function_declarator).as_str();
            },
            "pointer_declarator" => {
                let pointer_declarator = handle_pointer_declarator(node, src.clone());
                result = result.trim_end().to_string();
                result += format!("{}\n", pointer_declarator).as_str();
            },
            "parenthesized_declarator" => {
                let parenthesized_declarator = handle_parenthesized_declarator(node, src.clone());
                result += format!("{}\n", parenthesized_declarator).as_str();
            },
            "struct_specifier" => {
                let struct_specifier = handle_struct_specifier(node.clone(), src.clone());
                result += format!("{}\n", struct_specifier).as_str();
            },
            "storage_class_specifier" => {
                let storage_class_specifier = handle_storage_class_specifier(node, src.clone());
                result += format!("{} ", storage_class_specifier).as_str();
            },
            "sized_type_specifier" => {
                let sized_type_specifier = handle_sized_type_specifier(node, src.clone());
                result += format!("{} ", sized_type_specifier).as_str();
            },
            _ => println!("You shouldn't be here (function_definition): {}\n", node.grammar_name()),
        }
    }
    result = utils::remove_pointer_spaces(result);
    result = result.trim_end().to_string();
    return result;
}

fn handle_expression_statement(root: Node, src: String) -> String {
    let mut result = "".to_string();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "assignment_expression" => {
                result = handle_assignment_expression(node, src.clone());
            },
            "update_expression" => {
                result = handle_update_expression(node, src.clone());
            },
            "call_expression" => {
                result = handle_call_expression(node, src.clone());
            },
            "identifier" => {
                result = handle_identifier(node, src.clone());
            },
            "binary_expression" => {
                result = handle_binary_expression(node, src.clone());
            },
            "pointer_expression" => {
                result = handle_pointer_expression(node, src.clone());
            },
            "ERROR" => {
                result = handle_error(node, src.clone());
            },
            ";" => result += ";",
            _ => println!("You shouldn't be here (expression_statement): {}\n", node.grammar_name()),
        }
    }
    result = utils::remove_unnecessary_spaces(result);
    return result;
}

fn handle_assignment_expression(root: Node, src: String) -> String {
    let mut parts = Vec::<String>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            ";" => parts.push(";".to_string()),
            "=" => parts.push("=".to_string()),
            "+=" => parts.push("=".to_string()),
            "-=" => parts.push("=".to_string()),
            "*=" => parts.push("=".to_string()),
            "/=" => parts.push("=".to_string()),
            "%=" => parts.push("=".to_string()),
            "<<=" => parts.push("=".to_string()),
            ">>=" => parts.push("=".to_string()),
            "&=" => parts.push("=".to_string()),
            "|=" => parts.push("=".to_string()),
            "null" => parts.push("NULL".to_string()),
            "true" => parts.push("true".to_string()),
            "false" => parts.push("false".to_string()),
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
            "field_expression" => {
                let field_expression = handle_field_expression(node, src.clone());
                parts.push(field_expression);
            },
            "number_literal" => {
                let number_literal = handle_number_literal(node, src.clone());
                parts.push(number_literal);
            },
            "char_literal" => {
                let char_literal = handle_char_literal(node, src.clone());
                parts.push(char_literal);
            },
            "cast_expression" => {
                let cast_expression = handle_cast_expression(node, src.clone());
                parts.push(cast_expression);
            },
            "binary_expression" => {
                let binary_expression = handle_binary_expression(node, src.clone());
                parts.push(binary_expression);
            },
            "unary_expression" => {
                let unary_expression = handle_unary_expression(node, src.clone());
                parts.push(unary_expression);
            },
            "pointer_expression" => {
                let pointer_expression = handle_pointer_expression(node, src.clone());
                parts.push(pointer_expression);
            },
            "conditional_expression" => {
                let conditional_expression = handle_conditional_expression(node, src.clone());
                parts.push(conditional_expression);
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(node, src.clone());
                parts.push(parenthesized_expression);
            },
            "sizeof_expression" => {
                let sizeof_expression = handle_sizeof_expression(node, src.clone());
                parts.push(sizeof_expression);
            },
            "ERROR" => {
                let error = handle_error(node, src.clone());
                parts.push(error);
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
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                parts.push(identifier);
            },
            "call_expression" => {
                let call_expression = handle_call_expression(node, src.clone());
                parts.push(call_expression);
            },
            "assignment_expression" => {
                let assignment_expression = handle_assignment_expression(node, src.clone());
                parts.push(assignment_expression);
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(node, src.clone());
                parts.push(parenthesized_expression);
            },
            "pointer_expression" => {
                let pointer_expression = handle_pointer_expression(node, src.clone());
                parts.push(pointer_expression);
            },
            "conditional_expression" => {
                let conditional_expression = handle_conditional_expression(node, src.clone());
                parts.push(conditional_expression);
            },
            "number_literal" => {
                let number_literal = handle_number_literal(node, src.clone());
                parts.push(number_literal);
            },
            "char_literal" => {
                let char_literal = handle_char_literal(node, src.clone());
                parts.push(char_literal);
            },
            "subscript_expression" => {
                let subscript_expression = handle_subscript_expression(node, src.clone());
                parts.push(subscript_expression);
            },
            "field_expression" => {
                let field_expression = handle_field_expression(node, src.clone());
                parts.push(field_expression);
            },
            "binary_expression" => {
                let binary_expression = handle_binary_expression(node, src.clone());
                parts.push(binary_expression);
            },
            "unary_expression" => {
                let unary_expression = handle_unary_expression(node, src.clone());
                parts.push(unary_expression);
            },
            "cast_expression" => {
                let cast_expression = handle_cast_expression(node, src.clone());
                parts.push(cast_expression);
            },
            "sizeof_expression" => {
                let sizeof_expression = handle_sizeof_expression(node, src.clone());
                parts.push(sizeof_expression);
            },
            "ERROR" => {
                let error = handle_error(node, src.clone());
                parts.push(error);
            },
            "true" => parts.push("true".to_string()),
            "false" => parts.push("false".to_string()),
            "null" => parts.push("NULL".to_string()),
            "=" => parts.push("=".to_string()),
            "+=" => parts.push("+=".to_string()),
            "-=" => parts.push("-=".to_string()),
            "*=" => parts.push("*=".to_string()),
            "/=" => parts.push("/=".to_string()),
            "%=" => parts.push("%=".to_string()),
            "|=" => parts.push("|=".to_string()),
            "&=" => parts.push("&=".to_string()),
            ">>=" => parts.push(">>=".to_string()),
            "<<=" => parts.push("<<=".to_string()),
            _ => println!("You shouldn't be here (inner_assignment_expression): {}\n", node.grammar_name()),
        }
    }
    let result = parts.join(" ").to_string();
    return result;
}

fn handle_inner_compound_statement(root: Node, src: String) -> String {
    let mut parts = Vec::<String>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "expression_statement" => {
                let expression_statement = handle_expression_statement(node, src.clone());
                parts.push(format!("\t{}", expression_statement));
            },
            "return_statement" => {
                let return_statement = handle_return_statement(node, src.clone());
                parts.push(format!("\t{}", return_statement));
            },
            "goto_statement" => {
                let goto_statement = handle_goto_statement(node, src.clone());
                parts.push(format!("\t{}", goto_statement));
            },
            "if_statement" => {
                let mut if_statement = handle_if_statement(node, src.clone());
                if_statement = utils::add_all_leading_tabs(if_statement);
                parts.push(if_statement);
            },
            "switch_statement" => {
                let mut switch_statement = handle_switch_statement(node, src.clone());
                switch_statement = utils::add_all_leading_tabs(switch_statement);
                parts.push(switch_statement);
            },
            "break_statement" => {
                parts.push("\tbreak;".to_string());
            },
            "continue_statement" => {
                parts.push("\tcontinue;".to_string());
            },
            "case_statement" => {
                let mut case_statement = handle_case_statement(node, src.clone());
                // Without the line below, "case" vertically aligns with "switch".
                // case_statement = utils::add_all_leading_tabs(case_statement);
                case_statement = utils::remove_blank_lines(case_statement.split("\n").collect::<Vec<&str>>());
                parts.push(case_statement);
            },
            "declaration" => {
                let declaration = handle_declaration(node, src.clone());
                parts.push(format!("\t{}", declaration));
            },
            "comment" => {
                let mut comment = handle_comment(node, src.clone());
                comment = utils::add_all_leading_tabs(comment);
                parts.push(comment);
            },
            "compound_statement" => {
                let compound_statement = handle_nested_inner_compound_statement(node, src.clone());
                parts.push(format!("\t{}", compound_statement));
            },
            "while_statement" => {
                let mut while_statement = handle_while_statement(node, src.clone());
                while_statement = utils::add_all_leading_tabs(while_statement);
                parts.push(while_statement);
            },
            "preproc_call" => {
                let preproc_call = handle_preproc_call(node, src.clone());
                parts.push(format!("\t{}", preproc_call));
            },
            "do_statement" => {
                let mut do_statement = handle_do_statement(node, src.clone());
                do_statement = utils::add_all_leading_tabs(do_statement);
                parts.push(do_statement);
            },
            "for_statement" => {
                let mut for_statement = handle_for_statement(node, src.clone());
                for_statement = utils::add_all_leading_tabs(for_statement);
                parts.push(for_statement);
            },
            "{" => parts.push("{".to_string()),
            "}" => parts.push("}".to_string()),
            _ => println!("You should't be here (inner_compound_statement): {}\n", node.grammar_name()),
        }
    }
    let result = parts.join("\n");
    return result;
}

fn handle_update_expression(root: Node, src: String) -> String {
    let mut result = root.utf8_text(src.as_bytes()).unwrap().to_string();
    result = utils::remove_all_spaces(result);
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
                        "parenthesized_expression" => {
                            let parenthesized_expression = handle_parenthesized_expression(subnode, src.clone());
                            temp += parenthesized_expression.as_str();
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
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(node, src.clone());
                result += parenthesized_expression.as_str();
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
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(node, src.clone());
                parts.push(parenthesized_expression);
            },
            "switch_statement" => {
                let switch_statement = handle_switch_statement(node, src.clone());
                parts.push(switch_statement);
            },
            "break_statement" => {
                parts.push("break;".to_string());
            },
            _ => println!("You shouldn't be here (if_statement): {}\n", node.grammar_name()),
        }
    }
    let result = parts.join(" ");
    return result;
}

//todo: Eventually make a parameter to handle else style
fn handle_else_clause(root: Node, src: String) -> String {
    let mut pieces = Vec::<String>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => {
                let inner_compound_statement = handle_inner_compound_statement(node, src.clone());
                pieces.push(inner_compound_statement);
            }
            "else" => {
                //* This is where placement of "else" happens
                pieces.push("else".to_string());
            },
            "switch_statement" => {
                let switch_statement = handle_switch_statement(node, src.clone());
                pieces.push(switch_statement);
            },
            "if_statement" => {
                let if_statement = handle_if_statement(node, src.clone());
                pieces.push(if_statement);
            },
            "return_statement" => {
                let return_statement = handle_return_statement(node, src.clone());
                pieces.push(return_statement);
            },
            "expression_statement" => {
                let expression_statement = handle_expression_statement(node, src.clone());
                pieces.push(expression_statement);
            },
            "for_statement" => {
                let for_statement = handle_for_statement(node, src.clone());
                pieces.push(for_statement);
            },
            _ => println!("You shouldn't be here (else_clause): {}\n", node.grammar_name()),
        }
    }
    let result = pieces.join(" ");
    return result;
}

fn handle_switch_statement(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "switch" => result += "switch ",
            "compound_statement" => {
                let compound_statement = handle_inner_compound_statement(node, src.clone());
                result += compound_statement.as_str();
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(node, src.clone());
                result += format!("{} ", parenthesized_expression).as_str();
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
                result += format!("\t{}\n", expression_statement).as_str();
            },
            "compound_statement" => {
                let compound_statement = handle_inner_compound_statement(node, src.clone());
                result += format!("{}\n", compound_statement).as_str();
            },
            "case" => {
                result += "case ";
            },
            "break_statement" => {
                result += "\tbreak;";
            },
            "declaration" => {
                let declaration = handle_declaration(node, src.clone());
                result += format!("\t{}\n", declaration).as_str();
            },
            "return_statement" => {
                let return_statement = handle_return_statement(node, src.clone());
                result += format!("\t{}\n", return_statement).as_str();
            },
            "if_statement" => {
                let if_statement = handle_if_statement(node, src.clone());
                result += format!("\t{}\n", if_statement).as_str();
            },
            "goto_statement" => {
                let goto_statement = handle_goto_statement(node, src.clone());
                result += format!("\t{}\n", goto_statement).as_str();
            },
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                result += identifier.as_str();
            },
            "switch_statement" => {
                let switch_statement = handle_switch_statement(node, src.clone());
                result += format!("{}\n", switch_statement).as_str();
            },
            "default" => {
                result += "default"
            },
            "number_literal" => {
                let number_literal = handle_number_literal(node, src.clone());
                result += number_literal.as_str();
            },
            "char_literal" => {
                let char_literal = handle_char_literal(node, src.clone());
                result += char_literal.as_str();
            },
            "comment" => {
                let comment = handle_comment(node, src.clone());
                result += format!("\t{}\n", comment).as_str();
            },
            ":" => {
                result = result.trim_end().to_string();
                result += ": \n"
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
            "goto" => {
                result += "goto ";
            },
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                result += identifier.as_str();
            },
            ";" => {
                result = result.trim_end().to_string();
                result += ";";
            },
            _ => println!("You shouldn't be here (goto_statement): {}\n", node.grammar_name()),
        }
    }
    return result;
}

fn handle_while_statement(root: Node, src: String) -> String {
    let mut parts = Vec::<String>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => {
                let compound_statement = handle_compound_statement(node.clone(), src.clone());
                parts.push(compound_statement);
            },
            "while" => {
                parts.push("while".to_string());
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(node, src.clone());
                parts.push(parenthesized_expression);
            },
            "expression_statement" => {
                let expression_statement = handle_expression_statement(node, src.clone());
                parts.push(expression_statement);
            },
            "switch_statement" => {
                let switch_statement = handle_switch_statement(node, src.clone());
                parts.push(switch_statement);
            },
            _ => println!("You shouldn't be here (while_statement): {}\n", node.grammar_name()),
        }
    }
    let result = parts.join(" ");
    return result;
}

fn handle_argument_list(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "," => result += ", ",
            "(" => result += "(",
            ")" => result += ")",
            "null" => result += "NULL",
            "true" => result += "true",
            "false" => result += "false",
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                result += identifier.as_str();
            },
            "call_expression" => {
                let call_expression = handle_call_expression(node, src.clone());
                result += call_expression.as_str();
            },
            "cast_expression" => {
                let cast_expression = handle_cast_expression(node, src.clone());
                result += cast_expression.as_str();
            },
            "field_expression" => {
                let field_expression = handle_field_expression(node, src.clone());
                result += field_expression.as_str();
            },
            "binary_expression" => {
                let binary_expression = handle_binary_expression(node, src.clone());
                result += binary_expression.as_str();
            },
            "pointer_expression" => {
                let pointer_expression = handle_pointer_expression(node, src.clone());
                result += pointer_expression.as_str();
            },
            "subscript_expression" => {
                let subscript_expression = handle_subscript_expression(node, src.clone());
                result += subscript_expression.as_str();
            },
            "sizeof_expression" => {
                let sizeof_expression = handle_sizeof_expression(node, src.clone());
                result += sizeof_expression.as_str();
            },
            "update_expression" => {
                let update_expression = handle_update_expression(node, src.clone());
                result += update_expression.as_str();
            },
            "number_literal" => {
                let number_literal = handle_number_literal(node, src.clone());
                result += number_literal.as_str();
            },
            "char_literal" => {
                let char_literal = handle_char_literal(node, src.clone());
                result += char_literal.as_str();
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(node, src.clone());
                result += parenthesized_expression.as_str();
            },
            "conditional_expression" => {
                let conditional_expression = handle_conditional_expression(node, src.clone());
                result += conditional_expression.as_str();
            },
            "unary_expression" => {
                let unary_expression = handle_unary_expression(node, src.clone());
                result += unary_expression.as_str();
            },
            "assignment_expression" => {
                let assignment_expression = handle_assignment_expression(node, src.clone());
                result += assignment_expression.as_str();
            },
            "string_literal" => {
                let string_literal = handle_string_literal(node, src.clone());
                result += string_literal.as_str();
            },
            "concatenated_string" => {
                let concatenated_string = handle_concatenated_string(node, src.clone());
                result += concatenated_string.as_str();
            },
            _ => println!("You shouldn't be here (argument_list): {}\n", node.grammar_name()),
        }
    }
    return result;
}

fn handle_return_statement(root: Node, src: String) -> String {
    let mut result: String;
    let mut parts = Vec::<String>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => {
                let compound_statement = handle_compound_statement(node.clone(), src.clone());
                parts.push(compound_statement);
            },
            "call_expression" => {
                let call_expression = handle_call_expression(node.clone(), src.clone());
                parts.push(call_expression);
            },
            "binary_expression" => {
                let binary_expression = handle_binary_expression(node.clone(), src.clone());
                parts.push(binary_expression);
            },
            "unary_expression" => {
                let unary_expression = handle_unary_expression(node.clone(), src.clone());
                parts.push(unary_expression);
            },
            "identifier" => {
                let identifier = handle_identifier(node.clone(), src.clone());
                parts.push(identifier);
            },
            "conditional_expression" => {
                let conditional_expression = handle_conditional_expression(node.clone(), src.clone());
                parts.push(conditional_expression);
            },
            "field_expression" => {
                let field_expression = handle_field_expression(node.clone(), src.clone());
                parts.push(field_expression);
            },
            "cast_expression" => {
                let cast_expression = handle_cast_expression(node.clone(), src.clone());
                parts.push(cast_expression);
            },
            "number_literal" => {
                let number_literal = handle_number_literal(node.clone(), src.clone());
                parts.push(number_literal);
            },
            "char_literal" => {
                let char_literal = handle_char_literal(node.clone(), src.clone());
                parts.push(char_literal);
            },
            "string_literal" => {
                let string_literal = handle_string_literal(node.clone(), src.clone());
                parts.push(string_literal);
            },
            "subscript_expression" => {
                let subscript_expression = handle_subscript_expression(node.clone(), src.clone());
                parts.push(subscript_expression);
            },
            "pointer_expression" => {
                let pointer_expression = handle_pointer_expression(node.clone(), src.clone());
                parts.push(pointer_expression);
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(node.clone(), src.clone());
                parts.push(parenthesized_expression);
            },
            "return" => {
                parts.push("return".to_string());
            },
            "false" => {
                parts.push("false".to_string());
            },
            "true" => {
                parts.push("true".to_string());
            },
            ";" => {
                parts.push(";".to_string());
            },
            _ => {
                println!("You shouldn't be here (return statement): {}\n", node.grammar_name());
            },
        }
    }
    result = parts.join(" ");
    result = utils::remove_unnecessary_spaces(result);
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
                    "pointer_declarator" => {
                        let pointer_declarator = handle_pointer_declarator(subnode, src.clone());
                        temp += pointer_declarator.as_str();
                    },
                    "parameter_declaration" => {
                        let mut parameter_declaration = handle_parameter_declaration(subnode, src.clone());
                        parameter_declaration = utils::remove_pointer_spaces(parameter_declaration);
                        temp += parameter_declaration.as_str();
                    },
                    "identifier" => {
                        let identifier = handle_identifier(subnode, src.clone());
                        temp += identifier.as_str();
                    },
                    "variadic_parameter" => {
                        let variadic_parameter = handle_variadic_parameter(subnode, src.clone());
                        temp += variadic_parameter.as_str();
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
        match node.kind() {
            "*" => {
                result += "*";
                // Recursively handle nested pointer declarators
                let child_result = handle_pointer_declarator(node.named_child(0).unwrap_or(node), src.clone());
                result += child_result.as_str();
            },
            "pointer_declarator" => {
                // Recursively handle nested pointer declarators
                let child_result = handle_pointer_declarator(node, src.clone());
                result += child_result.as_str();
            },
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                result += format!(" {}", identifier).as_str();
            },
            "function_declarator" => {
                let function_declarator = handle_function_declarator(node, src.clone());
                result += format!(" {}", function_declarator).as_str();
            },
            "type_identifier" => {
                let type_identifier = handle_type_identifier(node, src.clone());
                result += type_identifier.as_str();
            },
            "field_identifier" => {
                let field_identifier = handle_field_identifier(node, src.clone());
                result += field_identifier.as_str();
            },
            "array_declarator" => {
                let array_declarator = handle_array_declarator(node, src.clone());
                result += format!(" {}", array_declarator).as_str();
            },
            _ => println!("You shouldn't be here (pointer_declarator): {}\n", node.kind()),
        }
    }
    return result;
}

fn handle_identifier(root: Node, src: String) -> String {
    let mut result = String::new();
    let mut parts = Vec::<String>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                result = handle_identifier(node, src.clone());
            },
            "expression_statement" => {
                result = handle_expression_statement(root, src.clone());
            },
            ";" => {
                result = ";".to_string();
            },
            _ => println!("You shouldn't be here (identifier): {}\n", root.grammar_name()),
        }
    }
    if result.len() == 0 {
        result = root.utf8_text(src.as_bytes()).unwrap().to_string();
    }
    return result;
}

fn handle_nested_inner_compound_statement(root: Node, src: String) -> String {
    let mut parts = Vec::<String>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "expression_statement" => {
                let expression_statement = handle_expression_statement(node, src.clone());
                parts.push(expression_statement);
            }
            "if_statement" => {
                let if_statement = handle_if_statement(node, src.clone());
                parts.push(if_statement);
            }
            "{" => parts.push("{".to_string()),
            "}" => parts.push("}".to_string()),
            _ => println!("You shouldn't be here (nested_inner_compound_statement): {}\n", node.grammar_name()),
        }
    }
    let result = parts.join("\n");
    return result;
}

fn handle_binary_expression(root: Node, src: String) -> String {
    let mut parts = Vec::<String>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                parts.push(identifier);
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(node, src.clone());
                parts.push(parenthesized_expression);
            },
            "field_expression" => {
                let field_expression = handle_field_expression(node, src.clone());
                parts.push(field_expression);
            },
            "number_literal" => {
                let number_literal = handle_number_literal(node, src.clone());
                parts.push(number_literal);
            },
            "sizeof_expression" => {
                let sizeof_expression = handle_sizeof_expression(node, src.clone());
                parts.push(sizeof_expression);
            },
            "subscript_expression" => {
                let subscript_expression = handle_subscript_expression(node, src.clone());
                parts.push(subscript_expression);
            },
            "call_expression" => {
                let call_expression = handle_call_expression(node, src.clone());
                parts.push(call_expression);
            },
            "ERROR" => {
                let error = handle_error(node, src.clone());
                parts.push(error);
            },
            "update_expression" => {
                let update_expression = handle_update_expression(node, src.clone());
                parts.push(update_expression);
            },
            "char_literal" => {
                let char_literal = handle_char_literal(node, src.clone());
                parts.push(char_literal);
            },
            "binary_expression" => {
                let binary_expression = handle_inner_binary_expression(node, src.clone());
                parts.push(binary_expression);
            },
            "pointer_expression" => {
                let pointer_expression = handle_pointer_expression(node, src.clone());
                parts.push(pointer_expression);
            },
            "cast_expression" => {
                let cast_expression = handle_cast_expression(node, src.clone());
                parts.push(cast_expression);
            },
            "unary_expression" => {
                let unary_expression = handle_unary_expression(node, src.clone());
                parts.push(unary_expression);
            },
            "null" => parts.push("NULL".to_string()),
            "+" => parts.push("+".to_string()),
            "-" => parts.push("-".to_string()),
            "*" => parts.push("*".to_string()),
            "/" => parts.push("/".to_string()),
            "%" => parts.push("%".to_string()),
            ">" => parts.push(">".to_string()),
            "<" => parts.push("<".to_string()),
            ">=" => parts.push(">=".to_string()),
            "<=" => parts.push("<=".to_string()),
            "!=" => parts.push("!=".to_string()),
            "==" => parts.push("==".to_string()),
            ">>" => parts.push(">>".to_string()),
            "<<" => parts.push("<<".to_string()),
            "|" => parts.push("|".to_string()),
            "||" => parts.push("||".to_string()),
            "&" => parts.push("&".to_string()),
            "&&" => parts.push("&&".to_string()),
            "^" => parts.push("^".to_string()),
            ";" => parts.push(";".to_string()),
            _ => println!("You shouldn't be here (binary_expression): {}\n", node.grammar_name()),
        }
    }
    let result = parts.join(" ");
    return result;
}

fn handle_inner_binary_expression(root: Node, src: String) -> String {
    let mut parts = Vec::<String>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                parts.push(identifier);
            },
            "field_expression" => {
                let field_expression = handle_field_expression(node, src.clone());
                parts.push(field_expression);
            },
            "call_expression" => {
                let call_expression = handle_call_expression(node, src.clone());
                parts.push(call_expression);
            },
            "pointer_expression" => {
                let pointer_expression = handle_pointer_expression(node, src.clone());
                parts.push(pointer_expression);
            },
            "update_expression" => {
                let update_expression = handle_update_expression(node, src.clone());
                parts.push(update_expression);
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(node, src.clone());
                parts.push(parenthesized_expression);
            },
            "sizeof_expression" => {
                let sizeof_expression = handle_sizeof_expression(node, src.clone());
                parts.push(sizeof_expression);
            },
            "number_literal" => {
                let number_literal = handle_number_literal(node, src.clone());
                parts.push(number_literal);
            },
            "char_literal" => {
                let char_literal = handle_char_literal(node, src.clone());
                parts.push(char_literal);
            },
            "binary_expression" => {
                let binary_expression = handle_binary_expression(node, src.clone());
                parts.push(binary_expression);
            },
            "subscript_expression" => {
                let subscript_expression = handle_subscript_expression(node, src.clone());
                parts.push(subscript_expression);
            },
            "+" => parts.push("+".to_string()),
            "-" => parts.push("-".to_string()),
            "*" => parts.push("*".to_string()),
            "/" => parts.push("/".to_string()),
            "%" => parts.push("%".to_string()),
            ">" => parts.push(">".to_string()),
            "<" => parts.push("<".to_string()),
            ">=" => parts.push(">=".to_string()),
            "<=" => parts.push("<=".to_string()),
            "!=" => parts.push("!=".to_string()),
            "==" => parts.push("==".to_string()),
            ">>" => parts.push(">>".to_string()),
            "<<" => parts.push("<<".to_string()),
            "|" => parts.push("|".to_string()),
            "||" => parts.push("||".to_string()),
            "&" => parts.push("&".to_string()),
            "&&" => parts.push("&&".to_string()),
            "^" => parts.push("^".to_string()),
            ";" => parts.push(";".to_string()),
            _ => println!("You shouldn't be here (inner_binary_expression): {}\n", node.grammar_name()),
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
            "field_expression" => {
                let field_expression = handle_field_expression(node, src.clone());
                result += field_expression.as_str();
            },
            "unary_expression" => {
                let unary_expression = handle_unary_expression(node, src.clone());
                result += unary_expression.as_str();
            },
            "conditional_expression" => {
                let conditional_expression = handle_conditional_expression(node, src.clone());
                result += conditional_expression.as_str();
            },
            "concatenated_string" => {
                let concatenated_string = handle_concatenated_string(node, src.clone());
                result += concatenated_string.as_str();
            },
            "char_literal" => {
                let char_literal = handle_char_literal(node, src.clone());
                result += char_literal.as_str();
            },
            "number_literal" => {
                let number_literal = handle_number_literal(node, src.clone());
                result += number_literal.as_str();
            },
            "update_expression" => {
                let update_expression = handle_update_expression(node, src.clone());
                result += update_expression.as_str();
            },
            "(" => result += "(",
            ")" => result += ")",
            _ => println!("You shouldn't be here (parenthesized_expression): {}\n", node.grammar_name()),
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
            },
            "pointer_expression" => {
                let pointer_expression = handle_inner_pointer_expression(node, src.clone());
                result += pointer_expression.as_str();
            },
            "subscript_expression" => {
                let subscript_expression = handle_subscript_expression(node, src.clone());
                result += subscript_expression.as_str();
            },
            "update_expression" => {
                let update_expression = handle_update_expression(node, src.clone());
                result += update_expression.as_str();
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(node, src.clone());
                result += parenthesized_expression.as_str();
            },
            "field_expression" => {
                let field_expression = handle_field_expression(node, src.clone());
                result += field_expression.as_str();
            },
            "cast_expression" => {
                let cast_expression = handle_cast_expression(node, src.clone());
                result += cast_expression.as_str();
            },
            "*" => result += "* ",
            "&" => result += "&",
            ";" => result += ";",
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
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                result += identifier.as_str();
            },
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
            "abstract_array_declarator" => {
                let abstract_array_declarator = handle_abstract_array_declarator(node, src.clone());
                parts.push(abstract_array_declarator);
            },
            "sized_type_specifier" => {
                let sized_type_specifier = handle_sized_type_specifier(node, src.clone());
                parts.push(sized_type_specifier);
            },
            "array_declarator" => {
                let array_declarator = handle_array_declarator(node, src.clone());
                parts.push(array_declarator);
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
            },
            "update_expression" => {
                let update_expression = handle_update_expression(node, src.clone());
                result += update_expression.as_str();
            },
            "binary_expression" => {
                let binary_expression = handle_binary_expression(node, src.clone());
                result += binary_expression.as_str();
            },
            "number_literal" => {
                let number_literal = handle_number_literal(node, src.clone());
                result += number_literal.as_str();
            },
            "field_expression" => {
                let field_expression = handle_field_expression(node, src.clone());
                result += field_expression.as_str();
            },
            "call_expression" => {
                let call_expression = handle_call_expression(node, src.clone());
                result += call_expression.as_str();
            },
            "cast_expression" => {
                let cast_expression = handle_cast_expression(node, src.clone());
                result += cast_expression.as_str();
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(node, src.clone());
                result += parenthesized_expression.as_str();
            },
            "subscript_expression" => {
                let subscript_expression = handle_subscript_expression(node, src.clone());
                result += subscript_expression.as_str();
            },
            "pointer_expression" => {
                let pointer_expression = handle_pointer_expression(node, src.clone());
                result += pointer_expression.as_str();
            },
            _ => println!("You shouldn't be here (subscript_expression): {}\n", node.grammar_name()),
        }
    }
    result = utils::remove_all_spaces(result);
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
            "subscript_expression" => {
                let subscript_expression = handle_subscript_expression(node, src.clone());
                result += subscript_expression.as_str();
            },
            "call_expression" => {
                let call_expression = handle_call_expression(node, src.clone());
                result += call_expression.as_str();
            },
            "field_expression" => {
                let field_expression = handle_inner_field_expression(node, src.clone());
                result += field_expression.as_str();
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
            "binary_expression" => {
                result = handle_binary_expression(node, src.clone());
            },
            "ERROR" => {
                result = node.utf8_text(src.as_bytes()).unwrap().to_string();
            },
            "{" => result = "{".to_string(),
            "}" => result = "}".to_string(),
            ";" => result = ";".to_string(),
            "=" => result = "=".to_string(),
            _ => println!("You shouldn't be here (ERROR): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap()),
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
            "null" => result += "NULL",
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
            "field_expression" => {
                let field_expression = handle_field_expression(node, src.clone());
                result += field_expression.as_str();
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(node, src.clone());
                result += parenthesized_expression.as_str();
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
                parts.push(initializer_list);
            },
            "unary_expression" => {
                let unary_expression = handle_unary_expression(node, src.clone());
                parts.push(unary_expression);
            },
            "conditional_expression" => {
                let conditional_expression = handle_conditional_expression(node, src.clone());
                parts.push(conditional_expression);
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(node, src.clone());
                parts.push(parenthesized_expression);
            },
            "array_declarator" => {
                let array_declarator = handle_array_declarator(node, src.clone());
                parts.push(array_declarator);
            },
            "=" => parts.push("=".to_string()),
            "false" => parts.push("false".to_string()),
            "true" => parts.push("true".to_string()),
            "null" => parts.push("NULL".to_string()),
            _ => println!("You shouldn't be here (init_declarator): {}\n", node.grammar_name()),
        }
    }
    let mut result = parts.join(" ");
    return result;
}

fn handle_type_descriptor(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                result += identifier.as_str();
            },
            "primitive_type" => {
                let primitive_type = handle_primitive_type(node, src.clone());
                result += primitive_type.as_str();
            },
            "abstract_pointer_declarator" => {
                let abstract_pointer_declaration = handle_abstract_pointer_declarator(node, src.clone());
                result += format!("{} ", abstract_pointer_declaration).as_str();
            },
            "struct_specifier" => {
                let struct_specifier = handle_struct_specifier(node, src.clone());
                result += struct_specifier.as_str();
            },
            "sized_type_specifier" => {
                let sized_type_specifier = handle_sized_type_specifier(node, src.clone());
                result += sized_type_specifier.as_str();
            },
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
                result += number_literal.as_str();
            },
            "char_literal" => {
                let char_literal = handle_char_literal(node, src.clone());
                result += char_literal.as_str();
            },
            "comment" => {
                let comment = handle_comment(node, src.clone());
                result += comment.as_str();
            },
            "initializer_list" => {
                let initializer_list = handle_initializer_list(node, src.clone());
                result += initializer_list.as_str();
            },
            "string_literal" => {
                let string_literal = handle_string_literal(node, src.clone());
                result += string_literal.as_str();
            },
            "{" => result += "{",
            "}" => result += "}",
            "," => result += ", ",
            _ => println!("You shouldn't be here (initializer_list): {}\n", node.grammar_name()),
        }
    }
    return result;
}

fn handle_sizeof_expression(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "sizeof" => result += "sizeof",
            "(" => result += "(",
            ")" => result += ")",
            "parenthesized_expression" => {
                let mut parenthesized_expression = handle_parenthesized_expression(node, src.clone());
                parenthesized_expression = utils::remove_all_spaces(parenthesized_expression);
                result += parenthesized_expression.as_str();
            },
            "type_descriptor" => {
                let type_descriptor = handle_type_descriptor(node, src.clone());
                result += type_descriptor.as_str();
            },
            _ => println!("You shouldn't be here (sizeof_expression): {}\n", node.grammar_name()),
        }
    }
    return result;
}

fn handle_type_qualifier(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "const" => {
                result = "const".to_string();
            },
            _ => println!("You shouldn't be here (type_qualifier): {}\n", node.grammar_name()),
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

fn handle_conditional_expression(root: Node, src: String) -> String {
    let mut parts = Vec::<String>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                parts.push(identifier);
            },
            "call_expression" => {
                let call_expression = handle_call_expression(node, src.clone());
                parts.push(call_expression);
            },
            "field_expression" => {
                let field_expression = handle_field_expression(node, src.clone());
                parts.push(field_expression);
            },
            "binary_expression" => {
                let binary_expression = handle_binary_expression(node, src.clone());
                parts.push(binary_expression);
            },
            "unary_expression" => {
                let unary_expression = handle_unary_expression(node, src.clone());
                parts.push(unary_expression);
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(node, src.clone());
                parts.push(parenthesized_expression);
            },
            "comment" => {
                let comment = handle_comment(node, src.clone());
                parts.push(comment);
            },
            "subscript_expression" => {
                let subscript_expression = handle_subscript_expression(node, src.clone());
                parts.push(subscript_expression);
            },
            "number_literal" => {
                let number_literal = handle_number_literal(node, src.clone());
                parts.push(number_literal);
            },
            "char_literal" => {
                let char_literal = handle_char_literal(node, src.clone());
                parts.push(char_literal);
            },
            ":" => parts.push(":".to_string()),
            "?" => parts.push("?".to_string()),
            _ => println!("You shouldn't be here (conditional_expression): {}\n", node.grammar_name()),
        }
    }
    let result = parts.join(" ");
    return result;
}

fn handle_char_literal(root: Node, src: String) -> String {
    let result = root.utf8_text(src.as_bytes()).unwrap().to_string();
    return result;
}

fn handle_number_literal(root: Node, src: String) -> String {
    let result = root.utf8_text(src.as_bytes()).unwrap().to_string();
    return result;
}

fn handle_unary_expression(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                result += identifier.as_str();
            },
            "call_expression" => {
                let call_expression = handle_call_expression(node, src.clone());
                result += call_expression.as_str();
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(node, src.clone());
                result += parenthesized_expression.as_str();
            },
            "field_expression" => {
                let field_expression = handle_field_expression(node, src.clone());
                result += field_expression.as_str();
            },
            "pointer_expression" => {
                let pointer_expression = handle_pointer_expression(node, src.clone());
                result += pointer_expression.as_str();
            },
            "number_literal" => {
                let number_literal = handle_number_literal(node, src.clone());
                result += number_literal.as_str();
            },
            "!" => result += "!",
            "~" => result += "~",
            "-" => result += "-",
            _ => println!("You shouldn't be here (unary_expression): {}\n", node.grammar_name()),
        }
    }
    return result;
}

fn handle_array_declarator(root: Node, src: String) -> String {
    let mut parts = Vec::<String>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "number_literal" => {
                let number_literal = handle_number_literal(node, src.clone());
                parts.push(number_literal);
            },
            "char_literal" => {
                let char_literal = handle_char_literal(node, src.clone());
                parts.push(char_literal);
            },
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                parts.push(identifier);
            },
            "binary_expression" => {
                let binary_expression = handle_binary_expression(node, src.clone());
                parts.push(binary_expression);
            },
            "array_declarator" => {
                let array_declarator = handle_array_declarator(node, src.clone());
                parts.push(array_declarator);
            },
            "[" => parts.push("[".to_string()),
            "]" => parts.push("]".to_string()),
            _ => println!("You shouldn't be here (array_declarator): {}\n", node.grammar_name()),
        }
    }
    let result = parts.join("");
    return result;
}

fn handle_string_literal(root: Node, src: String) -> String {
    let mut parts = Vec::<String>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "string_content" => {
                parts.push(node.utf8_text(src.as_bytes()).unwrap().to_string());
            },
            "escape_sequence" => {
                parts.push(node.utf8_text(src.as_bytes()).unwrap().to_string());
            },
            "\"" => parts.push("\"".to_string()),
            _ => println!("You shouldn't be here (string_literal): {}\n", node.grammar_name()),
        }
    }
    let result = parts.join("");
    return result;
}

fn handle_concatenated_string(root: Node, src: String) -> String {
    let mut parts = Vec::<String>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "string_literal" => {
                let string_literal = handle_string_literal(node, src.clone());
                parts.push(string_literal);
            }
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                parts.push(identifier);
            }
            _ => println!("You shouldn't be here (concatenated_string): {}\n", node.grammar_name()),
        }
    }
    let result = parts.join(" ");
    return result;
}

fn handle_labeled_statement(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "expression_statement" => {
                let expression_statement = handle_expression_statement(node, src.clone());
            },
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                result += identifier.as_str();
            },
            "return_statement" => {
                let return_statement = handle_return_statement(node, src.clone());
                result += return_statement.as_str();
            },
            "if_statement" => {
                let if_statement = handle_if_statement(node, src.clone());
                result += if_statement.as_str();
            },
            ":" => result += ": ",
            _ => println!("You shouldn't be here (labeled_statement): {}\n", node.grammar_name()),
        }
    }
    return result;
}

fn handle_for_statement(root: Node, src: String) -> String {
    let mut vec = Vec::<String>::new();
    let mut temp = String::new();
    let mut reached_compound = false;
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => {
                reached_compound = true;
                let compound_statement = handle_compound_statement(node, src.clone());
                vec.push(compound_statement);
            },
            "if_statement" => {
                let if_statement = handle_if_statement(node, src.clone());
                vec.push(if_statement);
            },
            "update_expression" => {
                let update_expression = handle_update_expression(node, src.clone());
                if !reached_compound {
                    temp += update_expression.as_str();
                }
                else {
                    vec.push(update_expression);
                }
            },
            "assignment_expression" => {
                let assignment_expression = handle_assignment_expression(node, src.clone());
                vec.push(assignment_expression);
            },
            "binary_expression" => {
                let binary_expression = handle_binary_expression(node, src.clone());
                if !reached_compound {
                    temp += format!(" {}", binary_expression).as_str();
                }
                else {
                    vec.push(binary_expression);
                }
            },
            "declaration" => {
                let declaration = handle_declaration(node, src.clone());
                if !reached_compound {
                    temp += declaration.as_str();
                }
                else {
                    vec.push(declaration);
                }
            },
            "expression_statement" => {
                let expression_statement = handle_expression_statement(node, src.clone());
                vec.push(expression_statement);
            },
            ";" => temp += "; ",
            "(" => temp += "(",
            ")" => {
                temp += ")";
                vec.push(temp);
                temp = "".to_string();
            },
            "for" => temp += "for ",
            _ => println!("You shouldn't be here (for_statement): {}\n", node.grammar_name()),
        }
    }
    let result = vec.join(" ");
    return result;
}

fn handle_struct_specifier(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                result += format!("{} ", identifier).as_str();
            },
            "field_declaration_list" => {
                let field_declaration_list = handle_field_declaration_list(node, src.clone());
                // This adds a newline before the curly brace.
                if result.contains("\n") {
                    result += field_declaration_list.as_str();
                }
                else {
                    result += format!("\n{}", field_declaration_list).as_str();
                }
            },
            "struct" => result += "struct ",
            _ => println!("You shouldn't be here (struct_specifier): {}\n", node.grammar_name()),
        }
    }
    return result;
}

fn handle_type_definition(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                // This is just the name of the typedef.
                let identifier = handle_identifier(node, src.clone());
                result += format!(" {}", identifier).as_str();
            },
            "typedef" => {
                result += "typedef ";
            },
            "primitive_type" => {
                let primitive_type = handle_primitive_type(node, src.clone());
                result += primitive_type.as_str();
            },
            "function_declarator" => {
                let function_declarator = handle_function_declarator(node, src.clone());
                result += function_declarator.as_str();
            },
            "struct_specifier" => {
                let struct_specifier = handle_struct_specifier(node, src.clone());
                result += struct_specifier.as_str();
            },
            "union_specifier" => {
                let union_specifier = handle_union_specifier(node, src.clone());
                result += union_specifier.as_str();
            },
            "pointer_declarator" => {
                let pointer_declarator = handle_pointer_declarator(node, src.clone());
                result = result.trim_end().to_string();
                result += format!(" {}", pointer_declarator).as_str();
            },
            "enum_specifier" => {
                let enum_specifier = handle_enum_specifier(node, src.clone());
                result += enum_specifier.as_str();
            },
            ";" => result += ";",
            _ => println!("You shouldn't be here (type_definition): {}\n", node.grammar_name()),
        }
    }
    return result;
}

fn handle_field_declaration_list(root: Node, src: String) -> String {
    let mut parts = Vec::<String>::new();
    let mut temp = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "field_declaration" => {
                let mut field_declaration = handle_field_declaration(node, src.clone());
                if field_declaration.contains("*") { 
                    field_declaration = utils::switch_pointer_spaces(field_declaration); 
                }
                if temp.len() > 0 {
                    parts.push(temp);
                    temp = String::new();
                }
                else {
                    temp += format!("\t{}", field_declaration).as_str();
                }
            },
            "comment" => {
                let comment = handle_comment(node, src.clone());
                temp += format!("\t{}", comment).as_str();
                parts.push(temp);
                temp = String::new();
            },
            "{" => parts.push("{".to_string()),
            "}" => parts.push("}".to_string()),
            _ => println!("You shouldn't be here (field_declaration_list): {}\n", node.grammar_name()),
        }
    }
    if temp.len() > 0 {
        parts.remove(parts.len()-1);
        parts.push(temp);
        parts.push("}".to_string());
    }
    let result = parts.join("\n");
    return result;
}

fn handle_field_declaration(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                result += format!("{} ", identifier).as_str();
            },
            "primitive_type" => {
                let primitive_type = handle_primitive_type(node, src.clone());
                result += format!("{} ", primitive_type).as_str();
            },
            "array_declarator" => {
                let array_declarator = handle_array_declarator(node, src.clone());
                result += format!("{} ", array_declarator).as_str();
            },
            "pointer_declarator" => {
                let pointer_declarator = handle_pointer_declarator(node, src.clone());
                result += format!("{} ", pointer_declarator).as_str();
            },
            ";" => result += ";",
            "," => result += ", ",
            _ => println!("You shouldn't be here (field_declaration): {}\n", node.grammar_name()),
        }
    }
    result = utils::remove_unnecessary_spaces(result);
    return result;
}

fn handle_preproc_def(root: Node, src: String) -> String {
    let mut parts = Vec::<String>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "preproc_arg" => {
                parts.push(node.utf8_text(src.as_bytes()).unwrap().to_string());
            },
            "identifier" => {
                parts.push(node.utf8_text(src.as_bytes()).unwrap().to_string());
            },
            "#define" => parts.push("#define".to_string()),
            _ => println!("You shouldn't be here (preproc_def): {}\n", node.grammar_name()),
        }
    }
    let result = parts.join(" ");
    return result;
}

fn handle_abstract_array_declarator(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "number_literal" => {
                let number_literal = handle_number_literal(node, src.clone());
                result += number_literal.as_str();
            },
            "[" => result += "[",
            "]" => result += "]",
            _ => println!("You shouldn't be here (abstract_array_declarator): {}\n", node.grammar_name()),
        }
    }
    return result;
}

fn handle_sized_type_specifier(root: Node, src: String) -> String {
    let mut parts = Vec::<String>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "primitive_type" => {
                let primitive_type = handle_primitive_type(node, src.clone());
                parts.push(primitive_type);
            },
            "long" => parts.push("long".to_string()),
            "unsigned" => parts.push("unsigned".to_string()),
            _ => println!("You shouldn't be here (sized_type_specifier): {}\n", node.grammar_name()),
        }
    }
    let result = parts.join(" ");
    return result;
}

fn handle_enum_specifier(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                result += format!("{} ", identifier).as_str();
            },
            "enumerator_list" => {
                let enumerator_list = handle_enumerator_list(node, src.clone());
                result += format!("{} ", enumerator_list).as_str();
            },
            "field_declaration_list" => {
                let field_declaration_list = handle_field_declaration_list(node, src.clone());
                // This adds a newline before the curly brace.
                if result.contains("\n") {
                    result += field_declaration_list.as_str();
                }
                else {
                    result += format!("\n{}", field_declaration_list).as_str();
                }
            },
            "enum" => result += "enum ",
            _ => println!("You shouldn't be here (enum_specifier): {}\n", node.grammar_name()),
        }
    }
    return result;
}

fn handle_union_specifier(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                result += format!("{} ", identifier).as_str();
            },
            "field_declaration_list" => {
                let field_declaration_list = handle_field_declaration_list(node, src.clone());
                // This adds a newline before the curly brace.
                if result.contains("\n") {
                    result += field_declaration_list.as_str();
                }
                else {
                    result += format!("\n{}", field_declaration_list).as_str();
                }
            },
            "union" => result += "union ",
            _ => println!("You shouldn't be here (union_specifier): {}\n", node.grammar_name()),
        }
    }
    return result;
}

fn handle_enumerator_list(root: Node, src: String) -> String {
    let mut parts = Vec::<String>::new();
    let mut temp = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "enumerator" => {
                temp += format!("\t{}", node.utf8_text(src.as_bytes()).unwrap()).as_str();
            },
            "," => {
                temp += ",";
                parts.push(temp);
                temp = String::new();
            },
            "{" => parts.push("{".to_string()),
            "}" => parts.push("}".to_string()),
            _ => println!("You shouldn't be here (enumerator_list): {}\n", node.grammar_name()),
        }
    }
    if temp.len() > 0 {
        parts.remove(parts.len()-1);
        parts.push(temp);
        parts.push("}".to_string());
    }
    let result = parts.join("\n");
    return result;
}

fn handle_storage_class_specifier(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "static" => result += "static",
            _ => println!("You shouldn't be here (storage_class_specifier): {}\n", node.grammar_name()),
        }
    }
    return result;
}


fn handle_preproc_ifdef(root: Node, src: String) -> String {
    let mut parts = Vec::<String>::new();
    let mut temp = String::new();
    let mut last_kind = "";
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "#ifndef" => {
                temp += "#ifndef ";
                last_kind = "#ifndef";
            },
            "#ifdef" => {
                temp += "#ifdef ";
                last_kind = "#ifdef";
            },
            "#endif" => {
                parts.push("\n#endif".to_string());
                last_kind = "#endif";
            },
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                temp += identifier.as_str();
                parts.push(temp);
                temp = String::new();
                last_kind = "identifier";
            },
            "preproc_def" => {
                let preproc_def = handle_preproc_def(node, src.clone()) + "\n";
                parts.push(preproc_def);
                last_kind = "preproc_def";
            },
            "preproc_include" => {
                let preproc_include = handle_preproc_include(node, src.clone());
                parts.push(preproc_include);
                last_kind = "preproc_include";
            },
            "declaration" => {
                let mut declaration = handle_declaration(node, src.clone());
                if declaration.contains("*") {
                    declaration = utils::remove_pointer_spaces(declaration);
                }
                if last_kind != "declaration" { declaration = format!("\n{}", declaration); }
                parts.push(declaration);
                last_kind = "declaration";
            },
            _ => println!("You shouldn't be here (preproc_ifdef): {}\n", node.grammar_name()),
        }
    }
    let result = parts.join("\n");
    return result;
}

fn handle_preproc_if(root: Node, src: String) -> String {
    let mut parts = Vec::<String>::new();
    let mut temp = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "#if" => {
                temp += "#if ";
            },
            "#endif" => {
                parts.push("#endif".to_string());
            },
            "binary_expression" => {
                let binary_expression = handle_binary_expression(node, src.clone());
                temp += binary_expression.as_str();
                parts.push(temp);
                temp = String::new();
            },
            "return_statement" => {
                let return_statement = handle_return_statement(node, src.clone());
                parts.push(return_statement);
            },
            "function_definition" => {
                let function_definition = handle_function_definition(node, src.clone());
                parts.push(function_definition);
            },
            "preproc_else" => {
                let preproc_else = handle_preproc_else(node, src.clone());
                parts.push(preproc_else);
            },
            "comment" => {
                let comment = handle_comment(node, src.clone());
                parts.push(comment);
            },
            "number_literal" => {
                let number_literal = handle_number_literal(node, src.clone());
                parts.push(number_literal);
            },
            "\n" => {
                // maybe remove?
                parts.push("\n".to_string());
            },
            _ => println!("You shouldn't be here (preproc_if): {}\n", node.grammar_name()),
        }
    }
    let result = parts.join("\n");
    return result;
}

fn handle_preproc_function_def(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "preproc_directive" => {
                result += format!("{} ", node.utf8_text(src.as_bytes()).unwrap()).as_str();
            },
            "#define" => {
                result += "#define ";
            },
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                result += format!("{} ", identifier).as_str();
            },
            "preproc_params" => {
                let preproc_params = handle_preproc_params(node, src.clone());
                result += format!("{} ", preproc_params).as_str();
            },
            "preproc_arg" => {
                result += node.utf8_text(src.as_bytes()).unwrap();
            },
            _ =>  println!("You shouldn't be here (preproc_function_def): {}\n", node.grammar_name()),
        }
    }
    return result;
}

fn handle_preproc_else(root: Node, src: String) -> String {
    let mut parts = Vec::<String>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "#else" => {
                parts.push("#else".to_string());
            },
            "return_statement" => {
                let return_statement = handle_return_statement(node, src.clone());
                parts.push(return_statement);
            },
            _ => println!("You shouldn't be here (preproc_else): {}\n", node.grammar_name()),
        }
    }
    let result = parts.join("\n");
    return result;
}

fn handle_variadic_parameter(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "..." => result += "...",
            _ => println!("You shouldn't be here (variadic_parameter): {}\n", node.grammar_name()),
        }
    }
    return result;
}

fn handle_preproc_params(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                result += identifier.as_str();
            },
            "(" => result += "(",
            ")" => result += ")",
            "," => result += ", ",
            _ => println!("You shouldn't be here (preproc_params): {}\n", node.grammar_name()),
        }
    }
    return result;
}

fn handle_preproc_call(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "preproc_directive" => {
                result += format!("{} ", node.utf8_text(src.as_bytes()).unwrap()).as_str();
            },
            "preproc_arg" => {
                result += node.utf8_text(src.as_bytes()).unwrap();
            },
            _ => println!("You shouldn't be here (preproc_call): {}\n", node.grammar_name()),
        }
    }
    return result;
}

fn handle_do_statement(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "do" => result += "do ",
            "while" => result += "while ",
            ";" => result += ";",
            "compound_statement" => {
                let compound_statement = handle_compound_statement(node, src.clone());
                result += compound_statement.as_str();
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(node, src.clone());
                result += parenthesized_expression.as_str();
            },
            "expression_statement" => {
                let expression_statement = handle_expression_statement(node, src.clone());
                result += expression_statement.as_str();
            },
            _ => println!("You shouldn't be here (do_statement): {}\n", node.grammar_name()),
        }
    }
    return result;
}

fn handle_inner_field_expression(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "->" => result += "->",
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                result += identifier.as_str();
            },
            "field_expression" => {
                let field_expression = handle_field_expression(node, src.clone());
                result += field_expression.as_str();
            },
            "call_expression" => {
                let call_expression = handle_call_expression(node, src.clone());
                result += call_expression.as_str();
            },
            _ => println!("You shouldn't be here (inner_field_expression): {}\n", node.grammar_name()),
        }
    }
    return result;
}

fn handle_parameter_list(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            _ => println!("Parameter list: {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_type_identifier(root: Node, src: String) -> String {
    return root.utf8_text(src.as_bytes()).unwrap().to_string();
}

fn handle_field_identifier(root: Node, src: String) -> String {
    return root.utf8_text(src.as_bytes()).unwrap().to_string();
}
