use tree_sitter::{Tree, Node};
use crate::{cpp_format, utils};

pub fn traverse_cpp_ast(ast: Tree, src: String, style: utils::Style) -> String {
    let root = ast.root_node();
    let mut result = String::new();
    let mut last_group_kind = String::new();
    let lines_before_blank_lines = utils::scan_for_lines_before_blank_lines(src.clone());
    for child in root.children(&mut root.walk()) {
        match child.grammar_name() {
            "preproc_include" => {
                if !last_group_kind.contains("preproc") { result += "\n"; }
                let preproc_include = handle_preproc_include(child, src.clone());
                result += format!("{}\n", preproc_include).as_str();
                last_group_kind = "preproc_include".to_string();
            },
            "declaration" => {
                if last_group_kind.contains("preproc") { result += "\n"; }
                let declaration = handle_declaration(child, src.clone());
                result += format!("{}\n", declaration).as_str();
                last_group_kind = "declaration".to_string();
            },
            "function_definition" => {
                let should_add_space = {
                    last_group_kind.contains("preproc")    ||
                    last_group_kind == "using_declaration" ||
                    last_group_kind == "declaration"
                };
                if should_add_space { result += "\n"; }
                let function_definition = handle_function_definition(child, src.clone());
                result += format!("{}\n\n", function_definition).as_str();
                last_group_kind = "function_definition".to_string();
            },
            "expression_statement" => {
                let expression_statement = handle_expression_statement(child, src.clone());
                result += format!("{}\n\n", expression_statement).as_str();
                last_group_kind = "expression_statement".to_string();
            },
            "compound_statement" => {
                let compound_statement = handle_compound_statement(child, src.clone());
                result += format!("{}\n\n", compound_statement).as_str();
                last_group_kind = "compound_statement".to_string();
            },
            "if_statement" => {
                let if_statement = handle_if_statement(child, src.clone());
                result += format!("{}\n\n", if_statement).as_str();
                last_group_kind = "if_statement".to_string();
            },
            "switch_statement" => {
                let switch_statement = handle_switch_statement(child, src.clone());
                result += format!("{}\n\n", switch_statement).as_str();
                last_group_kind = "switch_statement".to_string();
            },
            "continue_statement" => {
                let continue_statement = "continue;\n";
                result += continue_statement;
                last_group_kind = "continue_statement".to_string();
            },
            "break_statement" => {
                let break_statement = "break;\n";
                result += break_statement;
                last_group_kind = "break_statement".to_string();
            },
            "return_statement" => {
                let return_statement = handle_return_statement(child, src.clone());
                result += format!("{}\n\n", return_statement).as_str();
                last_group_kind = "return_statement".to_string();
            },
            "comment" => {
                let comment = handle_comment(child, src.clone());
                if last_group_kind.contains("preproc") { result += "\n"; }
                result += format!("{}\n", comment).as_str();
                last_group_kind = "comment".to_string();
            },
            "ERROR" => {
                let error = handle_error(child, src.clone());
                result += format!("{}\n\n", error).as_str();
                last_group_kind = "ERROR".to_string();
            },
            "type_definition" => {
                let type_definition = handle_type_definition(child, src.clone());
                result += format!("{}\n\n", type_definition).as_str();
                last_group_kind = "type_definition".to_string();
            },
            "struct_specifier" => {
                let struct_specifier = handle_struct_specifier(child, src.clone());
                result += format!("{}\n\n", struct_specifier).as_str();
                last_group_kind = "struct_specifier".to_string();
            },
            "preproc_def" => {
                if !last_group_kind.contains("preproc") { result += "\n"; }
                let preproc_def = handle_preproc_def(child, src.clone());
                result += format!("{}\n\n", preproc_def).as_str();
                last_group_kind = "preproc_def".to_string();
            },
            "preproc_ifdef" => {
                if !last_group_kind.contains("preproc") { result += "\n"; }
                let preproc_ifdef = handle_preproc_ifdef(child, src.clone());
                result += format!("{}\n\n", preproc_ifdef).as_str();
                last_group_kind = "preproc_ifdef".to_string();
            },
            "preproc_if" => {
                if !last_group_kind.contains("preproc") { result += "\n"; }
                let preproc_if = handle_preproc_if(child, src.clone());
                result += format!("{}\n\n", preproc_if).as_str();
                last_group_kind = "preproc_if".to_string();
            },
            "preproc_function_def" => {
                if !last_group_kind.contains("preproc") { result += "\n"; }
                let preproc_function_def = handle_preproc_function_def(child, src.clone());
                result += format!("{}\n\n", preproc_function_def).as_str();
                last_group_kind = "preproc_function_def".to_string();
            },
            "preproc_call" => {
                if !last_group_kind.contains("preproc") { result += "\n"; }
                let preproc_call = handle_preproc_call(child, src.clone());
                result += format!("{}\n\n", preproc_call).as_str();
                last_group_kind = "preproc_call".to_string();
            },
            "sized_type_specifier" => {
                let sized_type_specifier = handle_sized_type_specifier(child, src.clone());
                result += format!("{}\n\n", sized_type_specifier).as_str();
                last_group_kind = "sized_type_specifier".to_string();
            },
            "using_declaration" => {
                let using_declaration = handle_using_declaration(child, src.clone());
                if last_group_kind != "using_declaration" { result += "\n"; }
                result += format!("{}\n", using_declaration).as_str();
                last_group_kind = "using_declaration".to_string();
            },
            "enum_specifier" => {
                let enum_specifier = handle_enum_specifier(child, src.clone());
                if last_group_kind.contains("preproc") { result += "\n"; }
                result += format!("{};\n\n", enum_specifier.trim_end()).as_str();
                last_group_kind = "enum_specifier".to_string();
            },
            "class_specifier" => {
                let class_specifier = handle_class_specifier(child, src.clone());
                if last_group_kind.contains("preproc") { result += "\n"; }
                result += format!("{};\n\n", class_specifier.trim_end()).as_str();
                last_group_kind = "class_specifier".to_string();
            },
            "template_declaration" => {
                let template_declaration = handle_template_declaration(child, src.clone());
                result += format!("{}\n\n", template_declaration).as_str();
                last_group_kind = "template_declaration".to_string();
            },
            "namespace_alias_definition" => {
                if last_group_kind != "namespace_alias_definition" { result += "\n"; }
                let namespace_alias_definition = handle_namespace_alias_definition(child, src.clone());
                result += format!("{}\n\n", namespace_alias_definition).as_str();
                last_group_kind = "namespace_alias_definition".to_string();
            },
            "namespace_definition" => {
                if last_group_kind != "namespace_definition" { result += "\n"; }
                let namespace_definition = handle_namespace_definition(child, src.clone());
                // the end semicolon is added at an unknown point
                result += format!("{}", namespace_definition).as_str();
                last_group_kind = "namespace_definition".to_string();
            },
            "alias_declaration" => {
                if last_group_kind != "alias_declaration" { result += "\n"; }
                let alias_declaration = handle_alias_declaration(child, src.clone());
                result += format!("{}\n\n", alias_declaration).as_str();
                last_group_kind = "alias_declaration".to_string();
            },
            ";" => (), // handled in functions above
            _ => println!("Unknown grammar name 1: {}\n", &child.grammar_name()),
        }
    }
    result = utils::sort_include_groups(result);
    utils::format_else_lines(&mut result, &style);
    utils::close_empty_curly_brace_blocks(&mut result);
    utils::tidy_up_loose_ends(&mut result, lines_before_blank_lines);
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
            "for_range_loop" => {
                let mut for_range_loop = handle_for_range_loop(node, src.clone());
                for_range_loop = utils::add_all_leading_tabs(for_range_loop);
                result += format!("{}\n", for_range_loop).as_str();
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
            "try_statement" => {
                let mut try_statement = handle_try_statement(node, src.clone());
                try_statement = utils::add_all_leading_tabs(try_statement);
                result += format!("{}\n", try_statement).as_str();
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
            "template_type" => {
                let template_type = handle_template_type(node, src.clone());
                parts.push(template_type);
            },
            "qualified_identifier" => {
                let qualified_identifier = handle_qualified_identifier(node, src.clone());
                parts.push(qualified_identifier);
            },
            "placeholder_type_specifier" => parts.push("auto".to_string()),
            ";" => parts.push(";".to_string()),
            "," => parts.push(",".to_string()),
            _ => println!("You shouldn't be here (declaration): {}\n", node.grammar_name()),
        }
    }
    result = parts.join(" ");
    if result.contains(",") { result = utils::remove_whitespace_before_commas(result); }
    result = utils::remove_unnecessary_spaces(result);
    if result.contains("= \t[") {
        let idx = result.find("= \t[").unwrap();
        result.remove(idx+2);
    }
    if result.contains("**") { result = utils::remove_pointer_spaces(result); }
    if result.contains("::") { result = utils::remove_object_constructor_space(result); }
    return result;
}

