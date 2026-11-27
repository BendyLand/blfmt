use tree_sitter::{Tree, Node};
use crate::{c_format, utils};

fn node_has_missing_semicolon_c(root: Node) -> bool {
    for child in root.children(&mut root.walk()) {
        if child.grammar_name() == ";" && child.is_missing() {
            return true;
        }
    }
    false
}

pub fn traverse_c_ast(ast: Tree, src: String, style: utils::Style) -> String {
    let root = ast.root_node();
    let mut result = String::new();
    let mut last_group_kind = "";
    let mut pending_incomplete_decl = false;
    for child in root.children(&mut root.walk()) {
        match child.grammar_name() {
            "preproc_include" => {
                pending_incomplete_decl = false;
                if last_group_kind != "preproc_include" { result += "\n"; }
                let preproc_include = handle_preproc_include(child, src.clone());
                result += format!("{}\n", preproc_include).as_str();
                last_group_kind = "preproc_include";
            },
            "declaration" => {
                let missing_semi = node_has_missing_semicolon_c(child);
                let mut declaration = handle_declaration(child, src.clone());
                if declaration.contains("*") { declaration = utils::remove_pointer_spaces(declaration); }
                if pending_incomplete_decl {
                    utils::merge_incomplete_decl_into_result(&mut result, &declaration);
                } else {
                    if last_group_kind.contains("preproc") { result += "\n"; }
                    result += format!("{}\n", declaration).as_str();
                }
                pending_incomplete_decl = missing_semi;
                last_group_kind = "declaration";
            },
            "function_definition" => {
                pending_incomplete_decl = false;
                if last_group_kind != "function_definition".to_string() { result += "\n"; }
                let function_definition = handle_function_definition(child, src.clone());
                result += format!("{}\n\n", function_definition).as_str();
                last_group_kind = "function_definition";
            },
            "expression_statement" => {
                let missing_semi = node_has_missing_semicolon_c(child);
                let expression_statement = handle_expression_statement(child, src.clone());
                if pending_incomplete_decl {
                    utils::merge_incomplete_decl_into_result(&mut result, &expression_statement);
                    pending_incomplete_decl = missing_semi;
                } else {
                    result += format!("{}\n\n", expression_statement).as_str();
                    pending_incomplete_decl = missing_semi;
                    last_group_kind = "expression_statement";
                }
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
                let comment = handle_comment(child, src.clone());
                if !result.trim().is_empty() && utils::trailing_comment_gap(child.start_byte(), &src).is_some() {
                    utils::append_comment_to_result(&mut result, &comment, child.start_byte(), &src, "", "\n", true);
                } else {
                    if last_group_kind == "declaration" || last_group_kind == "preproc_include" {
                        result += "\n";
                    }
                    result += format!("{}\n", comment).as_str();
                    last_group_kind = "comment";
                }
            },
            "ERROR" => {
                let error = handle_error(child, src.clone());
                result += format!("{}\n\n", error).as_str();
                last_group_kind = "ERROR";
            },
            "type_definition" => {
                if last_group_kind.contains("preproc") { result += "\n"; }
                let type_definition = handle_type_definition(child, src.clone());
                result += format!("{}\n\n", type_definition).as_str();
                last_group_kind = "type_definition";
            },
            "struct_specifier" => {
                let struct_specifier = handle_struct_specifier(child, src.clone());
                result += format!("{};\n\n", struct_specifier.trim_end()).as_str();
                last_group_kind = "struct_specifier";
            },
            "union_specifier" => {
                let union_specifier = handle_union_specifier(child, src.clone());
                result += format!("{};\n\n", union_specifier.trim_end()).as_str();
                last_group_kind = "union_specifier";
            },
            "enum_specifier" => {
                let enum_specifier = handle_enum_specifier(child, src.clone());
                result += format!("{};\n\n", enum_specifier.trim_end()).as_str();
                last_group_kind = "enum_specifier";
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
            "preproc_elif" => {
                let preproc_elif = handle_preproc_elif(child, src.clone());
                result += format!("{}\n\n", preproc_elif).as_str();
                last_group_kind = "preproc_elif";
            },
            "preproc_function_def" => {
                if !last_group_kind.contains("preproc") { result += "\n"; }
                let preproc_function_def = handle_preproc_function_def(child, src.clone());
                result += format!("{}\n", preproc_function_def).as_str();
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
            "declaration_list" => {
                let declaration_list = handle_declaration_list(child, src.clone());
                result += format!("{}\n\n", declaration_list).as_str();
                last_group_kind = "declaration_list";
            },
            ";" => (), // handled in functions above
            _ => println!("Unknown c grammar name 1: {}\n", &child.grammar_name()),
        }
    }
    if result.is_empty() {
        panic!("Fatal: Unable to parse language grammar");
    }
    result = utils::sort_include_groups(result);
    utils::format_else_lines(&mut result, &style);
    utils::close_empty_curly_brace_blocks(&mut result);
    result = utils::fix_stars(result);
    utils::tidy_up_loose_ends(&mut result, style);
    return result;
}

fn try_split_leading_statement_macro_c(children: &[Node], idx: usize, src: &str) -> Option<(String, usize)> {
    let node = children[idx];
    if node.start_position().row >= node.end_position().row {
        return None;
    }
    if utils::trailing_comment_gap(node.start_byte(), src).is_some() {
        return None;
    }
    let span = &src[node.start_byte()..node.end_byte()];
    let rel_nl = span.find('\n')?;
    let first_line = &span[..rel_nl];
    let macro_tok = utils::strip_trailing_comment(first_line).trim();
    if !utils::is_standalone_statement_macro_token(macro_tok) {
        return None;
    }
    let rem_start_byte = node.start_byte() + rel_nl + 1;

    let mut end_idx = idx;
    loop {
        let cur_end_byte = children[end_idx].end_byte();
        let cur_slice = src[rem_start_byte..cur_end_byte].trim_end();
        let ends_complete = cur_slice.ends_with(';') || cur_slice.ends_with('}');
        if !ends_complete && end_idx + 1 < children.len() {
            let nxt = children[end_idx + 1];
            if nxt.grammar_name() != "}" && nxt.grammar_name() != "comment" {
                end_idx += 1;
                continue;
            }
        }
        if ends_complete && end_idx + 1 < children.len() {
            let nxt = children[end_idx + 1];
            let nxt_txt = nxt.utf8_text(src.as_bytes()).unwrap_or("").trim_start();
            if nxt_txt == "else" || nxt_txt.starts_with("else ") || nxt_txt.starts_with("else\t") || nxt_txt.starts_with("else{") {
                end_idx += 1;
                continue;
            }
        }
        break;
    }

    let rem_slice = &src[rem_start_byte..children[end_idx].end_byte()];
    if rem_slice.trim().is_empty() {
        return None;
    }

    let wrapped = format!("{{\n{}\n}}", rem_slice);
    let tree = c_format::parse_existing_c_file(wrapped.clone());
    let root = tree.root_node();
    let mut compound_opt = None;
    for child in root.children(&mut root.walk()) {
        if child.grammar_name() == "compound_statement" {
            compound_opt = Some(child);
            break;
        }
    }
    let compound = compound_opt?;

    if end_idx == idx && !node.has_error() && node.grammar_name() == "declaration" {
        let mut first_stmt = None;
        for c in compound.named_children(&mut compound.walk()) {
            if c.grammar_name() != "comment" {
                first_stmt = Some(c);
                break;
            }
        }
        let Some(stmt) = first_stmt else {
            return None;
        };
        let is_macro_prefix = match stmt.grammar_name() {
            "declaration" => true,
            "expression_statement" => stmt
                .named_child(0)
                .map(|e| e.grammar_name() == "call_expression")
                .unwrap_or(false),
            _ => true,
        };
        if !is_macro_prefix {
            return None;
        }
    }

    let formatted_compound = handle_compound_statement(compound, wrapped);
    let inner = formatted_compound
        .strip_prefix("{\n")
        .unwrap_or(&formatted_compound)
        .strip_suffix('}')
        .unwrap_or(&formatted_compound);
    let first_formatted = first_line.trim();
    Some((format!("\t{}\n{}", first_formatted, inner), end_idx))
}

fn handle_compound_statement(root: Node, src: String) -> String {
    let mut result = String::new();
    let children: Vec<Node> = root.children(&mut root.walk()).collect();
    let mut idx = 0usize;
    while idx < children.len() {
        let node = children[idx];
        if matches!(
            node.grammar_name(),
            "function_definition" | "declaration" | "expression_statement" | "ERROR"
        ) {
            if let Some((split_out, new_end_idx)) = try_split_leading_statement_macro_c(&children, idx, &src) {
                result += split_out.as_str();
                idx = new_end_idx + 1;
                continue;
            }
        }
        idx += 1;
        match node.grammar_name() {
            "preproc_include" => {
                let preproc_include = handle_preproc_include(node, src.clone());
                result += format!("\t{}\n", preproc_include).as_str();
            },
            "declaration" => {
                let mut declaration = handle_declaration(node, src.clone());
                declaration = utils::add_all_leading_tabs(declaration);
                result += format!("{}\n", declaration).as_str();
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
                utils::append_comment_to_result(&mut result, &comment, node.start_byte(), &src, "\t", "\n", true);
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
            "union_specifier" => {
                let union_specifier = handle_union_specifier(node, src.clone());
                result += format!("\t{}\n", union_specifier).as_str();
            },
            "enum_specifier" => {
                let enum_specifier = handle_enum_specifier(node, src.clone());
                result += format!("\t{}\n", enum_specifier).as_str();
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
    let in_for_loop = root.parent().map(|p| p.grammar_name() == "for_statement").unwrap_or(false);
    let mut has_comma = false;
    for node in root.children(&mut root.walk()) {
        if node.grammar_name() == "," {
            has_comma = true;
            break;
        }
    }
    let mut specifiers = Vec::<String>::new();
    let mut declarators = Vec::<String>::new();
    let mut current_decl = Vec::<String>::new();
    let mut seen_declarator = false;

    for node in root.children(&mut root.walk()) {
        let is_specifier = matches!(
            node.grammar_name(),
            "primitive_type"
                | "type_qualifier"
                | "storage_class_specifier"
                | "struct_specifier"
                | "union_specifier"
                | "enum_specifier"
                | "sized_type_specifier"
                | "macro_type_specifier"
        ) || node.kind() == "type_identifier";

        let piece = match node.grammar_name() {
            "compound_statement" => Some(handle_compound_statement(node.clone(), src.clone())),
            "init_declarator" => Some(handle_init_declarator(node, src.clone())),
            "primitive_type" => Some(handle_primitive_type(node, src.clone())),
            "identifier" => Some(handle_identifier(node, src.clone())),
            "function_declarator" => Some(handle_function_declarator(node, src.clone())),
            "parenthesized_declarator" => Some(handle_parenthesized_declarator(node, src.clone())),
            "type_qualifier" => Some(handle_type_qualifier(node, src.clone())),
            "ERROR" => Some(handle_error(node, src.clone())),
            "pointer_declarator" => Some(handle_pointer_declarator(node, src.clone())),
            "array_declarator" => Some(handle_array_declarator(node, src.clone())),
            "storage_class_specifier" => Some(handle_storage_class_specifier(node, src.clone())),
            "struct_specifier" => Some(handle_struct_specifier(node, src.clone())),
            "union_specifier" => Some(handle_union_specifier(node, src.clone())),
            "enum_specifier" => Some(handle_enum_specifier(node, src.clone())),
            "sized_type_specifier" => Some(handle_sized_type_specifier(node, src.clone())),
            "macro_type_specifier" => Some(handle_macro_type_specifier(node, src.clone())),
            ";" => {
                if !current_decl.is_empty() {
                    declarators.push(current_decl.join(" "));
                    current_decl.clear();
                }
                if !node.is_missing() {
                    parts.push(";".to_string());
                }
                None
            },
            "," => {
                if !current_decl.is_empty() {
                    declarators.push(current_decl.join(" "));
                    current_decl.clear();
                }
                parts.push(",".to_string());
                None
            },
            _ => {
                println!("You shouldn't be here (c declaration): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or(""));
                None
            },
        };

        if let Some(p) = piece {
            if !seen_declarator && is_specifier {
                specifiers.push(p.clone());
            } else {
                seen_declarator = true;
                current_decl.push(p.clone());
            }
            parts.push(p);
        }
    }

    let spec_str = specifiers.join(" ");
    if has_comma && !in_for_loop && !spec_str.is_empty() && !spec_str.contains('{') && declarators.len() > 1 {
        let mut lines = Vec::<String>::new();
        for decl in declarators {
            let mut line = format!("{} {};", spec_str, decl);
            line = utils::remove_unnecessary_spaces(&line);
            if line.contains("*") {
                line = utils::remove_pointer_spaces(line);
            }
            lines.push(line);
        }
        return lines.join("\n");
    }

    result = parts.join(" ");
    if result.contains(",") { result = utils::remove_whitespace_before_commas(&result); }
    result = utils::remove_unnecessary_spaces(&result);
    if result.contains("*") { result = utils::remove_pointer_spaces(result); }
    return result;
}

fn handle_function_definition(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => {
                let compound_statement = handle_compound_statement(node, src.clone());
                result = result.trim_end().to_string();
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
                result += format!("{} ", struct_specifier.trim_end()).as_str();
            },
            "union_specifier" => {
                let union_specifier = handle_union_specifier(node.clone(), src.clone());
                result += format!("{} ", union_specifier.trim_end()).as_str();
            },
            "enum_specifier" => {
                let enum_specifier = handle_enum_specifier(node.clone(), src.clone());
                result += format!("{} ", enum_specifier.trim_end()).as_str();
            },
            "storage_class_specifier" => {
                let storage_class_specifier = handle_storage_class_specifier(node, src.clone());
                result += format!("{} ", storage_class_specifier).as_str();
            },
            "sized_type_specifier" => {
                let sized_type_specifier = handle_sized_type_specifier(node, src.clone());
                result += format!("{} ", sized_type_specifier).as_str();
            },
            "comment" => {
                let comment = handle_comment(node, src.clone());
                utils::append_comment_to_result(&mut result, &comment, node.start_byte(), &src, "", "\n", true);
            },
            _ => println!("You shouldn't be here (c function_definition): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
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
            "unary_expression" => {
                result = handle_unary_expression(node, src.clone());
            },
            "pointer_expression" => {
                result = handle_pointer_expression(node, src.clone());
            },
            "cast_expression" => {
                result = handle_cast_expression(node, src.clone());
            },
            "comma_expression" => {
                result = handle_comma_expression(node, src.clone());
            },
            "conditional_expression" => {
                result = handle_conditional_expression(node, src.clone());
            },
            "parenthesized_expression" => {
                result = handle_parenthesized_expression(node, src.clone());
            },
            "field_expression" => {
                result = handle_field_expression(node, src.clone());
            },
            "subscript_expression" => {
                result = handle_subscript_expression(node, src.clone());
            },
            "compound_literal_expression" => {
                result = handle_compound_literal_expression(node, src.clone());
            },
            "gnu_asm_expression" => {
                result = handle_gnu_asm_expression(node, src.clone());
            },
            "number_literal" => {
                result = handle_number_literal(node, src.clone());
            },
            "string_literal" => {
                result = handle_string_literal(node, src.clone());
            },
            "char_literal" => {
                result = handle_char_literal(node, src.clone());
            },
            "true" => result = "true".to_string(),
            "false" => result = "false".to_string(),
            "null" => result = "NULL".to_string(),
            "ERROR" => {
                println!(
                    "Warning: encountered ERROR node, likely due to preprocessor macro interrupt.\n\t--> {:?}\n",
                    node.utf8_text(src.as_bytes()).unwrap_or("[unreadable]")
                );
            },
            ";" => {
                if !node.is_missing() {
                    result += ";";
                }
            },
            _ => println!("You shouldn't be here (c expression_statement): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    result = utils::remove_unnecessary_spaces(&result);
    return result;
}

fn handle_assignment_expression(root: Node, src: String) -> String {
    let mut parts = Vec::<String>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            ";" => parts.push(";".to_string()),
            "=" => parts.push("=".to_string()),
            "+=" => parts.push("+=".to_string()),
            "-=" => parts.push("-=".to_string()),
            "*=" => parts.push("*=".to_string()),
            "/=" => parts.push("/=".to_string()),
            "%=" => parts.push("%=".to_string()),
            "<<=" => parts.push("<<=".to_string()),
            ">>=" => parts.push(">>=".to_string()),
            "&=" => parts.push("&=".to_string()),
            "|=" => parts.push("|=".to_string()),
            "^=" => parts.push("^=".to_string()),
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
            "string_literal" => {
                let string_literal = handle_string_literal(node, src.clone());
                parts.push(string_literal);
            },
            "concatenated_string" => {
                let concatenated_string = handle_concatenated_string(node, src.clone());
                parts.push(concatenated_string);
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
            "update_expression" => {
                let update_expression = handle_update_expression(node, src.clone());
                parts.push(update_expression);
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
            "compound_literal_expression" => {
                let compound_literal_expression = handle_compound_literal_expression(node, src.clone());
                parts.push(compound_literal_expression);
            },
            "ERROR" => {
                println!(
                    "Warning: encountered ERROR node, likely due to preprocessor macro interrupt.\n\t--> {:?}\n",
                    node.utf8_text(src.as_bytes()).unwrap_or("[unreadable]")
                );
            },
            _ => println!("You shouldn't be here (c assignment_expression): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap()),
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
            "string_literal" => {
                let string_literal = handle_string_literal(node, src.clone());
                parts.push(string_literal);
            },
            "concatenated_string" => {
                let concatenated_string = handle_concatenated_string(node, src.clone());
                parts.push(concatenated_string);
            },
            "update_expression" => {
                let update_expression = handle_update_expression(node, src.clone());
                parts.push(update_expression);
            },
            "compound_literal_expression" => {
                let compound_literal_expression = handle_compound_literal_expression(node, src.clone());
                parts.push(compound_literal_expression);
            },
            "ERROR" => {
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
            "^=" => parts.push("^=".to_string()),
            ">>=" => parts.push(">>=".to_string()),
            "<<=" => parts.push("<<=".to_string()),
            _ => println!("You shouldn't be here (c inner_assignment_expression): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let result = parts.join(" ").to_string();
    return result;
}

fn handle_inner_compound_statement(root: Node, src: String) -> String {
    let mut parts = Vec::<String>::new();
    let children: Vec<Node> = root.children(&mut root.walk()).collect();
    let mut idx = 0usize;
    while idx < children.len() {
        let node = children[idx];
        if matches!(
            node.grammar_name(),
            "function_definition" | "declaration" | "expression_statement" | "ERROR"
        ) {
            if let Some((split_out, new_end_idx)) = try_split_leading_statement_macro_c(&children, idx, &src) {
                parts.push(split_out.trim_end_matches('\n').to_string());
                idx = new_end_idx + 1;
                continue;
            }
        }
        idx += 1;
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
                let mut declaration = handle_declaration(node, src.clone());
                declaration = utils::add_all_leading_tabs(declaration);
                parts.push(declaration);
            },
            "comment" => {
                let comment = handle_comment(node, src.clone());
                utils::push_comment_to_parts(&mut parts, &comment, node.start_byte(), &src, true);
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
            "preproc_ifdef" => {
                let mut preproc_ifdef = handle_preproc_ifdef(node, src.clone());
                preproc_ifdef = utils::add_all_leading_tabs(preproc_ifdef);
                parts.push(preproc_ifdef);
            },
            "preproc_if" => {
                let mut preproc_if = handle_preproc_if(node, src.clone());
                preproc_if = utils::add_all_leading_tabs(preproc_if);
                parts.push(preproc_if);
            },
            "preproc_def" => {
                let mut preproc_def = handle_preproc_def(node, src.clone());
                preproc_def = utils::add_all_leading_tabs(preproc_def);
                parts.push(preproc_def);
            },
            "preproc_function_def" => {
                let mut preproc_function_def = handle_preproc_function_def(node, src.clone());
                preproc_function_def = utils::add_all_leading_tabs(preproc_function_def);
                parts.push(preproc_function_def);
            },
            "preproc_include" => {
                let preproc_include = handle_preproc_include(node, src.clone());
                parts.push(format!("\t{}", preproc_include));
            },
            "type_definition" => {
                let type_definition = handle_type_definition(node, src.clone());
                parts.push(format!("\t{}", type_definition));
            },
            "struct_specifier" => {
                let struct_specifier = handle_struct_specifier(node, src.clone());
                parts.push(format!("\t{}", struct_specifier));
            },
            "union_specifier" => {
                let union_specifier = handle_union_specifier(node, src.clone());
                parts.push(format!("\t{}", union_specifier));
            },
            "enum_specifier" => {
                let enum_specifier = handle_enum_specifier(node, src.clone());
                parts.push(format!("\t{}", enum_specifier));
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
            "labeled_statement" => {
                let mut labeled_statement = handle_labeled_statement(node, src.clone());
                labeled_statement = utils::add_all_leading_tabs(labeled_statement);
                parts.push(labeled_statement);
            },
            "ERROR" => {
                let mut error = handle_error(node, src.clone());
                error = utils::add_all_leading_tabs(error);
                parts.push(error);
            },
            ";" => parts.push(";".to_string()),
            "{" => parts.push("{".to_string()),
            "}" => parts.push("}".to_string()),
            _ => println!("You shouldn't be here (inner_compound_statement): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    if root.start_position().row == root.end_position().row && root.named_child_count() == 1 {
        if parts.len() == 3 {
            let inner = parts[1].trim();
            if !inner.contains('\n') && !inner.contains("//") && inner.len() < 60 {
                return format!("{{ {} }}", inner);
            }
        } else if parts.len() == 2 && parts[0].starts_with('{') && parts[1] == "}" && !parts[0].contains('\n') && !parts[0].contains("//") {
            return format!("{} }}", parts[0]);
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
                let call_expression = handle_call_expression(node, src.clone());
                result += call_expression.as_str();
            },
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                result += identifier.as_str();
            },
            "field_expression" => {
                let field_expression = handle_field_expression(node, src.clone());
                result += field_expression.as_str();
            },
            "subscript_expression" => {
                let subscript_expression = handle_subscript_expression(node, src.clone());
                result += subscript_expression.as_str();
            },
            "pointer_expression" => {
                let pointer_expression = handle_pointer_expression(node, src.clone());
                result += pointer_expression.as_str();
            },
            "cast_expression" => {
                let cast_expression = handle_cast_expression(node, src.clone());
                result += cast_expression.as_str();
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
            _ => println!("You shouldn't be here (c call_expression): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            },
            "for_statement" => {
                let for_statement = handle_for_statement(node, src.clone());
                parts.push(for_statement);
            },
            "while_statement" => {
                let while_statement = handle_while_statement(node, src.clone());
                parts.push(while_statement);
            },
            "do_statement" => {
                let do_statement = handle_do_statement(node, src.clone());
                parts.push(do_statement);
            },
            "continue_statement" => {
                parts.push("continue;".to_string());
            },
            "comment" => {
                let comment = handle_comment(node, src.clone());
                utils::push_expr_comment(&mut parts, &comment, node.start_byte(), node.end_byte(), &src);
            },
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
            ";" => parts.push(";".to_string()),
            _ => println!("You shouldn't be here (c if_statement): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            },
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
            "while_statement" => {
                let while_statement = handle_while_statement(node, src.clone());
                pieces.push(while_statement);
            },
            "do_statement" => {
                let do_statement = handle_do_statement(node, src.clone());
                pieces.push(do_statement);
            },
            "continue_statement" => {
                pieces.push("continue;".to_string());
            },
            "break_statement" => {
                pieces.push("break;".to_string());
            },
            "goto_statement" => {
                let goto_statement = handle_goto_statement(node, src.clone());
                pieces.push(goto_statement);
            },
            "comment" => {
                let comment = handle_comment(node, src.clone());
                utils::push_expr_comment(&mut pieces, &comment, node.start_byte(), node.end_byte(), &src);
            },
            ";" => pieces.push(";".to_string()),
            _ => println!("You shouldn't be here (c else_clause): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            _ => println!("You shouldn't be here (c switch_statement): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_case_statement(root: Node, src: String) -> String {
    let mut result = String::new();
    let mut opened_inline_brace = false;
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "expression_statement" => {
                let expression_statement = handle_expression_statement(node, src.clone());
                result += format!("\t{}\n", expression_statement).as_str();
            },
            "compound_statement" => {
                let compound_statement = handle_inner_compound_statement(node, src.clone());
                if result.ends_with(":\n") {
                    result.pop();
                    result += format!(" {}\n", compound_statement).as_str();
                    opened_inline_brace = true;
                } else {
                    result += format!("{}\n", compound_statement).as_str();
                }
            },
            "case" => {
                result += "case ";
            },
            "break_statement" => {
                if opened_inline_brace && result.ends_with("}\n") {
                    result.pop();
                    result += " break;";
                    opened_inline_brace = false;
                } else {
                    result += "\tbreak;";
                }
            },
            "continue_statement" => {
                if opened_inline_brace && result.ends_with("}\n") {
                    result.pop();
                    result += " continue;";
                    opened_inline_brace = false;
                } else {
                    result += "\tcontinue;";
                }
            },
            "declaration" => {
                let mut declaration = handle_declaration(node, src.clone());
                declaration = utils::add_all_leading_tabs(declaration);
                result += format!("{}\n", declaration).as_str();
            },
            "return_statement" => {
                let return_statement = handle_return_statement(node, src.clone());
                result += format!("\t{}\n", return_statement).as_str();
            },
            "if_statement" => {
                let if_statement = handle_if_statement(node, src.clone());
                result += format!("\t{}\n", if_statement).as_str();
            },
            "while_statement" => {
                let while_statement = handle_while_statement(node, src.clone());
                result += format!("\t{}\n", while_statement).as_str();
            },
            "for_statement" => {
                let for_statement = handle_for_statement(node, src.clone());
                result += format!("\t{}\n", for_statement).as_str();
            },
            "do_statement" => {
                let do_statement = handle_do_statement(node, src.clone());
                result += format!("\t{}\n", do_statement).as_str();
            },
            "goto_statement" => {
                let goto_statement = handle_goto_statement(node, src.clone());
                result += format!("\t{}\n", goto_statement).as_str();
            },
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                result += identifier.as_str();
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(node, src.clone());
                result += parenthesized_expression.as_str();
            },
            "binary_expression" => {
                let binary_expression = handle_binary_expression(node, src.clone());
                result += binary_expression.as_str();
            },
            "unary_expression" => {
                let unary_expression = handle_unary_expression(node, src.clone());
                result += unary_expression.as_str();
            },
            "cast_expression" => {
                let cast_expression = handle_cast_expression(node, src.clone());
                result += cast_expression.as_str();
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
                utils::append_comment_to_result(&mut result, &comment, node.start_byte(), &src, "\t", "\n", true);
            },
            "labeled_statement" => {
                let labeled_statment = handle_labeled_statement(node, src.clone());
                result += format!("\t{}\n", labeled_statment).as_str();
            },
            ":" => {
                result = result.trim_end().to_string();
                result += ":\n"
            },
            ";" => result += ";",
            _ => println!("You shouldn't be here (c case_statement): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            _ => println!("You shouldn't be here (c goto_statement): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            "if_statement" => {
                let if_statement = handle_if_statement(node, src.clone());
                parts.push(if_statement);
            },
            "for_statement" => {
                let for_statement = handle_for_statement(node, src.clone());
                parts.push(for_statement);
            },
            "while_statement" => {
                let while_statement = handle_while_statement(node, src.clone());
                parts.push(while_statement);
            },
            "do_statement" => {
                let do_statement = handle_do_statement(node, src.clone());
                parts.push(do_statement);
            },
            "return_statement" => {
                let return_statement = handle_return_statement(node, src.clone());
                parts.push(return_statement);
            },
            "break_statement" => parts.push("break;".to_string()),
            "continue_statement" => parts.push("continue;".to_string()),
            "goto_statement" => {
                let goto_statement = handle_goto_statement(node, src.clone());
                parts.push(goto_statement);
            },
            ";" => parts.push(";".to_string()),
            _ => println!("You shouldn't be here (c while_statement): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            "compound_literal_expression" => {
                let compound_literal_expression = handle_compound_literal_expression(node, src.clone());
                result += compound_literal_expression.as_str();
            },
            "offsetof_expression" => {
                let offsetof_expression = handle_offsetof_expression(node, src.clone());
                result += offsetof_expression.as_str();
            },
            "type_descriptor" => {
                let type_descriptor = handle_type_descriptor(node, src.clone());
                result += type_descriptor.as_str();
            },
            "ERROR" => {
                let error = handle_error(node, src.clone());
                result += error.as_str();
            },
            _ => println!("You shouldn't be here (c argument_list): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            "sizeof_expression" => {
                let sizeof_expression = handle_sizeof_expression(node.clone(), src.clone());
                parts.push(sizeof_expression);
            },
            "update_expression" => {
                let update_expression = handle_update_expression(node.clone(), src.clone());
                parts.push(update_expression);
            },
            "assignment_expression" => {
                let assignment_expression = handle_assignment_expression(node.clone(), src.clone());
                parts.push(assignment_expression);
            },
            "comma_expression" => {
                let comma_expression = handle_comma_expression(node.clone(), src.clone());
                parts.push(comma_expression);
            },
            "compound_literal_expression" => {
                let compound_literal_expression = handle_compound_literal_expression(node.clone(), src.clone());
                parts.push(compound_literal_expression);
            },
            "concatenated_string" => {
                let concatenated_string = handle_concatenated_string(node.clone(), src.clone());
                parts.push(concatenated_string);
            },
            "return" => {
                parts.push("return".to_string());
            },
            "null" => {
                parts.push("NULL".to_string());
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
                println!("You shouldn't be here (c return statement): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or(""));
            },
        }
    }
    result = parts.join(" ");
    result = utils::remove_unnecessary_spaces(&result);
    return result;
}

fn is_comment_container_c(grammar_name: &str) -> bool {
    matches!(
        grammar_name,
        "translation_unit"
            | "compound_statement"
            | "field_declaration_list"
            | "enumerator_list"
            | "declaration_list"
            | "case_statement"
            | "preproc_ifdef"
            | "preproc_if"
            | "preproc_elif"
            | "preproc_else"
    )
}

fn is_valid_ref_sibling_c(node: Node, src: &str) -> bool {
    let g = node.grammar_name();
    if matches!(
        g,
        "comment"
            | "{"
            | "}"
            | ";"
            | ","
            | ":"
            | "case"
            | "default"
            | "labeled_statement"
    ) || g.starts_with("preproc_") || g.starts_with('#')
    {
        return false;
    }
    utils::trailing_comment_gap(node.start_byte(), src).is_none()
}

fn find_comment_ref_byte_c(root: Node, src: &str) -> (Option<usize>, bool) {
    let Some(direct_parent) = root.parent() else {
        return (None, true);
    };
    if direct_parent.grammar_name() == "translation_unit" {
        return (None, true);
    }

    if !is_comment_container_c(direct_parent.grammar_name()) {
        let mut cur = direct_parent;
        while let Some(p) = cur.parent() {
            if is_comment_container_c(p.grammar_name()) {
                if p.grammar_name() == "translation_unit" {
                    return (Some(cur.start_byte()), false);
                }
                if is_valid_ref_sibling_c(cur, src) {
                    return (Some(cur.start_byte()), false);
                }
                break;
            }
            cur = p;
        }
    }

    let mut sib_opt = root.next_sibling();
    while let Some(sib) = sib_opt {
        if is_valid_ref_sibling_c(sib, src) {
            return (Some(sib.start_byte()), false);
        }
        sib_opt = sib.next_sibling();
    }

    sib_opt = root.prev_sibling();
    while let Some(sib) = sib_opt {
        if is_valid_ref_sibling_c(sib, src) {
            return (Some(sib.start_byte()), false);
        }
        sib_opt = sib.prev_sibling();
    }

    (None, false)
}

fn handle_comment(root: Node, src: String) -> String {
    let mut parts = Vec::<String>::new();
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
            _ => println!("You shouldn't be here (c comment): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap()),
        }
    }
    let result: String;
    if parts.len() == 0 {
        result = root.utf8_text(src.as_bytes()).unwrap().to_string();
    }
    else {
        result = parts.join(" ");
    }
    let (ref_byte, is_top_level) = find_comment_ref_byte_c(root, &src);
    return utils::format_standalone_comment_indent(result, root.start_byte(), ref_byte, &src, is_top_level);
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
            if !result.is_empty() && !result.ends_with(' ') && node.grammar_name() == "identifier" {
                result += " ";
            }
            result += node.utf8_text(src.as_bytes()).unwrap();
        }
        else {
            if node.grammar_name() == "ERROR" && result.ends_with(')') {
                result += " ";
            }
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
                    "parenthesized_declarator" => {
                        let parenthesized_declarator = handle_parenthesized_declarator(subnode, src.clone());
                        temp += parenthesized_declarator.as_str();
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
                        let comment = handle_comment(subnode, src.clone());
                        temp += format!("{} ", comment).as_str();
                    },
                    "ERROR" => {
                        let error = handle_error(subnode, src.clone());
                        temp += error.as_str();
                    },
                    _ => println!("You shouldn't be here (c function_declarator): {}: {}\n", subnode.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
                let child_result = handle_pointer_declarator(node.named_child(0).unwrap_or(node), src.clone());
                result += child_result.as_str();
            },
            "pointer_declarator" => {
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
            "parenthesized_declarator" => {
                let parenthesized_declarator = handle_parenthesized_declarator(node, src.clone());
                result += format!(" {}", parenthesized_declarator).as_str();
            },
            "type_identifier" => {
                let type_identifier = handle_type_identifier(node, src.clone());
                result += format!(" {}", type_identifier).as_str();
            },
            "type_qualifier" => {
                let type_qualifier = handle_type_qualifier(node, src.clone());
                result += format!(" {}", type_qualifier).as_str();
            },
            "field_identifier" => {
                let field_identifier = handle_field_identifier(node, src.clone());
                result += format!(" {}", field_identifier).as_str();
            },
            "array_declarator" => {
                let array_declarator = handle_array_declarator(node, src.clone());
                result += format!(" {}", array_declarator).as_str();
            },
            "ERROR" => {
                let error = handle_error(node, src.clone());
                result += format!(" {}", error).as_str();
            },
            _ => println!("You shouldn't be here (c pointer_declarator): {}: {}\n", node.kind(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            _ => println!("You shouldn't be here (c identifier): {}: {}\n", root.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    if result.len() == 0 {
        result = root.utf8_text(src.as_bytes()).unwrap().to_string();
    }
    return result;
}

fn handle_nested_inner_compound_statement(root: Node, src: String) -> String {
    return handle_inner_compound_statement(root, src);
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
                /*
                    This is likely the condition being checked by some kind
                    of preproc check mid-expression. In:

                    keyring_alloc(".blacklist",                                                                   
                                  GLOBAL_ROOT_UID, GLOBAL_ROOT_GID, current_cred(),                               
                                  KEY_POS_VIEW | KEY_POS_READ | KEY_POS_SEARCH |                                  
                                  KEY_POS_WRITE |                                                                 
                                  KEY_USR_VIEW | KEY_USR_READ | KEY_USR_SEARCH                                    
                #ifdef CONFIG_SYSTEM_BLACKLIST_AUTH_UPDATE                                                                    
                                 | KEY_USR_WRITE                                                                 
                #endif                                                                                                        
                                  , KEY_ALLOC_NOT_IN_QUOTA |                                                      
                                  KEY_ALLOC_SET_KEEP,                                                             
                                  restriction, NULL);

                    The ERROR would be CONFIG_SYSTEM_BLACKLIST_AUTH_UPDATE                    
                */
                // let error = handle_error(node, src.clone());
                // parts.push(error);
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
                let mut pointer_expression = handle_pointer_expression(node, src.clone());
                pointer_expression = utils::remove_dereference_spaces(pointer_expression);
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
            "preproc_defined" => {
                let preproc_defined = handle_preproc_defined(node, src.clone());
                parts.push(preproc_defined);
            },
            "string_literal" => {
                let string_literal = handle_string_literal(node, src.clone());
                parts.push(string_literal);
            },
            "conditional_expression" => {
                let conditional_expression = handle_conditional_expression(node, src.clone());
                parts.push(conditional_expression);
            },
            "offsetof_expression" => {
                let offsetof_expression = handle_offsetof_expression(node, src.clone());
                parts.push(offsetof_expression);
            },
            "comment" => {
                let comment = handle_comment(node, src.clone());
                utils::push_expr_comment(&mut parts, &comment, node.start_byte(), node.end_byte(), &src);
            },
            "true" => parts.push("true".to_string()),
            "false" => parts.push("false".to_string()),
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
            _ => println!("You shouldn't be here (c binary_expression): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let mut result = parts.join(" ");
    result = utils::remove_unnecessary_spaces(&result);
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
            "cast_expression" => {
                let cast_expression = handle_cast_expression(node, src.clone());
                parts.push(cast_expression);
            },
            "unary_expression" => {
                let unary_expression = handle_unary_expression(node, src.clone());
                parts.push(unary_expression);
            },
            "conditional_expression" => {
                let conditional_expression = handle_conditional_expression(node, src.clone());
                parts.push(conditional_expression);
            },
            "string_literal" => {
                let string_literal = handle_string_literal(node, src.clone());
                parts.push(string_literal);
            },
            "preproc_defined" => {
                let preproc_defined = handle_preproc_defined(node, src.clone());
                parts.push(preproc_defined);
            },
            "offsetof_expression" => {
                let offsetof_expression = handle_offsetof_expression(node, src.clone());
                parts.push(offsetof_expression);
            },
            "comment" => {
                let comment = handle_comment(node, src.clone());
                utils::push_expr_comment(&mut parts, &comment, node.start_byte(), node.end_byte(), &src);
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
            _ => println!("You shouldn't be here (c inner_binary_expression): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            "pointer_declarator" => {
                let pointer_declarator = handle_pointer_declarator(node, src.clone());
                result += pointer_declarator.as_str();
            },
            "function_declarator" => {
                let function_declarator = handle_function_declarator(node, src.clone());
                result += function_declarator.as_str();
            },
            "array_declarator" => {
                let array_declarator = handle_array_declarator(node, src.clone());
                result += array_declarator.as_str();
            },
            "parenthesized_declarator" => {
                let parenthesized_declarator = handle_parenthesized_declarator(node, src.clone());
                result += parenthesized_declarator.as_str();
            },
            "(" => result += "(",
            ")" => result += ")",
            _ => println!("You shouldn't be here (c parenthesized_declarator): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            "sizeof_expression" => {
                let sizeof_expression = handle_sizeof_expression(node, src.clone());
                result += sizeof_expression.as_str();
            },
            "comma_expression" => {
                let comma_expression = handle_comma_expression(node, src.clone());
                result += comma_expression.as_str();
            },
            "compound_literal_expression" => {
                let compound_literal_expression = handle_compound_literal_expression(node, src.clone());
                result += compound_literal_expression.as_str();
            },
            "string_literal" => {
                let string_literal = handle_string_literal(node, src.clone());
                result += string_literal.as_str();
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
            "subscript_expression" => {
                let subscript_expression = handle_subscript_expression(node, src.clone());
                result += subscript_expression.as_str();
            },
            "true" => result += "true",
            "false" => result += "false",
            "null" => result += "NULL",
            "ERROR" => {
                let error = handle_error(node, src.clone());
                result += format!("{} ", error).as_str();
            },
            "(" => result += "(",
            ")" => result += ")",
            _ => println!("You shouldn't be here (c parenthesized_expression): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            "call_expression" => {
                let call_expression = handle_call_expression(node, src.clone());
                result += call_expression.as_str();
            },
            "binary_expression" => {
                let binary_expression = handle_binary_expression(node, src.clone());
                result += binary_expression.as_str();
            },
            "unary_expression" => {
                let unary_expression = handle_unary_expression(node, src.clone());
                result += unary_expression.as_str();
            },
            "*" => result += "*",
            "&" => result += "&",
            ";" => result += ";",
            _ => println!("You shouldn't be here (c pointer_expression): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_inner_pointer_expression(root: Node, src: String) -> String {
    return handle_pointer_expression(root, src);
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
            "function_declarator" => {
                let function_declarator = handle_function_declarator(node, src.clone());
                parts.push(function_declarator);
            },
            "parenthesized_declarator" => {
                let parenthesized_declarator = handle_parenthesized_declarator(node, src.clone());
                parts.push(parenthesized_declarator);
            },
            "primitive_type" => {
                let primitive_type = handle_primitive_type(node, src.clone());
                parts.push(primitive_type);
            },
            "type_qualifier" => {
                let type_qualifier = handle_type_qualifier(node, src.clone());
                parts.push(type_qualifier);
            },
            "storage_class_specifier" => {
                let storage_class_specifier = handle_storage_class_specifier(node, src.clone());
                parts.push(storage_class_specifier);
            },
            "struct_specifier" => {
                let struct_specifier = handle_struct_specifier(node, src.clone());
                parts.push(struct_specifier);
            },
            "abstract_array_declarator" => {
                let abstract_array_declarator = handle_abstract_array_declarator(node, src.clone());
                parts.push(abstract_array_declarator);
            },
            "abstract_pointer_declarator" => {
                let abstract_pointer_declarator = handle_abstract_pointer_declarator(node, src.clone());
                parts.push(abstract_pointer_declarator);
            },
            "abstract_function_declarator" => {
                let abstract_function_declarator = handle_abstract_function_declarator(node, src.clone());
                parts.push(abstract_function_declarator);
            },
            "sized_type_specifier" => {
                let sized_type_specifier = handle_sized_type_specifier(node, src.clone());
                parts.push(sized_type_specifier);
            },
            "array_declarator" => {
                let array_declarator = handle_array_declarator(node, src.clone());
                parts.push(array_declarator);
            },
            "enum_specifier" => {
                let enum_specifier = handle_enum_specifier(node, src.clone());
                parts.push(enum_specifier);
            },
            "union_specifier" => {
                let union_specifier = handle_union_specifier(node, src.clone());
                parts.push(union_specifier);
            },
            "ERROR" => {
                let error = handle_error(node, src.clone());
                parts.push(error);
            },
            _ => println!("You shouldn't be here (c parameter_declaration): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let mut result = parts.join(" ");
    result = utils::remove_unnecessary_spaces(&result);
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
            "unary_expression" => {
                let unary_expression = handle_unary_expression(node, src.clone());
                result += unary_expression.as_str();
            },
            "char_literal" => {
                let char_literal = handle_char_literal(node, src.clone());
                result += char_literal.as_str();
            },
            "conditional_expression" => {
                let conditional_expression = handle_conditional_expression(node, src.clone());
                result += conditional_expression.as_str();
            },
            _ => println!("You shouldn't be here (c subscript_expression): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
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
            "compound_literal_expression" => {
                let compound_literal_expression = handle_compound_literal_expression(node, src.clone());
                result += compound_literal_expression.as_str();
            },
            "->" => result += "->",
            "." => result += ".",
            _ => println!("You shouldn't be here (c field_expression): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_error(root: Node, src: String) -> String {
    let raw = root.utf8_text(src.as_bytes()).unwrap_or("");
    if root.child_count() == 0 || (!raw.contains("#if") && !raw.contains("#endif")) {
        return raw.to_string();
    }
    let mut parts = Vec::<String>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                parts.push(handle_identifier(node, src.clone()));
            },
            "binary_expression" => {
                parts.push(handle_binary_expression(node, src.clone()));
            },
            "compound_statement" => {
                parts.push(handle_compound_statement(node, src.clone()));
            },
            /*NOTE
                These two become errors when they are used mid-expression.
                This usually breaks the surrounding syntax, so they are simply removed. 
                *This is a destructive action and code behavior **will** change*
            */
            "#ifdef" => {},
            "#endif" => {},
            "ERROR" => {
                parts.push(node.utf8_text(src.as_bytes()).unwrap_or("").to_string());
            },
            "parenthesized_expression" => {
                parts.push(handle_parenthesized_expression(node, src.clone()));
            },
            "primitive_type" => {
                parts.push(handle_primitive_type(node, src.clone()));
            },
            "type_qualifier" => {
                parts.push(handle_type_qualifier(node, src.clone()));
            },
            "switch" => {
                parts.push("switch".to_string());
            },
            "{" => parts.push("{".to_string()),
            "}" => parts.push("}".to_string()),
            ";" => parts.push(";".to_string()),
            "=" => parts.push("=".to_string()),
            "(" => parts.push("(".to_string()),
            ")" => parts.push(")".to_string()),
            "-" => parts.push("-".to_string()),
            ">" => parts.push(">".to_string()),
            "*" => parts.push("*".to_string()),
            "," => parts.push(",".to_string()),
            _ => parts.push(node.utf8_text(src.as_bytes()).unwrap_or("").to_string()),
        }
    }
    return parts.join(" ");
}

fn handle_cast_expression(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "(" => result += "(",
            ")" => result += ")",
            "null" => result += "NULL",
            "true" => result += "true",
            "false" => result += "false",
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
            "cast_expression" => {
                let cast_expression = handle_cast_expression(node, src.clone());
                result += cast_expression.as_str();
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
            "number_literal" => {
                let number_literal = handle_number_literal(node, src.clone());
                result += number_literal.as_str();
            },
            "char_literal" => {
                let char_literal = handle_char_literal(node, src.clone());
                result += char_literal.as_str();
            },
            "string_literal" => {
                let string_literal = handle_string_literal(node, src.clone());
                result += string_literal.as_str();
            },
            "concatenated_string" => {
                let concatenated_string = handle_concatenated_string(node, src.clone());
                result += concatenated_string.as_str();
            },
            "unary_expression" => {
                let unary_expression = handle_unary_expression(node, src.clone());
                result += unary_expression.as_str();
            },
            "binary_expression" => {
                let binary_expression = handle_binary_expression(node, src.clone());
                result += binary_expression.as_str();
            },
            "sizeof_expression" => {
                let sizeof_expression = handle_sizeof_expression(node, src.clone());
                result += sizeof_expression.as_str();
            },
            "conditional_expression" => {
                let conditional_expression = handle_conditional_expression(node, src.clone());
                result += conditional_expression.as_str();
            },
            "update_expression" => {
                let update_expression = handle_update_expression(node, src.clone());
                result += update_expression.as_str();
            },
            _ => println!("You shouldn't be here (c cast_expression): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            "char_literal" => {
                let char_literal = handle_char_literal(node, src.clone());
                parts.push(char_literal);
            },
            "string_literal" => {
                let string_literal = handle_string_literal(node, src.clone());
                parts.push(string_literal);
            },
            "concatenated_string" => {
                let concatenated_string = handle_concatenated_string(node, src.clone());
                parts.push(concatenated_string);
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
            "function_declarator" => {
                let function_declarator = handle_function_declarator(node, src.clone());
                parts.push(function_declarator);
            },
            "parenthesized_declarator" => {
                let parenthesized_declarator = handle_parenthesized_declarator(node, src.clone());
                parts.push(parenthesized_declarator);
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
            "compound_literal_expression" => {
                let compound_literal_expression = handle_compound_literal_expression(node, src.clone());
                parts.push(compound_literal_expression);
            },
            "sizeof_expression" => {
                let sizeof_expression = handle_sizeof_expression(node, src.clone());
                parts.push(sizeof_expression);
            },
            "offsetof_expression" => {
                let offsetof_expression = handle_offsetof_expression(node, src.clone());
                parts.push(offsetof_expression);
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
            "ERROR" => {
                let error = handle_error(node, src.clone());
                parts.push(error);
            },
            "=" => parts.push("=".to_string()),
            "false" => parts.push("false".to_string()),
            "true" => parts.push("true".to_string()),
            "null" => parts.push("NULL".to_string()),
            _ => println!("You shouldn't be here (c init_declarator): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let result = parts.join(" ");
    return result;
}

fn handle_type_descriptor(root: Node, src: String) -> String {
    let mut parts = Vec::<String>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                parts.push(identifier);
            },
            "primitive_type" => {
                let primitive_type = handle_primitive_type(node, src.clone());
                parts.push(primitive_type);
            },
            "type_qualifier" => {
                let type_qualifier = handle_type_qualifier(node, src.clone());
                parts.push(type_qualifier);
            },
            "abstract_pointer_declarator" => {
                let abstract_pointer_declaration = handle_abstract_pointer_declarator(node, src.clone());
                parts.push(abstract_pointer_declaration);
            },
            "abstract_array_declarator" => {
                let abstract_array_declarator = handle_abstract_array_declarator(node, src.clone());
                parts.push(abstract_array_declarator);
            },
            "abstract_function_declarator" => {
                let abstract_function_declarator = handle_abstract_function_declarator(node, src.clone());
                parts.push(abstract_function_declarator);
            },
            "struct_specifier" => {
                let struct_specifier = handle_struct_specifier(node, src.clone());
                parts.push(struct_specifier.trim_end().to_string());
            },
            "union_specifier" => {
                let union_specifier = handle_union_specifier(node, src.clone());
                parts.push(union_specifier.trim_end().to_string());
            },
            "enum_specifier" => {
                let enum_specifier = handle_enum_specifier(node, src.clone());
                parts.push(enum_specifier.trim_end().to_string());
            },
            "sized_type_specifier" => {
                let sized_type_specifier = handle_sized_type_specifier(node, src.clone());
                parts.push(sized_type_specifier);
            },
            "ERROR" => {
                let error = handle_error(node, src.clone());
                parts.push(error);
            },
            _ => println!("You shouldn't be here (c type_descriptor): {}", node.grammar_name()),
        }
    }
    let mut result = parts.join(" ");
    result = utils::remove_pointer_spaces(result);
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
                utils::append_expr_comment(&mut result, &comment, node.start_byte(), node.end_byte(), &src);
            },
            "initializer_list" => {
                let initializer_list = handle_initializer_list(node, src.clone());
                result += initializer_list.as_str();
            },
            "initializer_pair" => {
                let initializer_pair = handle_initializer_pair(node, src.clone());
                result += initializer_pair.as_str();
            },
            "string_literal" => {
                let string_literal = handle_string_literal(node, src.clone());
                result += string_literal.as_str();
            },
            "concatenated_string" => {
                let concatenated_string = handle_concatenated_string(node, src.clone());
                result += concatenated_string.as_str();
            },
            "cast_expression" => {
                let cast_expression = handle_cast_expression(node, src.clone());
                result += cast_expression.as_str();
            },
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
            "field_expression" => {
                let field_expression = handle_field_expression(node, src.clone());
                result += field_expression.as_str();
            },
            "subscript_expression" => {
                let subscript_expression = handle_subscript_expression(node, src.clone());
                result += subscript_expression.as_str();
            },
            "pointer_expression" => {
                let pointer_expression = handle_pointer_expression(node, src.clone());
                result += pointer_expression.as_str();
            },
            "conditional_expression" => {
                let conditional_expression = handle_conditional_expression(node, src.clone());
                result += conditional_expression.as_str();
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(node, src.clone());
                result += parenthesized_expression.as_str();
            },
            "sizeof_expression" => {
                let sizeof_expression = handle_sizeof_expression(node, src.clone());
                result += sizeof_expression.as_str();
            },
            "compound_literal_expression" => {
                let compound_literal_expression = handle_compound_literal_expression(node, src.clone());
                result += compound_literal_expression.as_str();
            },
            "null" => result += "NULL",
            "true" => result += "true",
            "false" => result += "false",
            "{" => result += "{",
            "}" => result += "}",
            "," => result += ", ",
            _ => println!("You shouldn't be here (c initializer_list): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
                let parenthesized_expression = handle_parenthesized_expression(node, src.clone());
                result += parenthesized_expression.as_str();
            },
            "type_descriptor" => {
                let type_descriptor = handle_type_descriptor(node, src.clone());
                result += type_descriptor.as_str();
            },
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                result += format!(" {}", identifier).as_str();
            },
            "field_expression" => {
                let field_expression = handle_field_expression(node, src.clone());
                result += format!(" {}", field_expression).as_str();
            },
            "subscript_expression" => {
                let subscript_expression = handle_subscript_expression(node, src.clone());
                result += format!(" {}", subscript_expression).as_str();
            },
            "pointer_expression" => {
                let pointer_expression = handle_pointer_expression(node, src.clone());
                result += format!(" {}", pointer_expression).as_str();
            },
            "unary_expression" => {
                let unary_expression = handle_unary_expression(node, src.clone());
                result += format!(" {}", unary_expression).as_str();
            },
            "call_expression" => {
                let call_expression = handle_call_expression(node, src.clone());
                result += format!(" {}", call_expression).as_str();
            },
            "string_literal" => {
                let string_literal = handle_string_literal(node, src.clone());
                result += format!(" {}", string_literal).as_str();
            },
            _ => println!("You shouldn't be here (c sizeof_expression): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_type_qualifier(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "const" => result = "const".to_string(),
            "volatile" => result = "volatile".to_string(),
            "restrict" => result = "restrict".to_string(),
            "__restrict" => result = "__restrict".to_string(),
            "__restrict__" => result = "__restrict__".to_string(),
            "_Atomic" => result = "_Atomic".to_string(),
            _ => println!("You shouldn't be here (c type_qualifier): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_abstract_pointer_declarator(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "abstract_pointer_declarator" => {
                let inner = handle_abstract_pointer_declarator(node, src.clone());
                result += inner.as_str();
            },
            "type_qualifier" => {
                let type_qualifier = handle_type_qualifier(node, src.clone());
                result += format!(" {}", type_qualifier).as_str();
            },
            "*" => result += "*",
            _ => println!("You shouldn't be here (c abstract_pointer_declarator): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            "cast_expression" => {
                let cast_expression = handle_cast_expression(node, src.clone());
                parts.push(cast_expression);
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
            "pointer_expression" => {
                let pointer_expression = handle_pointer_expression(node, src.clone());
                parts.push(pointer_expression);
            },
            "sizeof_expression" => {
                let sizeof_expression = handle_sizeof_expression(node, src.clone());
                parts.push(sizeof_expression);
            },
            "conditional_expression" => {
                let conditional_expression = handle_conditional_expression(node, src.clone());
                parts.push(conditional_expression);
            },
            "assignment_expression" => {
                let assignment_expression = handle_assignment_expression(node, src.clone());
                parts.push(assignment_expression);
            },
            "update_expression" => {
                let update_expression = handle_update_expression(node, src.clone());
                parts.push(update_expression);
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(node, src.clone());
                parts.push(parenthesized_expression);
            },
            "comment" => {
                let comment = handle_comment(node, src.clone());
                utils::push_expr_comment(&mut parts, &comment, node.start_byte(), node.end_byte(), &src);
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
            "concatenated_string" => {
                let concatenated_string = handle_concatenated_string(node, src.clone());
                parts.push(concatenated_string);
            },
            "null" => parts.push("NULL".to_string()),
            "true" => parts.push("true".to_string()),
            "false" => parts.push("false".to_string()),
            ":" => parts.push(":".to_string()),
            "?" => parts.push("?".to_string()),
            _ => println!("You shouldn't be here (c conditional_expression): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            "assignment_expression" => {
                let assignment_expression = handle_assignment_expression(node, src.clone());
                result += assignment_expression.as_str();
            },
            "subscript_expression" => {
                let subscript_expression = handle_subscript_expression(node, src.clone());
                result += subscript_expression.as_str();
            },
            "preproc_defined" => {
                let preproc_defined = handle_preproc_defined(node, src.clone());
                result += preproc_defined.as_str();
            },
            "cast_expression" => {
                let cast_expression = handle_cast_expression(node, src.clone());
                result += cast_expression.as_str();
            },
            "unary_expression" => {
                let unary_expression = handle_unary_expression(node, src.clone());
                result += unary_expression.as_str();
            },
            "sizeof_expression" => {
                let sizeof_expression = handle_sizeof_expression(node, src.clone());
                result += sizeof_expression.as_str();
            },
            "char_literal" => {
                let char_literal = handle_char_literal(node, src.clone());
                result += char_literal.as_str();
            },
            "compound_literal_expression" => {
                let compound_literal_expression = handle_compound_literal_expression(node, src.clone());
                result += compound_literal_expression.as_str();
            },
            "true" => result += "true",
            "false" => result += "false",
            "ERROR" => {
                let error = handle_error(node, src.clone());
                result += format!("{} ", error).as_str();
            },
            "!" => result += "!",
            "~" => result += "~",
            "-" => result += "-",
            "+" => result += "+",
            _ => println!("You shouldn't be here (c unary_expression): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            "cast_expression" => {
                let cast_expression = handle_cast_expression(node, src.clone());
                parts.push(cast_expression);
            },
            "call_expression" => {
                let call_expression = handle_call_expression(node, src.clone());
                parts.push(call_expression);
            },
            "sizeof_expression" => {
                let sizeof_expression = handle_sizeof_expression(node, src.clone());
                parts.push(sizeof_expression);
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(node, src.clone());
                parts.push(parenthesized_expression);
            },
            "pointer_declarator" => {
                let pointer_declarator = handle_pointer_declarator(node, src.clone());
                parts.push(pointer_declarator);
            },
            "parenthesized_declarator" => {
                let parenthesized_declarator = handle_parenthesized_declarator(node, src.clone());
                parts.push(parenthesized_declarator);
            },
            "array_declarator" => {
                let array_declarator = handle_array_declarator(node, src.clone());
                parts.push(array_declarator);
            },
            "type_qualifier" => {
                let type_qualifier = handle_type_qualifier(node, src.clone());
                parts.push(type_qualifier);
            },
            "ERROR" => {
                let error = handle_error(node, src.clone());
                parts.push(error);
            },
            "static" => parts.push("static".to_string()),
            "*" => parts.push("*".to_string()),
            "[" => parts.push("[".to_string()),
            "]" => parts.push("]".to_string()),
            _ => println!("You shouldn't be here (c array_declarator): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let mut result = String::new();
    for p in parts {
        if p.is_empty() {
            continue;
        }
        if let (Some(prev), Some(next)) = (result.chars().last(), p.chars().next()) {
            let prev_word = prev.is_ascii_alphanumeric() || prev == '_' || prev == '>';
            let next_word = next.is_ascii_alphanumeric() || next == '_';
            if prev_word && next_word {
                result.push(' ');
            }
        }
        result.push_str(&p);
    }
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
            _ => println!("You shouldn't be here (c string_literal): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            _ => println!("You shouldn't be here (c concatenated_string): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let result = parts.join(" ");
    return result;
}

fn handle_labeled_statement(root: Node, src: String) -> String {
    let same_line = root.start_position().row == root.end_position().row;
    let sep = if same_line { " " } else { "\n\t" };
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "expression_statement" => {
                let expression_statement = handle_expression_statement(node, src.clone());
                if same_line && expression_statement.trim() == ";" {
                    result += ";";
                } else {
                    result += format!("{}{}", sep, expression_statement).as_str();
                }
            },
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                result += identifier.as_str();
            },
            "return_statement" => {
                let return_statement = handle_return_statement(node, src.clone());
                result += format!("{}{}", sep, return_statement).as_str();
            },
            "if_statement" => {
                let if_statement = handle_if_statement(node, src.clone());
                result += format!("{}{}", sep, if_statement).as_str();
            },
            "switch_statement" => {
                let switch_statement = handle_switch_statement(node, src.clone());
                result += format!("{}{}", sep, switch_statement).as_str();
            },
            "while_statement" => {
                let while_statement = handle_while_statement(node, src.clone());
                result += format!("{}{}", sep, while_statement).as_str();
            },
            "for_statement" => {
                let for_statement = handle_for_statement(node, src.clone());
                result += format!("{}{}", sep, for_statement).as_str();
            },
            "do_statement" => {
                let do_statement = handle_do_statement(node, src.clone());
                result += format!("{}{}", sep, do_statement).as_str();
            },
            "goto_statement" => {
                let goto_statement = handle_goto_statement(node, src.clone());
                result += format!("{}{}", sep, goto_statement).as_str();
            },
            "declaration" => {
                let declaration = handle_declaration(node, src.clone());
                result += format!("{}{}", sep, declaration).as_str();
            },
            "compound_statement" => {
                let compound_statement = handle_inner_compound_statement(node, src.clone());
                result += format!("{}{}", sep, compound_statement).as_str();
            },
            "labeled_statement" => {
                let labeled_statement = handle_labeled_statement(node, src.clone());
                result += format!("{}{}", sep, labeled_statement).as_str();
            },
            "comment" => {
                let comment = handle_comment(node, src.clone());
                utils::append_comment_to_result(&mut result, &comment, node.start_byte(), &src, "\n\t", "", false);
            },
            "break_statement" => result += format!("{}break;", sep).as_str(),
            "continue_statement" => result += format!("{}continue;", sep).as_str(),
            ":" => result += ":",
            ";" => result += ";",
            _ => println!("You shouldn't be here (c labeled_statement): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_for_statement(root: Node, src: String) -> String {
    let mut vec = Vec::<String>::new();
    let mut temp = String::new();
    let mut reached_header_end = false;
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => {
                let compound_statement = handle_compound_statement(node, src.clone());
                vec.push(compound_statement);
            },
            "if_statement" => {
                let if_statement = handle_if_statement(node, src.clone());
                vec.push(if_statement);
            },
            "while_statement" => {
                let while_statement = handle_while_statement(node, src.clone());
                vec.push(while_statement);
            },
            "for_statement" => {
                let for_statement = handle_for_statement(node, src.clone());
                vec.push(for_statement);
            },
            "do_statement" => {
                let do_statement = handle_do_statement(node, src.clone());
                vec.push(do_statement);
            },
            "switch_statement" => {
                let switch_statement = handle_switch_statement(node, src.clone());
                vec.push(switch_statement);
            },
            "return_statement" => {
                let return_statement = handle_return_statement(node, src.clone());
                vec.push(return_statement);
            },
            "break_statement" => vec.push("break;".to_string()),
            "continue_statement" => vec.push("continue;".to_string()),
            "goto_statement" => {
                let goto_statement = handle_goto_statement(node, src.clone());
                vec.push(goto_statement);
            },
            "update_expression" => {
                let update_expression = handle_update_expression(node, src.clone());
                if !reached_header_end {
                    temp += update_expression.as_str();
                } else {
                    vec.push(update_expression);
                }
            },
            "assignment_expression" => {
                let assignment_expression = handle_assignment_expression(node, src.clone());
                if !reached_header_end {
                    temp += assignment_expression.as_str();
                } else {
                    vec.push(assignment_expression);
                }
            },
            "binary_expression" => {
                let binary_expression = handle_binary_expression(node, src.clone());
                if !reached_header_end {
                    temp += binary_expression.as_str();
                } else {
                    vec.push(binary_expression);
                }
            },
            "call_expression" => {
                let call_expression = handle_call_expression(node, src.clone());
                if !reached_header_end {
                    temp += call_expression.as_str();
                } else {
                    vec.push(call_expression);
                }
            },
            "unary_expression" => {
                let unary_expression = handle_unary_expression(node, src.clone());
                if !reached_header_end {
                    temp += unary_expression.as_str();
                } else {
                    vec.push(unary_expression);
                }
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(node, src.clone());
                if !reached_header_end {
                    temp += parenthesized_expression.as_str();
                } else {
                    vec.push(parenthesized_expression);
                }
            },
            "field_expression" => {
                let field_expression = handle_field_expression(node, src.clone());
                if !reached_header_end {
                    temp += field_expression.as_str();
                } else {
                    vec.push(field_expression);
                }
            },
            "subscript_expression" => {
                let subscript_expression = handle_subscript_expression(node, src.clone());
                if !reached_header_end {
                    temp += subscript_expression.as_str();
                } else {
                    vec.push(subscript_expression);
                }
            },
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                if !reached_header_end {
                    temp += identifier.as_str();
                } else {
                    vec.push(identifier);
                }
            },
            "number_literal" => {
                let number_literal = handle_number_literal(node, src.clone());
                if !reached_header_end {
                    temp += number_literal.as_str();
                } else {
                    vec.push(number_literal);
                }
            },
            "declaration" => {
                let declaration = handle_declaration(node, src.clone());
                if !reached_header_end {
                    temp += format!("{} ", declaration).as_str();
                } else {
                    vec.push(declaration);
                }
            },
            "expression_statement" => {
                let expression_statement = handle_expression_statement(node, src.clone());
                vec.push(expression_statement);
            },
            "pointer_expression" => {
                let mut pointer_expression = handle_pointer_expression(node, src.clone());
                pointer_expression = utils::remove_dereference_spaces(pointer_expression);
                temp += pointer_expression.as_str();
            },
            "comma_expression" => {
                let comma_expression = handle_comma_expression(node, src.clone());
                temp += comma_expression.as_str();
            },
            "true" => temp += "true",
            "false" => temp += "false",
            ";" => {
                if !reached_header_end {
                    temp += "; ";
                } else {
                    vec.push(";".to_string());
                }
            },
            "(" => temp += "(",
            ")" => {
                reached_header_end = true;
                temp = temp.trim_end().to_string();
                temp += ")";
                vec.push(temp);
                temp = "".to_string();
            },
            "for" => temp += "for ",
            _ => println!("You shouldn't be here (c for_statement): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
                result += format!(" {}", identifier).as_str();
            },
            "field_declaration_list" => {
                let field_declaration_list = handle_field_declaration_list(node, src.clone());
                if result.contains("\n") {
                    result += field_declaration_list.as_str();
                }
                else {
                    result = result.trim_end().to_string();
                    result += format!("\n{}", field_declaration_list).as_str();
                }
            },
            "struct" => result += "struct",
            _ => println!("You shouldn't be here (c struct_specifier): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_type_definition(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
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
            "type_qualifier" => {
                let type_qualifier = handle_type_qualifier(node, src.clone());
                result += format!("{} ", type_qualifier).as_str();
            },
            "sized_type_specifier" => {
                let sized_type_specifier = handle_sized_type_specifier(node, src.clone());
                result += sized_type_specifier.as_str();
            },
            "function_declarator" => {
                let function_declarator = handle_function_declarator(node, src.clone());
                result += format!(" {}", function_declarator).as_str();
            },
            "parenthesized_declarator" => {
                let parenthesized_declarator = handle_parenthesized_declarator(node, src.clone());
                result += format!(" {}", parenthesized_declarator).as_str();
            },
            "array_declarator" => {
                let array_declarator = handle_array_declarator(node, src.clone());
                result += format!(" {}", array_declarator).as_str();
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
                result += pointer_declarator.as_str();
            },
            "enum_specifier" => {
                let enum_specifier = handle_enum_specifier(node, src.clone());
                result = result.trim_end().to_string();
                result += format!(" {}", enum_specifier.trim_end()).as_str();
            },
            ";" => result += ";",
            "," => result += ", ",
            _ => println!("You shouldn't be here (c type_definition): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_field_declaration_list(root: Node, src: String) -> String {
    let mut parts = Vec::<String>::new();
    let mut pending_incomplete_decl = false;
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "field_declaration" => {
                let missing_semi = node_has_missing_semicolon_c(node);
                let mut field_declaration = handle_field_declaration(node, src.clone());
                if field_declaration.contains("*") {
                    field_declaration = utils::remove_pointer_spaces(field_declaration);
                }
                if pending_incomplete_decl {
                    utils::merge_incomplete_decl_into_parts(&mut parts, &field_declaration);
                } else {
                    parts.push(utils::add_all_leading_tabs(field_declaration));
                }
                pending_incomplete_decl = missing_semi;
            },
            "ERROR" => {
                let error = handle_error(node, src.clone());
                if pending_incomplete_decl {
                    utils::merge_incomplete_decl_into_parts(&mut parts, &error);
                    if error.trim_end().ends_with(';') {
                        pending_incomplete_decl = false;
                    }
                } else {
                    parts.push(utils::add_all_leading_tabs(error));
                }
            },
            "comment" => {
                let comment = handle_comment(node, src.clone());
                utils::push_comment_to_parts(&mut parts, &comment, node.start_byte(), &src, true);
            },
            "preproc_ifdef" => {
                pending_incomplete_decl = false;
                let preproc_ifdef = handle_preproc_ifdef(node, src.clone());
                parts.push(preproc_ifdef);
            },
            "preproc_if" => {
                pending_incomplete_decl = false;
                let preproc_if = handle_preproc_if(node, src.clone());
                parts.push(preproc_if);
            },
            "preproc_def" => {
                pending_incomplete_decl = false;
                let preproc_def = handle_preproc_def(node, src.clone());
                parts.push(preproc_def);
            },
            "preproc_call" => {
                pending_incomplete_decl = false;
                let preproc_call = handle_preproc_call(node, src.clone());
                parts.push(preproc_call);
            },
            "{" => {
                pending_incomplete_decl = false;
                parts.push("{".to_string());
            },
            "}" => {
                pending_incomplete_decl = false;
                parts.push("}".to_string());
            },
            ";" => {
                if pending_incomplete_decl && !node.is_missing() {
                    if let Some(last) = parts.last_mut() {
                        last.push(';');
                    }
                    pending_incomplete_decl = false;
                }
            },
            _ => println!("You shouldn't be here (c field_declaration_list): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
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
            "type_qualifier" => {
                let type_qualifier = handle_type_qualifier(node, src.clone());
                result += format!("{} ", type_qualifier).as_str();
            },
            "sized_type_specifier" => {
                let sized_type_specifier = handle_sized_type_specifier(node, src.clone());
                result += format!("{} ", sized_type_specifier).as_str();
            },
            "storage_class_specifier" => {
                let storage_class_specifier = handle_storage_class_specifier(node, src.clone());
                result += format!("{} ", storage_class_specifier).as_str();
            },
            "struct_specifier" => {
                let struct_specifier = handle_struct_specifier(node, src.clone());
                result += format!("{} ", struct_specifier).as_str();
            },
            "union_specifier" => {
                let union_specifier = handle_union_specifier(node, src.clone());
                result += format!("{} ", union_specifier).as_str();
            },
            "enum_specifier" => {
                let enum_specifier = handle_enum_specifier(node, src.clone());
                result += format!("{} ", enum_specifier).as_str();
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
            "parenthesized_declarator" => {
                let parenthesized_declarator = handle_parenthesized_declarator(node, src.clone());
                result += format!("{} ", parenthesized_declarator).as_str();
            },
            "bitfield_clause" => {
                let bitfield_clause = handle_bitfield_clause(node, src.clone());
                result += format!("{} ", bitfield_clause).as_str();
            },
            "ERROR" => {
                let error = handle_error(node, src.clone());
                result += format!("{} ", error).as_str();
            },
            ";" => {
                if !node.is_missing() {
                    result += ";";
                }
            },
            "," => result += ", ",
            _ => println!("You shouldn't be here (c field_declaration): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    result = utils::remove_unnecessary_spaces(&result);
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
            _ => println!("You shouldn't be here (c preproc_def): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                result += identifier.as_str();
            },
            "binary_expression" => {
                let binary_expression = handle_binary_expression(node, src.clone());
                result += binary_expression.as_str();
            },
            "abstract_array_declarator" => {
                let abstract_array_declarator = handle_abstract_array_declarator(node, src.clone());
                result += abstract_array_declarator.as_str();
            },
            "abstract_pointer_declarator" => {
                let abstract_pointer_declarator = handle_abstract_pointer_declarator(node, src.clone());
                result += abstract_pointer_declarator.as_str();
            },
            "*" => result += "*",
            "[" => result += "[",
            "]" => result += "]",
            _ => println!("You shouldn't be here (c abstract_array_declarator): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                parts.push(identifier);
            },
            "long" => parts.push("long".to_string()),
            "short" => parts.push("short".to_string()),
            "unsigned" => parts.push("unsigned".to_string()),
            "signed" => parts.push("signed".to_string()),
            _ => println!("You shouldn't be here (c sized_type_specifier): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let result = parts.join(" ");
    return result;
}

fn handle_enum_specifier(root: Node, src: String) -> String {
    let single_line = root.start_position().row == root.end_position().row;
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                result += format!(" {}", identifier).as_str();
            },
            "primitive_type" => {
                let primitive_type = handle_primitive_type(node, src.clone());
                result += format!(" {}", primitive_type).as_str();
            },
            "sized_type_specifier" => {
                let sized_type_specifier = handle_sized_type_specifier(node, src.clone());
                result += format!(" {}", sized_type_specifier).as_str();
            },
            ":" => result += " :",
            "enumerator_list" => {
                let enumerator_list = handle_enumerator_list(node, src.clone());
                result = result.trim_end().to_string();
                if single_line && !enumerator_list.contains("//") && enumerator_list.len() < 80 {
                    let inline_list = enumerator_list
                        .lines()
                        .map(|l| l.trim())
                        .collect::<Vec<&str>>()
                        .join(" ");
                    result += format!(" {}", inline_list).as_str();
                } else {
                    result += format!("\n{}", enumerator_list).as_str();
                }
            },
            "field_declaration_list" => {
                let field_declaration_list = handle_field_declaration_list(node, src.clone());
                if result.contains("\n") {
                    result += field_declaration_list.as_str();
                }
                else {
                    result = result.trim_end().to_string();
                    result += format!("\n{}", field_declaration_list).as_str();
                }
            },
            "enum" => result += "enum",
            _ => println!("You shouldn't be here (c enum_specifier): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
                result += format!(" {}", identifier).as_str();
            },
            "field_declaration_list" => {
                let field_declaration_list = handle_field_declaration_list(node, src.clone());
                if result.contains("\n") {
                    result += field_declaration_list.as_str();
                }
                else {
                    result = result.trim_end().to_string();
                    result += format!("\n{}", field_declaration_list).as_str();
                }
            },
            "union" => result += "union",
            _ => println!("You shouldn't be here (c union_specifier): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
                let enumerator = handle_enumerator(node, src.clone());
                temp += format!("\t{}", enumerator).as_str();
            },
            "comment" => {
                let comment = handle_comment(node, src.clone());
                if !temp.is_empty() {
                    if let Some(gap) = utils::trailing_comment_gap(node.start_byte(), &src) {
                        let clean = comment
                            .trim_start_matches(utils::COMMENT_OFFSET_SENTINEL)
                            .trim_start_matches(utils::COMMENT_COL0_SENTINEL);
                        if clean.trim_start().starts_with("/*") && utils::has_code_after_on_line(node.end_byte(), &src) {
                            temp += format!("{}{}", gap, clean).as_str();
                        } else {
                            temp += format!("{}{}", gap, clean).as_str();
                            parts.push(temp);
                            temp = String::new();
                        }
                    } else {
                        parts.push(temp);
                        temp = String::new();
                        parts.push(utils::add_all_leading_tabs(comment));
                    }
                } else {
                    utils::push_comment_to_parts(&mut parts, &comment, node.start_byte(), &src, true);
                }
            },
            "preproc_if" => {
                if !temp.is_empty() {
                    parts.push(temp);
                    temp = String::new();
                }
                let preproc_if = handle_preproc_if(node, src.clone());
                parts.push(preproc_if);
            },
            "preproc_ifdef" => {
                if !temp.is_empty() {
                    parts.push(temp);
                    temp = String::new();
                }
                let preproc_ifdef = handle_preproc_ifdef(node, src.clone());
                parts.push(preproc_ifdef);
            },
            "preproc_def" => {
                if !temp.is_empty() {
                    parts.push(temp);
                    temp = String::new();
                }
                let preproc_def = handle_preproc_def(node, src.clone());
                parts.push(preproc_def);
            },
            "," => {
                temp += ",";
                parts.push(temp);
                temp = String::new();
            },
            "{" => parts.push("{".to_string()),
            "}" => {
                if !temp.is_empty() {
                    parts.push(temp);
                    temp = String::new();
                }
                parts.push("}".to_string());
            },
            _ => println!("You shouldn't be here (c enumerator_list): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let result = parts.join("\n");
    return result;
}

fn handle_enumerator(root: Node, src: String) -> String {
    let mut parts = Vec::<String>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => parts.push(handle_identifier(node, src.clone())),
            "=" => parts.push("=".to_string()),
            "number_literal" => parts.push(handle_number_literal(node, src.clone())),
            "char_literal" => parts.push(handle_char_literal(node, src.clone())),
            "binary_expression" => parts.push(handle_binary_expression(node, src.clone())),
            "unary_expression" => parts.push(handle_unary_expression(node, src.clone())),
            "parenthesized_expression" => parts.push(handle_parenthesized_expression(node, src.clone())),
            "cast_expression" => parts.push(handle_cast_expression(node, src.clone())),
            "call_expression" => parts.push(handle_call_expression(node, src.clone())),
            "sizeof_expression" => parts.push(handle_sizeof_expression(node, src.clone())),
            "conditional_expression" => parts.push(handle_conditional_expression(node, src.clone())),
            _ => parts.push(node.utf8_text(src.as_bytes()).unwrap_or("").to_string()),
        }
    }
    return parts.join(" ");
}

fn handle_storage_class_specifier(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "static" => result += "static",
            "extern" => result += "extern",
            "register" => result += "register",
            "auto" => result += "auto",
            "inline" => result += "inline",
            "__inline" => result += "__inline",
            "__inline__" => result += "__inline__",
            "_Thread_local" => result += "_Thread_local",
            "thread_local" => result += "thread_local",
            _ => println!("You shouldn't be here (c storage_class_specifier): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
                parts.push("#endif".to_string());
                last_kind = "#endif";
            },
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                temp += identifier.as_str();
                parts.push(temp);
                temp = String::new();
                last_kind = "identifier";
            },
            "preproc_if" => {
                let preproc_if = handle_preproc_if(node, src.clone());
                parts.push(preproc_if);
                last_kind = "preproc_if";
            },
            "preproc_ifdef" => {
                let preproc_ifdef = handle_preproc_ifdef(node, src.clone());
                parts.push(preproc_ifdef);
                last_kind = "preproc_ifdef";
            },
            "preproc_elif" => {
                let preproc_elif = handle_preproc_elif(node, src.clone());
                parts.push(preproc_elif);
                last_kind = "preproc_elif";
            },
            "preproc_function_def" => {
                let preproc_function_def = handle_preproc_function_def(node, src.clone());
                parts.push(preproc_function_def);
                last_kind = "preproc_function_def";
            },
            "preproc_def" => {
                let preproc_def = handle_preproc_def(node, src.clone());
                parts.push(preproc_def);
                last_kind = "preproc_def";
            },
            "preproc_call" => {
                let preproc_call = handle_preproc_call(node, src.clone());
                parts.push(preproc_call);
                last_kind = "preproc_call";
            },
            "preproc_include" => {
                let preproc_include = handle_preproc_include(node, src.clone());
                parts.push(preproc_include);
                last_kind = "preproc_include";
            },
            "comment" => {
                let comment = handle_comment(node, src.clone());
                if !temp.is_empty() {
                    temp += format!(" {}", comment).as_str();
                    parts.push(temp);
                    temp = String::new();
                } else {
                    utils::push_comment_to_parts(&mut parts, &comment, node.start_byte(), &src, false);
                }
                last_kind = "comment";
            },
            "expression_statement" => {
                let expression_statement = handle_expression_statement(node, src.clone());
                parts.push(expression_statement);
                last_kind = "expression_statement";
            },
            "function_definition" => {
                let function_definition = handle_function_definition(node, src.clone());
                parts.push(function_definition);
                last_kind = "function_definition";
            },
            "type_definition" => {
                let type_definition = handle_type_definition(node, src.clone());
                parts.push(type_definition);
                last_kind = "type_definition";
            },
            "struct_specifier" => {
                let struct_specifier = handle_struct_specifier(node, src.clone());
                parts.push(format!("{};", struct_specifier.trim_end()));
                last_kind = "struct_specifier";
            },
            "union_specifier" => {
                let union_specifier = handle_union_specifier(node, src.clone());
                parts.push(format!("{};", union_specifier.trim_end()));
                last_kind = "union_specifier";
            },
            "enum_specifier" => {
                let enum_specifier = handle_enum_specifier(node, src.clone());
                parts.push(format!("{};", enum_specifier.trim_end()));
                last_kind = "enum_specifier";
            },
            "field_declaration" => {
                let field_declaration = handle_field_declaration(node, src.clone());
                parts.push(format!("\t{}", field_declaration));
                last_kind = "field_declaration";
            },
            "if_statement" => {
                let if_statement = handle_if_statement(node, src.clone());
                parts.push(if_statement);
                last_kind = "if_statement";
            },
            "for_statement" => {
                let for_statement = handle_for_statement(node, src.clone());
                parts.push(for_statement);
                last_kind = "for_statement";
            },
            "while_statement" => {
                let while_statement = handle_while_statement(node, src.clone());
                parts.push(while_statement);
                last_kind = "while_statement";
            },
            "do_statement" => {
                let do_statement = handle_do_statement(node, src.clone());
                parts.push(do_statement);
                last_kind = "do_statement";
            },
            "switch_statement" => {
                let switch_statement = handle_switch_statement(node, src.clone());
                parts.push(switch_statement);
                last_kind = "switch_statement";
            },
            "return_statement" => {
                let return_statement = handle_return_statement(node, src.clone());
                parts.push(return_statement);
                last_kind = "return_statement";
            },
            "break_statement" => {
                parts.push("break;".to_string());
                last_kind = "break_statement";
            },
            "continue_statement" => {
                parts.push("continue;".to_string());
                last_kind = "continue_statement";
            },
            "goto_statement" => {
                let goto_statement = handle_goto_statement(node, src.clone());
                parts.push(goto_statement);
                last_kind = "goto_statement";
            },
            "labeled_statement" => {
                let labeled_statement = handle_labeled_statement(node, src.clone());
                parts.push(labeled_statement);
                last_kind = "labeled_statement";
            },
            "compound_statement" => {
                let compound_statement = handle_compound_statement(node, src.clone());
                parts.push(compound_statement);
                last_kind = "compound_statement";
            },
            "preproc_else" => {
                let preproc_else = handle_preproc_else(node, src.clone());
                parts.push(preproc_else);
                last_kind = "preproc_else";
            },
            "declaration" => {
                let mut declaration = handle_declaration(node, src.clone());
                if declaration.contains("*") {
                    declaration = utils::remove_pointer_spaces(declaration);
                }
                let _ = last_kind;
                parts.push(declaration);
                last_kind = "declaration";
            },
            ";" => (),
            _ => println!("You shouldn't be here (c preproc_ifdef): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
                if !temp.is_empty() {
                    temp += binary_expression.as_str();
                    parts.push(temp);
                    temp = String::new();
                } else {
                    parts.push(binary_expression);
                }
            },
            "unary_expression" => {
                let unary_expression = handle_unary_expression(node, src.clone());
                if !temp.is_empty() {
                    temp += unary_expression.as_str();
                    parts.push(temp);
                    temp = String::new();
                } else {
                    parts.push(unary_expression);
                }
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(node, src.clone());
                if !temp.is_empty() {
                    temp += parenthesized_expression.as_str();
                    parts.push(temp);
                    temp = String::new();
                } else {
                    parts.push(parenthesized_expression);
                }
            },
            "call_expression" => {
                let call_expression = handle_call_expression(node, src.clone());
                if !temp.is_empty() {
                    temp += call_expression.as_str();
                    parts.push(temp);
                    temp = String::new();
                } else {
                    parts.push(call_expression);
                }
            },
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                if !temp.is_empty() {
                    temp += identifier.as_str();
                    parts.push(temp);
                    temp = String::new();
                } else {
                    parts.push(identifier);
                }
            },
            "number_literal" => {
                let number_literal = handle_number_literal(node, src.clone());
                if !temp.is_empty() {
                    temp += number_literal.as_str();
                    parts.push(temp);
                    temp = String::new();
                } else {
                    parts.push(number_literal);
                }
            },
            "preproc_defined" => {
                let preproc_defined = handle_preproc_defined(node, src.clone());
                if !temp.is_empty() {
                    temp += preproc_defined.as_str();
                    parts.push(temp);
                    temp = String::new();
                } else {
                    parts.push(preproc_defined);
                }
            },
            "preproc_include" => {
                let preproc_include = handle_preproc_include(node, src.clone());
                parts.push(preproc_include);
            },
            "preproc_def" => {
                let preproc_def = handle_preproc_def(node, src.clone());
                parts.push(preproc_def);
            },
            "preproc_if" => {
                let preproc_if = handle_preproc_if(node, src.clone());
                parts.push(preproc_if);
            },
            "preproc_ifdef" => {
                let preproc_ifdef = handle_preproc_ifdef(node, src.clone());
                parts.push(preproc_ifdef);
            },
            "preproc_elif" => {
                let preproc_elif = handle_preproc_elif(node, src.clone());
                parts.push(preproc_elif);
            },
            "preproc_else" => {
                let preproc_else = handle_preproc_else(node, src.clone());
                parts.push(preproc_else);
            },
            "preproc_call" => {
                let preproc_call = handle_preproc_call(node, src.clone());
                parts.push(preproc_call);
            },
            "preproc_function_def" => {
                let preproc_function_def = handle_preproc_function_def(node, src.clone());
                parts.push(preproc_function_def);
            },
            "declaration" => {
                let mut declaration = handle_declaration(node, src.clone());
                if declaration.contains("*") {
                    declaration = utils::remove_pointer_spaces(declaration);
                }
                parts.push(declaration);
            },
            "field_declaration" => {
                let field_declaration = handle_field_declaration(node, src.clone());
                parts.push(format!("\t{}", field_declaration));
            },
            "function_definition" => {
                let function_definition = handle_function_definition(node, src.clone());
                parts.push(function_definition);
            },
            "type_definition" => {
                let type_definition = handle_type_definition(node, src.clone());
                parts.push(type_definition);
            },
            "struct_specifier" => {
                let struct_specifier = handle_struct_specifier(node, src.clone());
                parts.push(format!("{};", struct_specifier.trim_end()));
            },
            "union_specifier" => {
                let union_specifier = handle_union_specifier(node, src.clone());
                parts.push(format!("{};", union_specifier.trim_end()));
            },
            "enum_specifier" => {
                let enum_specifier = handle_enum_specifier(node, src.clone());
                parts.push(format!("{};", enum_specifier.trim_end()));
            },
            "expression_statement" => {
                let expression_statement = handle_expression_statement(node, src.clone());
                parts.push(expression_statement);
            },
            "if_statement" => {
                let if_statement = handle_if_statement(node, src.clone());
                parts.push(if_statement);
            },
            "for_statement" => {
                let for_statement = handle_for_statement(node, src.clone());
                parts.push(for_statement);
            },
            "while_statement" => {
                let while_statement = handle_while_statement(node, src.clone());
                parts.push(while_statement);
            },
            "do_statement" => {
                let do_statement = handle_do_statement(node, src.clone());
                parts.push(do_statement);
            },
            "switch_statement" => {
                let switch_statement = handle_switch_statement(node, src.clone());
                parts.push(switch_statement);
            },
            "return_statement" => {
                let return_statement = handle_return_statement(node, src.clone());
                parts.push(return_statement);
            },
            "break_statement" => parts.push("break;".to_string()),
            "continue_statement" => parts.push("continue;".to_string()),
            "goto_statement" => {
                let goto_statement = handle_goto_statement(node, src.clone());
                parts.push(goto_statement);
            },
            "labeled_statement" => {
                let labeled_statement = handle_labeled_statement(node, src.clone());
                parts.push(labeled_statement);
            },
            "compound_statement" => {
                let compound_statement = handle_compound_statement(node, src.clone());
                parts.push(compound_statement);
            },
            "comment" => {
                let comment = handle_comment(node, src.clone());
                if !temp.is_empty() {
                    temp += format!(" {}", comment).as_str();
                    parts.push(temp);
                    temp = String::new();
                } else {
                    utils::push_comment_to_parts(&mut parts, &comment, node.start_byte(), &src, false);
                }
            },
            "\n" => (),
            ";" => (),
            _ => println!("You shouldn't be here (c preproc_if): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
                result.pop();
                result += format!("{} ", preproc_params).as_str();
            },
            "preproc_arg" => {
                result += node.utf8_text(src.as_bytes()).unwrap();
            },
            _ =>  println!("You shouldn't be here (c preproc_function_def): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            "comment" => {
                let comment = handle_comment(node, src.clone());
                utils::push_comment_to_parts(&mut parts, &comment, node.start_byte(), &src, false);
            },
            "expression_statement" => {
                let expression_statement = handle_expression_statement(node, src.clone());
                parts.push(expression_statement);
            },
            "declaration" => {
                let mut declaration = handle_declaration(node, src.clone());
                if declaration.contains("*") {
                    declaration = utils::remove_pointer_spaces(declaration);
                }
                parts.push(declaration);
            },
            "field_declaration" => {
                let field_declaration = handle_field_declaration(node, src.clone());
                parts.push(format!("\t{}", field_declaration));
            },
            "function_definition" => {
                let function_definition = handle_function_definition(node, src.clone());
                parts.push(function_definition);
            },
            "type_definition" => {
                let type_definition = handle_type_definition(node, src.clone());
                parts.push(type_definition);
            },
            "struct_specifier" => {
                let struct_specifier = handle_struct_specifier(node, src.clone());
                parts.push(format!("{};", struct_specifier.trim_end()));
            },
            "union_specifier" => {
                let union_specifier = handle_union_specifier(node, src.clone());
                parts.push(format!("{};", union_specifier.trim_end()));
            },
            "enum_specifier" => {
                let enum_specifier = handle_enum_specifier(node, src.clone());
                parts.push(format!("{};", enum_specifier.trim_end()));
            },
            "if_statement" => {
                let if_statement = handle_if_statement(node, src.clone());
                parts.push(if_statement);
            },
            "for_statement" => {
                let for_statement = handle_for_statement(node, src.clone());
                parts.push(for_statement);
            },
            "while_statement" => {
                let while_statement = handle_while_statement(node, src.clone());
                parts.push(while_statement);
            },
            "do_statement" => {
                let do_statement = handle_do_statement(node, src.clone());
                parts.push(do_statement);
            },
            "switch_statement" => {
                let switch_statement = handle_switch_statement(node, src.clone());
                parts.push(switch_statement);
            },
            "break_statement" => parts.push("break;".to_string()),
            "continue_statement" => parts.push("continue;".to_string()),
            "goto_statement" => {
                let goto_statement = handle_goto_statement(node, src.clone());
                parts.push(goto_statement);
            },
            "compound_statement" => {
                let compound_statement = handle_compound_statement(node, src.clone());
                parts.push(compound_statement);
            },
            "preproc_include" => {
                let preproc_include = handle_preproc_include(node, src.clone());
                parts.push(preproc_include);
            },
            "preproc_def" => {
                let preproc_def = handle_preproc_def(node, src.clone());
                parts.push(preproc_def);
            },
            "preproc_if" => {
                let preproc_if = handle_preproc_if(node, src.clone());
                parts.push(preproc_if);
            },
            "preproc_ifdef" => {
                let preproc_ifdef = handle_preproc_ifdef(node, src.clone());
                parts.push(preproc_ifdef);
            },
            "preproc_elif" => {
                let preproc_elif = handle_preproc_elif(node, src.clone());
                parts.push(preproc_elif);
            },
            "preproc_call" => {
                let preproc_call = handle_preproc_call(node, src.clone());
                parts.push(preproc_call);
            },
            "binary_expression" => {
                let binary_expression = handle_binary_expression(node, src.clone());
                parts.push(binary_expression);
            },
            "preproc_defined" => {
                let preproc_defined = handle_preproc_defined(node, src.clone());
                parts.push(preproc_defined);
            },
            "preproc_function_def" => {
                let preproc_function_def = handle_preproc_function_def(node, src.clone());
                parts.push(preproc_function_def);
            },
            ";" => (),
            _ => println!("You shouldn't be here (c preproc_else): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            _ => println!("You shouldn't be here (c variadic_parameter): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_comma_expression(root: Node, src: String) -> String {
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
            "assignment_expression" => {
                let assignment_expression = handle_assignment_expression(node, src.clone());
                result += assignment_expression.as_str();
            },
            "call_expression" => {
                let call_expression = handle_call_expression(node, src.clone());
                result += call_expression.as_str();
            },
            "binary_expression" => {
                let binary_expression = handle_binary_expression(node, src.clone());
                result += binary_expression.as_str();
            },
            "unary_expression" => {
                let unary_expression = handle_unary_expression(node, src.clone());
                result += unary_expression.as_str();
            },
            "pointer_expression" => {
                let pointer_expression = handle_pointer_expression(node, src.clone());
                result += pointer_expression.as_str();
            },
            "comma_expression" => {
                let comma_expression = handle_comma_expression(node, src.clone());
                result += comma_expression.as_str();
            },
            "update_expression" => {
                let update_expression = handle_update_expression(node, src.clone());
                result += update_expression.as_str();
            },
            "compound_literal_expression" => {
                let compound_literal_expression = handle_compound_literal_expression(node, src.clone());
                result += compound_literal_expression.as_str();
            },
            "," => result += ", ",
            _ => println!("You shouldn't be here (c comma_expression): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            "..." => result += "...",
            _ => println!("You shouldn't be here (c preproc_params): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_preproc_call(root: Node, src: String) -> String {
    let mut parts = Vec::<String>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "preproc_directive" => {
                parts.push(node.utf8_text(src.as_bytes()).unwrap().trim().to_string());
            },
            "preproc_arg" => {
                parts.push(node.utf8_text(src.as_bytes()).unwrap().trim().to_string());
            },
            "comment" => {
                parts.push(handle_comment(node, src.clone()));
            },
            _ => println!("You shouldn't be here (c preproc_call): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return parts.join(" ");
}

fn handle_do_statement(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "do" => result += "do ",
            "while" => result += " while ",
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
            _ => println!("You shouldn't be here (c do_statement): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_inner_field_expression(root: Node, src: String) -> String {
    return handle_field_expression(root, src);
}

fn handle_parameter_list(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "(" => result += "(",
            ")" => result += ")",
            "," => result += ", ",
            "parameter_declaration" => {
                let mut parameter_declaration = handle_parameter_declaration(node, src.clone());
                parameter_declaration = utils::remove_pointer_spaces(parameter_declaration);
                result += parameter_declaration.as_str();
            },
            "variadic_parameter" => {
                let variadic_parameter = handle_variadic_parameter(node, src.clone());
                result += variadic_parameter.as_str();
            },
            "comment" => {
                let comment = handle_comment(node, src.clone());
                utils::append_expr_comment(&mut result, &comment, node.start_byte(), node.end_byte(), &src);
            },
            _ => println!("You shouldn't be here (c parameter_list): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_initializer_pair(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "field_designator" => {
                let field_designator = handle_field_designator(node, src.clone());
                result += field_designator.as_str();
            },
            "subscript_designator" => {
                let subscript_designator = handle_subscript_designator(node, src.clone());
                result += subscript_designator.as_str();
            },
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
            "string_literal" => {
                let string_literal = handle_string_literal(node, src.clone());
                result += string_literal.as_str();
            },
            "concatenated_string" => {
                let concatenated_string = handle_concatenated_string(node, src.clone());
                result += concatenated_string.as_str();
            },
            "binary_expression" => {
                let binary_expression = handle_binary_expression(node, src.clone());
                result += binary_expression.as_str();
            },
            "unary_expression" => {
                let unary_expression = handle_unary_expression(node, src.clone());
                result += unary_expression.as_str();
            },
            "cast_expression" => {
                let cast_expression = handle_cast_expression(node, src.clone());
                result += cast_expression.as_str();
            },
            "call_expression" => {
                let call_expression = handle_call_expression(node, src.clone());
                result += call_expression.as_str();
            },
            "field_expression" => {
                let field_expression = handle_field_expression(node, src.clone());
                result += field_expression.as_str();
            },
            "subscript_expression" => {
                let subscript_expression = handle_subscript_expression(node, src.clone());
                result += subscript_expression.as_str();
            },
            "pointer_expression" => {
                let pointer_expression = handle_pointer_expression(node, src.clone());
                result += pointer_expression.as_str();
            },
            "conditional_expression" => {
                let conditional_expression = handle_conditional_expression(node, src.clone());
                result += conditional_expression.as_str();
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(node, src.clone());
                result += parenthesized_expression.as_str();
            },
            "sizeof_expression" => {
                let sizeof_expression = handle_sizeof_expression(node, src.clone());
                result += sizeof_expression.as_str();
            },
            "initializer_list" => {
                let initializer_list = handle_initializer_list(node, src.clone());
                result += initializer_list.as_str();
            },
            "compound_literal_expression" => {
                let compound_literal_expression = handle_compound_literal_expression(node, src.clone());
                result += compound_literal_expression.as_str();
            },
            "null" => result += "NULL",
            "true" => result += "true",
            "false" => result += "false",
            "=" => result += " = ",
            ":" => result += ": ",
            _ => println!("You shouldn't be here (c initializer_pair): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_field_designator(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "." => result += ".",
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                result += identifier.as_str();
            },
            _ => println!("You shouldn't be here (c field_designator): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_subscript_designator(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "[" => result += "[",
            "]" => result += "]",
            "number_literal" => result += handle_number_literal(node, src.clone()).as_str(),
            "identifier" => result += handle_identifier(node, src.clone()).as_str(),
            "binary_expression" => result += handle_binary_expression(node, src.clone()).as_str(),
            "char_literal" => result += handle_char_literal(node, src.clone()).as_str(),
            _ => println!("You shouldn't be here (c subscript_designator): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_compound_literal_expression(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "(" => result += "(",
            ")" => result += ")",
            "type_descriptor" => result += handle_type_descriptor(node, src.clone()).as_str(),
            "initializer_list" => result += handle_initializer_list(node, src.clone()).as_str(),
            _ => println!("You shouldn't be here (c compound_literal_expression): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_offsetof_expression(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "offsetof" => result += "offsetof",
            "(" => result += "(",
            ")" => result += ")",
            "," => result += ", ",
            "type_descriptor" => result += handle_type_descriptor(node, src.clone()).as_str(),
            "identifier" => result += handle_identifier(node, src.clone()).as_str(),
            "field_expression" => result += handle_field_expression(node, src.clone()).as_str(),
            _ => println!("You shouldn't be here (c offsetof_expression): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_bitfield_clause(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            ":" => result += ": ",
            "number_literal" => result += handle_number_literal(node, src.clone()).as_str(),
            "identifier" => result += handle_identifier(node, src.clone()).as_str(),
            "binary_expression" => result += handle_binary_expression(node, src.clone()).as_str(),
            "parenthesized_expression" => result += handle_parenthesized_expression(node, src.clone()).as_str(),
            _ => println!("You shouldn't be here (c bitfield_clause): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_abstract_function_declarator(root: Node, src: String) -> String {
    let mut parts = Vec::<String>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "parameter_list" => parts.push(handle_parameter_list(node, src.clone())),
            "abstract_parenthesized_declarator" => parts.push(handle_abstract_parenthesized_declarator(node, src.clone())),
            _ => println!("You shouldn't be here (c abstract_function_declarator): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return parts.join("");
}

fn handle_abstract_parenthesized_declarator(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "(" => result += "(",
            ")" => result += ")",
            "abstract_pointer_declarator" => result += handle_abstract_pointer_declarator(node, src.clone()).as_str(),
            "abstract_function_declarator" => result += handle_abstract_function_declarator(node, src.clone()).as_str(),
            "abstract_array_declarator" => result += handle_abstract_array_declarator(node, src.clone()).as_str(),
            _ => println!("You shouldn't be here (c abstract_parenthesized_declarator): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_macro_type_specifier(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => result += handle_identifier(node, src.clone()).as_str(),
            "(" => result += "(",
            ")" => result += ")",
            "type_descriptor" => result += handle_type_descriptor(node, src.clone()).as_str(),
            _ => println!("You shouldn't be here (c macro_type_specifier): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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

fn handle_gnu_asm_expression(root: Node, src: String) -> String {
    return root.utf8_text(src.as_bytes()).unwrap().to_string();
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
                result += format!(" {}", identifier).as_str();
            },
            _ => println!("You shouldn't be here (c preproc_defined): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    result = result.replace("( ", "(");
    return result;
}

fn handle_preproc_elif(root: Node, src: String) -> String {
    let mut parts = Vec::<String>::new();
    let mut temp = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "#elif" => temp += "#elif ",
            "\n" => (),
            "binary_expression" => {
                let binary_expression = handle_binary_expression(node, src.clone());
                if !temp.is_empty() {
                    temp += binary_expression.as_str();
                    parts.push(temp);
                    temp = String::new();
                } else {
                    parts.push(binary_expression);
                }
            },
            "unary_expression" => {
                let unary_expression = handle_unary_expression(node, src.clone());
                if !temp.is_empty() {
                    temp += unary_expression.as_str();
                    parts.push(temp);
                    temp = String::new();
                } else {
                    parts.push(unary_expression);
                }
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(node, src.clone());
                if !temp.is_empty() {
                    temp += parenthesized_expression.as_str();
                    parts.push(temp);
                    temp = String::new();
                } else {
                    parts.push(parenthesized_expression);
                }
            },
            "call_expression" => {
                let call_expression = handle_call_expression(node, src.clone());
                if !temp.is_empty() {
                    temp += call_expression.as_str();
                    parts.push(temp);
                    temp = String::new();
                } else {
                    parts.push(call_expression);
                }
            },
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                if !temp.is_empty() {
                    temp += identifier.as_str();
                    parts.push(temp);
                    temp = String::new();
                } else {
                    parts.push(identifier);
                }
            },
            "number_literal" => {
                let number_literal = handle_number_literal(node, src.clone());
                if !temp.is_empty() {
                    temp += number_literal.as_str();
                    parts.push(temp);
                    temp = String::new();
                } else {
                    parts.push(number_literal);
                }
            },
            "preproc_defined" => {
                let preproc_defined = handle_preproc_defined(node, src.clone());
                if !temp.is_empty() {
                    temp += preproc_defined.as_str();
                    parts.push(temp);
                    temp = String::new();
                } else {
                    parts.push(preproc_defined);
                }
            },
            "preproc_def" => {
                let preproc_def = handle_preproc_def(node, src.clone());
                parts.push(preproc_def);
            },
            "preproc_include" => {
                let preproc_include = handle_preproc_include(node, src.clone());
                parts.push(preproc_include);
            },
            "preproc_if" => {
                let preproc_if = handle_preproc_if(node, src.clone());
                parts.push(preproc_if);
            },
            "preproc_ifdef" => {
                let preproc_ifdef = handle_preproc_ifdef(node, src.clone());
                parts.push(preproc_ifdef);
            },
            "preproc_elif" => {
                let preproc_elif = handle_preproc_elif(node, src.clone());
                parts.push(preproc_elif);
            },
            "preproc_else" => {
                let preproc_else = handle_preproc_else(node, src.clone());
                parts.push(preproc_else);
            },
            "preproc_call" => {
                let preproc_call = handle_preproc_call(node, src.clone());
                parts.push(preproc_call);
            },
            "preproc_function_def" => {
                let preproc_function_def = handle_preproc_function_def(node, src.clone());
                parts.push(preproc_function_def);
            },
            "declaration" => {
                let mut declaration = handle_declaration(node, src.clone());
                if declaration.contains("*") {
                    declaration = utils::remove_pointer_spaces(declaration);
                }
                parts.push(declaration);
            },
            "function_definition" => {
                let function_definition = handle_function_definition(node, src.clone());
                parts.push(function_definition);
            },
            "type_definition" => {
                let type_definition = handle_type_definition(node, src.clone());
                parts.push(type_definition);
            },
            "expression_statement" => {
                let expression_statement = handle_expression_statement(node, src.clone());
                parts.push(expression_statement);
            },
            "return_statement" => {
                let return_statement = handle_return_statement(node, src.clone());
                parts.push(return_statement);
            },
            "if_statement" => {
                let if_statement = handle_if_statement(node, src.clone());
                parts.push(if_statement);
            },
            "comment" => {
                let comment = handle_comment(node, src.clone());
                if !temp.is_empty() {
                    temp += format!(" {}", comment).as_str();
                    parts.push(temp);
                    temp = String::new();
                } else {
                    utils::push_comment_to_parts(&mut parts, &comment, node.start_byte(), &src, false);
                }
            },
            ";" => (),
            _ => println!("You shouldn't be here (c preproc_elif): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return parts.join("\n");
}

//NOTE Technically only C++, but some compilers support it, so just in case. 
fn handle_raw_string_literal(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "raw_string_content" => result += node.utf8_text(src.as_bytes()).unwrap(),
            "raw_string_delimiter" => result += node.utf8_text(src.as_bytes()).unwrap(),
            "R\"" => result += "R\"",
            "\"" => result += "\"",
            "(" => result += "(",
            ")" => result += ")",
            _ => println!("You shouldn't be here (c raw_string_literal): {}: {}", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_declaration_list(root: Node, src: String) -> String {
    let mut parts = Vec::<String>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "{" => parts.push("{".to_string()),
            "}" => parts.push("}".to_string()),
            ";" => (),
            "comment" => {
                let comment = handle_comment(node, src.clone());
                utils::push_comment_to_parts(&mut parts, &comment, node.start_byte(), &src, false);
            },
            "declaration" => {
                let mut declaration = handle_declaration(node, src.clone());
                if declaration.contains("*") {
                    declaration = utils::remove_pointer_spaces(declaration);
                }
                parts.push(declaration);
            },
            "function_definition" => {
                let function_definition = handle_function_definition(node, src.clone());
                parts.push(format!("{}\n", function_definition));
            },
            "type_definition" => {
                let td = handle_type_definition(node, src.clone());
                if td.contains('{') {
                    parts.push(format!("{}\n", td));
                } else {
                    parts.push(td);
                }
            },
            "struct_specifier" => parts.push(format!("{};\n", handle_struct_specifier(node, src.clone()).trim_end())),
            "union_specifier" => parts.push(format!("{};\n", handle_union_specifier(node, src.clone()).trim_end())),
            "enum_specifier" => parts.push(format!("{};\n", handle_enum_specifier(node, src.clone()).trim_end())),
            "preproc_include" => parts.push(handle_preproc_include(node, src.clone())),
            "preproc_def" => parts.push(handle_preproc_def(node, src.clone())),
            "preproc_function_def" => parts.push(handle_preproc_function_def(node, src.clone())),
            "preproc_call" => parts.push(handle_preproc_call(node, src.clone())),
            "preproc_ifdef" => parts.push(handle_preproc_ifdef(node, src.clone())),
            "preproc_if" => parts.push(handle_preproc_if(node, src.clone())),
            "preproc_elif" => parts.push(handle_preproc_elif(node, src.clone())),
            _ => println!("You shouldn't be here (c declaration_list): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let result = parts.join("\n");
    return result;
}