fn handle_function_definition(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => {
                let compound_statement = handle_compound_statement(node, src.clone());
                result = result.trim_end().to_string();
                result = utils::remove_reference_spaces(result);
                result += format!("\n{}\n", compound_statement).as_str();
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
            "qualified_identifier" => {
                let qualified_identifier = handle_qualified_identifier(node, src.clone());
                result += format!("{} ", qualified_identifier).as_str();
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
            "template_type" => {
                let template_type = handle_template_type(node, src.clone());
                result += format!("{} ", template_type).as_str();
            },
            "reference_declarator" => {
                let reference_declarator = handle_reference_declarator(node, src.clone());
                result += format!("{} ", reference_declarator).as_str();
            },
            "field_initializer_list" => {
                let field_initializer_list = handle_field_initializer_list(node, src.clone());
                result += format!("{} ", field_initializer_list).as_str();
                result = result.trim_end().to_string();
                result += "\n";
            },
            _ => println!("You shouldn't be here (function_definition): {}\n", node.grammar_name()),
        }
    }
    result = utils::remove_pointer_spaces(result);
    result = utils::remove_reference_spaces(result);
    result = utils::remove_dereference_spaces(result);
    result = utils::ensure_space_after_char(&result, '=');
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
            "gnu_asm_expression" => {
                result = handle_gnu_asm_expression(node, src.clone());
            },
            "lambda_expression" => {
                result = handle_lambda_expression(node, src.clone());
                let head = result.split("\n").collect::<Vec<&str>>()[0].to_string();
                let mut temp = result.split("\n").collect::<Vec<&str>>()[1..].join("\n");
                temp = utils::add_all_leading_tabs(temp);
                result = format!("{}\n{}", head, temp);
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
            "string_literal" => {
                let string_literal = handle_string_literal(node, src.clone());
                parts.push(string_literal);
            },
            "binary_expression" => {
                let binary_expression = handle_binary_expression(node, src.clone());
                parts.push(binary_expression);
            },
            "conditional_expression" => {
                let conditional_expression = handle_conditional_expression(node, src.clone());
                parts.push(conditional_expression);
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
            "string_literal" => {
                let string_literal = handle_string_literal(node, src.clone());
                parts.push(string_literal);
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
            "for_range_loop" => {
                let mut for_range_loop = handle_for_range_loop(node, src.clone());
                for_range_loop = utils::add_all_leading_tabs(for_range_loop);
                parts.push(for_range_loop);
            },
            "labeled_statement" => {
                let labeled_statement = handle_labeled_statement(node, src.clone());
                parts.push(labeled_statement);
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
                        "qualified_identifier" => {
                            let qualified_identifier = handle_qualified_identifier(subnode, src.clone());
                            temp += qualified_identifier.as_str();
                        },
                        "argument_list" => {
                            let argument_list = handle_argument_list(subnode, src.clone());
                            temp += argument_list.as_str();
                        },
                        "parenthesized_expression" => {
                            let parenthesized_expression = handle_parenthesized_expression(subnode, src.clone());
                            temp += parenthesized_expression.as_str();
                        },
                        "field_expression" => {
                            let field_expression = handle_field_expression(subnode, src.clone());
                            temp += field_expression.as_str();
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
            "qualified_identifier" => {
                let qualified_identifier = handle_qualified_identifier(node, src.clone());
                result += qualified_identifier.as_str();
            },
            "argument_list" => {
                let argument_list = handle_argument_list(node, src.clone());
                result += argument_list.as_str();
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(node, src.clone());
                result += parenthesized_expression.as_str();
            },
            "field_expression" => {
                let field_expression = handle_field_expression(node, src.clone());
                result += field_expression.as_str();
            },
            "template_function" => {
                let template_function = handle_template_function(node, src.clone());
                result += template_function.as_str();
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
            "condition_clause" => {
                let condition_clause = handle_condition_clause(node, src.clone());
                parts.push(condition_clause);
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

fn handle_else_clause(root: Node, src: String) -> String {
    let mut pieces = Vec::<String>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => {
                let inner_compound_statement = handle_inner_compound_statement(node, src.clone());
                pieces.push(inner_compound_statement);
            }
            "else" => {
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
            "condition_clause" => {
                let condition_clause = handle_condition_clause(node, src.clone());
                result += format!("{} ", condition_clause).as_str();
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
            "qualified_identifier" => {
                let qualified_identifier = handle_qualified_identifier(node, src.clone());
                result += qualified_identifier.as_str();
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
            "condition_clause" => {
                let condition_clause = handle_condition_clause(node, src.clone());
                parts.push(condition_clause);
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
            "qualified_identifier" => {
                let qualified_identifier = handle_qualified_identifier(node, src.clone());
                result += qualified_identifier.as_str();
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
            "raw_string_literal" => {
                let raw_string_literal = handle_raw_string_literal(node, src.clone());
                result += raw_string_literal.as_str();
            },
            "initializer_list" => {
                let initializer_list = handle_initializer_list(node, src.clone());
                result += initializer_list.as_str();
            },
            "new_expression" => {
                let new_expression = handle_new_expression(node, src.clone());
                result += new_expression.as_str();
            },
            "lambda_expression" => {
                let mut lambda_expression = handle_lambda_expression(node, src.clone());
                lambda_expression = utils::add_all_leading_tabs(lambda_expression).trim_start().to_string();
                result += lambda_expression.as_str();
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
                let compound_statement = handle_compound_statement(node, src.clone());
                parts.push(compound_statement);
            },
            "call_expression" => {
                let call_expression = handle_call_expression(node, src.clone());
                parts.push(call_expression);
            },
            "binary_expression" => {
                let binary_expression = handle_binary_expression(node, src.clone());
                parts.push(binary_expression);
            },
            "unary_expression" => {
                let unary_expression = handle_unary_expression(node, src.clone());
                parts.push(unary_expression);
            },
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                parts.push(identifier);
            },
            "conditional_expression" => {
                let conditional_expression = handle_conditional_expression(node, src.clone());
                parts.push(conditional_expression);
            },
            "field_expression" => {
                let field_expression = handle_field_expression(node, src.clone());
                parts.push(field_expression);
            },
            "cast_expression" => {
                let cast_expression = handle_cast_expression(node, src.clone());
                parts.push(cast_expression);
            },
            "number_literal" => {
                let number_literal = handle_number_literal(node, src.clone());
                parts.push(number_literal);
            },
            "char_literal" => {
                let char_literal = handle_char_literal(node, src.clone());
                parts.push(char_literal);
            },
            "string_literal" => {
                let string_literal = handle_string_literal(node, src.clone());
                parts.push(string_literal);
            },
            "subscript_expression" => {
                let subscript_expression = handle_subscript_expression(node, src.clone());
                parts.push(subscript_expression);
            },
            "pointer_expression" => {
                let pointer_expression = handle_pointer_expression(node, src.clone());
                parts.push(pointer_expression);
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(node, src.clone());
                parts.push(parenthesized_expression);
            },
            "initializer_list" => {
                let initializer_list = handle_initializer_list(node, src.clone());
                parts.push(initializer_list);
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
    let mut parts = Vec::<String>::new();
    let mut temp = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "(" => (),
            ")" => (),
            "," => (),
            "parameter_declaration" => (),
            "comment" => {
                let comment = extract_comment(node, src.clone());
                parts.push(comment);
            },
            _ => println!("You shouldn't be here (comment): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap()),
        }
    }
    let mut result: String; 
    if parts.len() == 0 {
        let content = root.utf8_text(src.as_bytes()).unwrap().to_string();
        let mut lines: Vec<&str> = content.lines().collect();
        if lines.len() > 1 {
            if lines[1].starts_with(" ") && !lines[0].starts_with(" ") {
                let num_whitespace = utils::count_leading_chars(&lines[1].to_string(), ' ');
                let temp = utils::add_leading_whitespace(lines[0].to_string(), num_whitespace);
                lines[0] = temp.as_str();
                result = lines.join("\n");
            }
            else {
                result = lines.join("\n");
            }
        }
        else {
            result = root.utf8_text(src.as_bytes()).unwrap().to_string();
        }
    }
    else {
        result = parts.join(" ");
    }
    result = utils::remove_unnecessary_spaces(result);
    return result;
}

fn extract_comment(root: Node, src: String) -> String {
    let result = root.utf8_text(src.as_bytes()).unwrap().to_string();
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
                    "::" => temp += "::",
                    "<<" => temp += "<<",
                    "operator" => temp += "operator",
                    "const" => temp += " const",
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
                    "comment" => {
                        let comment = handle_comment(node, src.clone());
                        temp += format!("{} ", comment).as_str();
                    },
                    _ => println!("You shouldn't be here (function_declarator): {}\n", subnode.grammar_name()),
                }
            }
            result += temp.as_str();
        }
    }
    result = utils::remove_reference_spaces(result);
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
            "reference_declarator" => {
                let reference_declarator = handle_reference_declarator(node, src.clone());
                result += format!(" {}", reference_declarator).as_str();
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
            "qualified_identifier" => {
                let qualified_identifier = handle_qualified_identifier(node, src.clone());
                parts.push(qualified_identifier);
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
            "string_literal" => {
                let string_literal = handle_string_literal(node, src.clone());
                parts.push(string_literal);
            },
            "preproc_defined" => {
                let preproc_defined = handle_preproc_defined(node, src.clone());
                parts.push(preproc_defined);
            },
            "null" => parts.push("NULL".to_string()),
            "true" => parts.push("true".to_string()),
            "false" => parts.push("false".to_string()),
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
            "qualified_identifier" => {
                let qualified_identifier = handle_qualified_identifier(node, src.clone());
                parts.push(qualified_identifier);
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
            "string_literal" => {
                let string_literal = handle_string_literal(node, src.clone());
                parts.push(string_literal);
            },
            "preproc_defined" => {
                let preproc_defined = handle_preproc_defined(node, src.clone());
                parts.push(preproc_defined);
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
            "pointer_expression" => {
                let pointer_expression = handle_pointer_expression(node, src.clone());
                result += pointer_expression.as_str();
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
            "binary_expression" => {
                let binary_expression = handle_binary_expression(node, src.clone());
                result += binary_expression.as_str();
            },
            "assignment_expression" => {
                let assignment_expression = handle_assignment_expression(node, src.clone());
                result += assignment_expression.as_str();
            },
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                result += identifier.as_str();
            },
            "qualifier_identified" => {
                let qualified_identifier = handle_qualified_identifier(node, src.clone());
                result += qualified_identifier.as_str();
            },
            "pointer_expression" => {
                let pointer_expression = handle_pointer_expression(node, src.clone());
                result += pointer_expression.as_str();
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
            "qualified_identifier" => {
                let qualified_identifier = handle_qualified_identifier(node, src.clone());
                result += qualified_identifier.as_str();
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
            "qualified_identifier" => {
                let qualified_identifier = handle_qualified_identifier(node, src.clone());
                parts.push(qualified_identifier);
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
            "abstract_reference_declarator" => {
                let abstract_reference_declarator = handle_abstract_reference_declarator(node, src.clone());
                parts.push(abstract_reference_declarator);
            },
            "reference_declarator" => {
                let reference_declarator = handle_reference_declarator(node, src.clone());
                parts.push(reference_declarator);
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
            "subscript_argument_list" => {
                let subscript_argument_list = handle_subscript_argument_list(node, src.clone());
                result += subscript_argument_list.as_str();
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
            "this" => result += "this",
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
            "string_literal" => {
                let string_literal = handle_string_literal(node, src.clone());
                parts.push(string_literal);
            },
            "argument_list" => {
                let argument_list = handle_argument_list(node, src.clone());
                parts.push(argument_list);
            },
            "char_literal" => {
                let char_literal = handle_char_literal(node, src.clone());
                parts.push(char_literal);
            },
            "new_expression" => {
                let new_expression = handle_new_expression(node, src.clone());
                parts.push(new_expression);
            },
            "lambda_expression" => {
                let mut lambda_expression = handle_lambda_expression(node, src.clone());
                lambda_expression = utils::add_all_leading_tabs(lambda_expression);
                parts.push(lambda_expression);
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
            "qualified_identifier" => {
                let qualified_identifier = handle_qualified_identifier(node, src.clone());
                result += qualified_identifier.as_str();
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
            "template_type" => {
                let template_type = handle_template_type(node, src.clone());
                result += format!("{} ", template_type).as_str();
            },
            "type_qualifier" => {
                let type_qualifier = handle_type_qualifier(node, src.clone());
                result += format!("{} ", type_qualifier).as_str();
            },
            _ => println!("You shouldn't be here (type_descriptor): {}\n", node.grammar_name()),
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
            "qualified_identifier" => {
                let qualified_identifier = handle_qualified_identifier(node, src.clone());
                result += qualified_identifier.as_str();
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
            "call_expression" => {
                let call_expression = handle_call_expression(node, src.clone());
                result += call_expression.as_str();
            },
            "conditional_expression" => {
                let conditional_expression = handle_conditional_expression(node, src.clone());
                result += conditional_expression.as_str();
            },
            "binary_expression" => {
                let binary_expression = handle_binary_expression(node, src.clone());
                result += binary_expression.as_str();
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
            "qualified_identifier" => {
                let qualified_identifier = handle_qualified_identifier(node, src.clone());
                parts.push(qualified_identifier);
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
            "string_literal" => {
                let string_literal = handle_string_literal(node, src.clone());
                parts.push(string_literal);
            },
            "true" => parts.push("true".to_string()),
            "false" => parts.push("false".to_string()),
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
                result += expression_statement.as_str();
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
            ":" => result += ":\n",
            _ => println!("You shouldn't be here (labeled_statement): {}\n", node.grammar_name()),
        }
    }
    result = utils::add_all_leading_tabs(result);
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
                if !reached_compound { temp += update_expression.as_str(); }
                else { vec.push(update_expression); }
            },
            "assignment_expression" => {
                let assignment_expression = handle_assignment_expression(node, src.clone());
                vec.push(assignment_expression);
            },
            "binary_expression" => {
                let binary_expression = handle_binary_expression(node, src.clone());
                if !reached_compound { temp += format!(" {}", binary_expression).as_str(); }
                else { vec.push(binary_expression); }
            },
            "declaration" => {
                let declaration = handle_declaration(node, src.clone());
                if !reached_compound { temp += declaration.as_str(); }
                else { vec.push(declaration); }
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
                let field_declaration = handle_field_declaration(node, src.clone());
                parts.push(format!("\t{}", field_declaration));
            },
            "comment" => {
                let comment = handle_comment(node, src.clone());
                parts.push(format!("\t{}", comment));
            },
            "access_specifier" => {
                let access_specifier = handle_access_specifier(node, src.clone());
                parts.push(access_specifier);
            },
            "function_definition" => {
                let mut function_definition = handle_function_definition(node, src.clone());
                function_definition = utils::add_all_leading_tabs(function_definition);
                parts.push(function_definition);
            },
            "{" => parts.push("{".to_string()),
            "}" => parts.push("}".to_string()),
            ":" => (),
            ";" => (),
            _ => println!("You shouldn't be here (field_declaration_list): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap()),
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
            "qualified_identifier" => {
                let qualified_identifier = handle_qualified_identifier(node, src.clone());
                result += format!("{} ", qualified_identifier).as_str();
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
            "function_declarator" => {
                let function_declarator = handle_function_declarator(node, src.clone());
                result += format!("{} ", function_declarator).as_str();
            },
            "storage_class_specifier" => {
                let storage_class_specifier = handle_storage_class_specifier(node, src.clone());
                result += format!("{} ", storage_class_specifier).as_str();
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
            "class" => result += "class ",
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
            "static" => result += "static ",
            "inline" => result += "inline ",
            _ => println!("You shouldn't be here (storage_class_specifier): {}\n", node.grammar_name()),
        }
    }
    result = result.trim_end().to_string();
    return result;
}


fn handle_preproc_ifdef(root: Node, src: String) -> String {
    let mut parts = Vec::<String>::new();
    let mut temp = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "#ifndef" => {
                temp += "#ifndef ";
            },
            "#ifdef" => {
                temp += "#ifdef ";
            },
            "#endif" => {
                parts.push("#endif".to_string());
            },
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                temp += identifier.as_str();
                parts.push(temp);
                temp = String::new();
            },
            "preproc_def" => {
                let preproc_def = handle_preproc_def(node, src.clone());
                parts.push(preproc_def);
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
            "declaration" => {
                let declaration = handle_declaration(node, src.clone());
                parts.push(declaration);
            },
            "if_statement" => {
                let if_statement = handle_if_statement(node, src.clone());
                parts.push(if_statement);
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
            "expression_statement" => {
                let expression_statement = handle_expression_statement(node, src.clone());
                parts.push(expression_statement);
            },
            "namespace_alias_definition" => {
                let namespace_alias_definition = handle_namespace_alias_definition(node, src.clone());
                parts.push(namespace_alias_definition);
            },
            "preproc_def" => {
                let preproc_def = handle_preproc_def(node, src.clone());
                parts.push(preproc_def);
            },
            "preproc_include" => {
                let preproc_include = handle_preproc_include(node, src.clone());
                parts.push(preproc_include);
            },
            "preproc_elif" => {
                let preproc_elif = handle_preproc_elif(node, src.clone());
                parts.push(preproc_elif);
            },
            "preproc_defined" => {
                let preproc_defined = handle_preproc_defined(node, src.clone());
                temp += preproc_defined.as_str();
                parts.push(temp);
                temp = String::new();
            },
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                temp += identifier.as_str();
                parts.push(temp);
                temp = String::new();
            },
            "\n" => (),
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
            "declaration" => {
                let declaration = handle_declaration(node, src.clone());
                parts.push(declaration);
            },
            "expression_statement" => {
                let expression_statement = handle_expression_statement(node, src.clone());
                parts.push(expression_statement);
            },
            "preproc_def" => {
                let preproc_def = handle_preproc_def(node, src.clone());
                parts.push(preproc_def);
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
            "." => result += ".",
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
            "this" => result += "this",
            _ => println!("You shouldn't be here (inner_field_expression): {}\n", node.grammar_name()),
        }
    }
    return result;
}

fn handle_parameter_list(root: Node, src: String) -> String {
    let result = root.utf8_text(src.as_bytes()).unwrap().to_string();
    return result;
}

fn handle_type_identifier(root: Node, src: String) -> String {
    return root.utf8_text(src.as_bytes()).unwrap().to_string();
}

fn handle_field_identifier(root: Node, src: String) -> String {
    return root.utf8_text(src.as_bytes()).unwrap().to_string();
}

fn handle_using_declaration(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "using" => result += "using ",
            "namespace" => result += "namespace ",
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                result += identifier.as_str();
            },
            "qualified_identifier" => {
                let qualified_identifier = handle_qualified_identifier(node, src.clone());
                result += qualified_identifier.as_str();
            },
            ";" => result += ";",
            _ => println!("Using declaration: {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_for_range_loop(root: Node, src: String) -> String {
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
                if !reached_compound { temp += update_expression.as_str(); }
                else { vec.push(update_expression); }
            },
            "assignment_expression" => {
                let assignment_expression = handle_assignment_expression(node, src.clone());
                vec.push(assignment_expression);
            },
            "binary_expression" => {
                let binary_expression = handle_binary_expression(node, src.clone());
                if !reached_compound { temp += format!(" {}", binary_expression).as_str(); }
                else { vec.push(binary_expression); }
            },
            "declaration" => {
                let declaration = handle_declaration(node, src.clone());
                if !reached_compound { temp += declaration.as_str(); }
                else { vec.push(declaration); }
            },
            "expression_statement" => {
                let expression_statement = handle_expression_statement(node, src.clone());
                vec.push(expression_statement);
            },
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                temp += format!("{} ", identifier).as_str();
            },
            "qualified_identifier" => {
                let qualified_identifier = handle_qualified_identifier(node, src.clone());
                temp += format!("{} ", qualified_identifier).as_str();
            },
            "primitive_type" => {
                let primitive_type = handle_primitive_type(node, src.clone());
                temp += format!("{} ", primitive_type).as_str();
            },
            "field_expression" => {
                let field_expression = handle_field_expression(node, src.clone());
                temp += format!("{} ", field_expression).as_str();
            },
            "reference_declarator" => {
                let reference_declarator = handle_reference_declarator(node, src.clone());
                temp += format!("{} ", reference_declarator).as_str();
            },
            "type_qualifier" => {
                let type_qualifier = handle_type_qualifier(node, src.clone());
                temp += format!("{} ", type_qualifier).as_str();
            },
            "call_expression" => {
                let call_expression = handle_call_expression(node, src.clone());
                temp += format!("{} ", call_expression).as_str();
            },
            "pointer_declarator" => {
                let pointer_declarator = handle_pointer_declarator(node, src.clone());
                temp += format!("{} ", pointer_declarator).as_str();
            },
            "placeholder_type_specifier" => temp += "auto",
            ":" => temp += ": ",
            ";" => temp += "; ",
            "(" => temp += "(",
            ")" => {
                temp = temp.trim_end().to_string();
                temp += ")";
                temp = utils::remove_reference_spaces(temp);
                vec.push(temp);
                temp = "".to_string();
            },
            "for" => temp += "for ",
            _ => println!("You shouldn't be here (for_range_loop): {} : {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap()),
        }
    }
    let result = vec.join(" ");
    return result;
}

fn handle_qualified_identifier(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                result += identifier.as_str();
            },
            "template_type" => {
                let template_type = handle_template_type(node, src.clone());
                result += template_type.as_str();
            },
            "qualified_identifier" => {
                let qualified_identifier = handle_qualified_identifier(node, src.clone());
                result += qualified_identifier.as_str();
            },
            "template_function" => {
                let template_function = handle_template_function(node, src.clone());
                result += template_function.as_str();
            },
            "::" => result += "::",
            _ => println!("You shouldn't be here (qualified_identifier): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_template_type(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                result += identifier.as_str();
            },
            "template_argument_list" => {
                let template_argument_list = handle_template_argument_list(node, src.clone());
                result += template_argument_list.as_str();
            },
            _ => println!("You shouldn't be here (template_type): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_template_function(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                result += identifier.as_str();
            },
            "qualified_identifier" => {
                let qualified_identifier = handle_qualified_identifier(node, src.clone());
                result += qualified_identifier.as_str();
            },
            "template_argument_list" => {
                let template_argument_list = handle_template_argument_list(node, src.clone());
                result += template_argument_list.as_str();
            },
            _ => println!("You shouldn't be here (template_function): {} : {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_template_argument_list(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "type_descriptor" => {
                let type_descriptor = handle_type_descriptor(node, src.clone());
                result += type_descriptor.as_str();
            },
            "<" => result += "<",
            ">" => result += ">",
            "," => result += ", ",
            _ => println!("You shouldn't be here (template_argument_list): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_condition_clause(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "binary_expression" => {
                let binary_expression = handle_binary_expression(node, src.clone());
                result += binary_expression.as_str();
            },
            "unary_expression" => {
                let unary_expression = handle_unary_expression(node, src.clone());
                result += unary_expression.as_str();
            },
            "call_expression" => {
                let call_expression = handle_call_expression(node, src.clone());
                result += call_expression.as_str();
            },
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                result += identifier.as_str();
            },
            "(" => result += "(",
            ")" => result += ")",
            "true" => result += "true",
            "false" => result += "false",
            _ => println!("You shouldn't be here (condition_clause): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_raw_string_literal(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "raw_string_content" => result += node.utf8_text(src.as_bytes()).unwrap(),
            "R\"" => result += "R\"",
            "\"" => result += "\"",
            "(" => result += "(",
            ")" => result += ")",
            _ => println!("You shouldn't be here (raw_string_literal): {}: {}", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_subscript_argument_list(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "number_literal" => {
                let number_literal = handle_number_literal(node, src.clone());
                result += number_literal.as_str();
            },
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                result += identifier.as_str();
            },
            "binary_expression" => {
                let binary_expression = handle_binary_expression(node, src.clone());
                result += binary_expression.as_str();
            },
            "[" => result += "[",
            "]" => result += "]",
            _ => println!("You shouldn't be here (subscript_argument_list): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_field_initializer_list(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "field_initializer" => {
                let field_initializer = handle_field_initializer(node, src.clone());
                result += field_initializer.as_str();
            },
            ":" => result += "\t: ",
            "," => result += ", ",
            _ => println!("You shouldn't be here (field_initializer_list): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_field_initializer(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                result += identifier.as_str();
            },
            "argument_list" => {
                let argument_list = handle_argument_list(node, src.clone());
                result += argument_list.as_str();
            },
            _ => println!("You shouldn't be here (field_initializer): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_reference_declarator(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                result += identifier.as_str();
            },
            "function_declarator" => {
                let function_declarator = handle_function_declarator(node, src.clone());
                result += function_declarator.as_str();
            },
            "structured_binding_declarator" => {
                let structured_binding_declarator = handle_structured_binding_declarator(node, src.clone());
                result += structured_binding_declarator.as_str();
            },
            "&" => result += "& ",
            _ => println!("You shouldn't be here (reference_declarator): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_abstract_reference_declarator(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "&" => result += "&",
            _ => println!("You shouldn't be here (abstract_reference_declarator): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_class_specifier(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                result += format!("{}\n", identifier).as_str();
            },
            "field_declaration_list" => {
                let field_declaration_list = handle_field_declaration_list(node, src.clone());
                result += field_declaration_list.as_str();
            },
            "class" => result += "class ",
            _ => println!("You shouldn't be here (class_specifier): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_template_declaration(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "template_parameter_list" => {
                let template_parameter_list = handle_template_parameter_list(node, src.clone());
                result += format!("{}\n", template_parameter_list).as_str();
            },
            "function_definition" => {
                let function_definition = handle_function_definition(node, src.clone());
                result += function_definition.as_str();
            }
            "template" => result += "template ",
            _ => println!("You shouldn't be here (template_declaration): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_template_parameter_list(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "type_parameter_declaration" => {
                let type_parameter_declaration = handle_type_parameter_declaration(node, src.clone());
                result += type_parameter_declaration.as_str();
            },
            "<" => result += "<",
            ">" => result += ">",
            _ => println!("You shouldn't be here (template_parameter_list): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_type_parameter_declaration(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                result += identifier.as_str();
            }
            "typename" => result += "typename ",
            _ => println!("You should't be here (type_parameter_declaration): {} : {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_access_specifier(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "public" => result += "public:",
            "private" => result += "private:",
            _ => println!("You shouldn't be here (access_specifier): {} : {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_gnu_asm_expression(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "gnu_asm_expression" => {
                result += node.utf8_text(src.as_bytes()).unwrap();
            },
            ";" => result += ";",
            _ => println!("Gnu asm expression: {} : {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_try_statement(root: Node, src: String) -> String {
    let mut parts = Vec::<String>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "try" => parts.push("try".to_string()),
            "compound_statement" => {
                let compound_statement = handle_compound_statement(node, src.clone());
                parts.push(compound_statement);
            }
            "catch_clause" => {
                let catch_clause = handle_catch_clause(node, src.clone());
                parts.push(catch_clause);
            }
            _ => println!("You shouldn't be here (try_statement): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let result = parts.join(" ");
    return result;
}

fn handle_catch_clause(root: Node, src: String) -> String {
    let mut parts = Vec::<String>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "catch" => parts.push("catch".to_string()),
            "parameter_list" => {
                let parameter_list = handle_parameter_list(node, src.clone());
                parts.push(parameter_list);
            },
            "compound_statement" => {
                let compound_statement = handle_compound_statement(node, src.clone());
                parts.push(compound_statement);
            },
            _ => println!("You shouldn't be here (catch_clause): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let result = parts.join(" ");
    return result;
}

fn handle_lambda_expression(root: Node, src: String) -> String {
    let mut parts = Vec::<String>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "lambda_capture_specifier" => {
                let lambda_capture_specifier = handle_lambda_capture_specifier(node, src.clone());
                parts.push(lambda_capture_specifier);
            },
            "abstract_function_declarator" => {
                let abstract_function_declarator = handle_abstract_function_declarator(node, src.clone());
                parts.push(abstract_function_declarator);
            },
            "compound_statement" => {
                let compound_statement = handle_compound_statement(node, src.clone());
                parts.push(compound_statement);
            },
            "lambda_expression" => {
                let mut lambda_expression = handle_lambda_expression(node, src.clone());
                parts.push(lambda_expression);
            },
            ";" => (), // handled in compound_statement
            _ => println!("You shouldn't be here (lambda_expression): {} : {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let mut result = parts.join(" ");
    // This function happens to work here too
    result = utils::remove_object_constructor_space(result);
    return result;
}

fn handle_lambda_capture_specifier(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                result += identifier.as_str();
            },
            "lambda_default_capture" => result += "=",
            "this" => result += "this",
            "&" => result += "&",
            "*" => result += "*",
            "," => result += ", ",
            "[" => result += "[",
            "]" => result += "]",
            _ => println!("You shouldn't be here (lambda_capture_specifier): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_abstract_function_declarator(root: Node, src: String) -> String {
    let mut parts = Vec::<String>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "parameter_list" => {
                let parameter_list = handle_parameter_list(node, src.clone());
                parts.push(parameter_list);
            }
            _ => println!("You shouldn't be here (abstract_function_declarator): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let result = parts.join(" ");
    return result;
}

fn handle_namespace_alias_definition(root: Node, src: String) -> String {
    let mut parts = Vec::<String>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "namespace" => parts.push("namespace".to_string()),
            "=" => parts.push("=".to_string()),
            ";" => parts.push(";".to_string()),
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                parts.push(identifier);
            },
            "nested_namespace_specifier" => {
                let nested_namespace_specifier = handle_nested_namespace_specifier(node, src.clone());
                parts.push(nested_namespace_specifier);
            },
            _ => println!("You shouldn't be here (namespace_alias_definition): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let mut result = parts.join(" ");
    result = utils::remove_unnecessary_spaces(result);
    return result;
}

fn handle_nested_namespace_specifier(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                result += identifier.as_str();
            },
            "::" => result += "::",
            _ => println!("You shouldn't be here (nested_namespace_specifier): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_preproc_elif(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "#elif" => result += "#elif ",
            "\n" => result += "\n",
            "binary_expression" => {
                let binary_expression = handle_binary_expression(node, src.clone());
                result += binary_expression.as_str();
            },
            "preproc_def" => {
                let preproc_def = handle_preproc_def(node, src.clone());
                result += format!("{}\n", preproc_def).as_str();
            },
            "preproc_elif" => {
                let preproc_elif = handle_preproc_elif(node, src.clone());
                result += format!("{}\n", preproc_elif).as_str();
            },
            "preproc_defined" => {
                let preproc_defined = handle_preproc_defined(node, src.clone());
                result += format!("{}\n", preproc_defined).as_str();
            },
            "preproc_else" => {
                let preproc_else = handle_preproc_else(node, src.clone());
                result += format!("{}\n", preproc_else).as_str();
            },
            _ => println!("You shouldn't be here (preproc_elif): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_preproc_defined(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "defined" => result += "defined",
            "(" => result += "(",
            ")" => result += ")",
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                result += identifier.as_str();
            },
            _ => println!("You shouldn't be here (preproc_defined): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_alias_declaration(root: Node, src: String) -> String {
    let mut parts = Vec::<String>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "using" => parts.push("using".to_string()),
            ";" => parts.push(";".to_string()),
            "=" => parts.push("=".to_string()),
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                parts.push(identifier);
            },
            "type_descriptor" => {
                let type_descriptor = handle_type_descriptor(node, src.clone());
                parts.push(type_descriptor);
            },
            _ => println!("You shouldn't be here (alias_declaration): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let mut result = parts.join(" ");
    result = utils::remove_unnecessary_spaces(result);
    return result;
}

fn handle_new_expression(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "new" => result += "new ",
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                result += identifier.as_str();
            },
            "argument_list" => {
                let argument_list = handle_argument_list(node, src.clone());
                result += argument_list.as_str();
            },
            _ => println!("You shouldn't be here (new_expression): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_namespace_definition(root: Node, src: String) -> String {
    let mut parts = Vec::<String>::new();
    let mut temp = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "namespace" => temp += "namespace ",
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                temp += identifier.as_str();
                parts.push(temp);
                temp = "".to_string();
            },
            "declaration_list" => {
                let declaration_list = handle_declaration_list(node, src.clone());
                parts.push(declaration_list);
            }
            _ => println!("Namespace definition: {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let result = parts.join("\n");
    return result;
}

fn handle_declaration_list(root: Node, src: String) -> String {
    let mut parts = Vec::<String>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "{" => parts.push("{".to_string()),
            "}" => parts.push("}".to_string()),
            "function_definition" => {
                let mut function_definition = handle_function_definition(node, src.clone());
                function_definition = utils::add_all_leading_tabs(function_definition);
                parts.push(function_definition);
            },
            _ => println!("You shouldn't be here (declaration_list): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let result = parts.join("\n");
    return result;
}

fn handle_structured_binding_declarator(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "[" => result += "[",
            "]" => result += "]",
            "," => result += ", ",
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                result += identifier.as_str();
            }
            _ => println!("Structured binding declarator: {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}
