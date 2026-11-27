use std::borrow::Cow;
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

pub fn traverse_c_ast(ast: &Tree, src: &str, style: utils::Style) -> String {
    let root = ast.root_node();
    let mut result = String::new();
    let mut last_group_kind = "";
    let mut pending_incomplete_decl = false;
    let mut skip_until_byte: usize = 0;
    for child in root.children(&mut root.walk()) {
        if child.start_byte() < skip_until_byte {
            continue;
        }
        match child.grammar_name() {
            "preproc_include" => {
                pending_incomplete_decl = false;
                if last_group_kind != "preproc_include" { result += "\n"; }
                let preproc_include = handle_preproc_include(child, src);
                result += format!("{}\n", preproc_include).as_str();
                last_group_kind = "preproc_include";
            },
            "declaration" => {
                let missing_semi = node_has_missing_semicolon_c(child);
                let mut declaration = handle_declaration(child, src);
                if declaration.contains("*") { declaration = utils::remove_pointer_spaces(&declaration); }
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
                if last_group_kind != "function_definition" { result += "\n"; }
                let function_definition = handle_function_definition(child, src);
                result += format!("{}\n\n", function_definition).as_str();
                last_group_kind = "function_definition";
            },
            "expression_statement" => {
                let missing_semi = node_has_missing_semicolon_c(child);
                let expression_statement = handle_expression_statement(child, src);
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
                let compound_statement = handle_compound_statement(child, src);
                result += format!("{}\n\n", compound_statement).as_str();
                last_group_kind = "compound_statement";
            },
            "if_statement" => {
                let if_statement = handle_if_statement(child, src);
                result += format!("{}\n\n", if_statement).as_str();
                last_group_kind = "if_statement";
            },
            "switch_statement" => {
                let switch_statement = handle_switch_statement(child, src);
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
                let return_statement = handle_return_statement(child, src);
                result += format!("{}\n\n", return_statement).as_str();
                last_group_kind = "return_statement";
            },
            "comment" => {
                let comment = handle_comment(child, src);
                if !result.trim().is_empty() && utils::trailing_comment_gap(child.start_byte(), src).is_some() {
                    utils::append_comment_to_result(&mut result, &comment, child.start_byte(), src, "", "\n", true);
                } else {
                    if last_group_kind == "declaration" || last_group_kind == "preproc_include" {
                        result += "\n";
                    }
                    result += format!("{}\n", comment).as_str();
                    last_group_kind = "comment";
                }
            },
            "ERROR" => {
                let error = handle_error(child, src);
                result += format!("{}\n\n", error).as_str();
                last_group_kind = "ERROR";
            },
            "type_definition" => {
                if last_group_kind.contains("preproc") { result += "\n"; }
                let type_definition = handle_type_definition(child, src);
                result += format!("{}\n\n", type_definition).as_str();
                last_group_kind = "type_definition";
            },
            "struct_specifier" => {
                let struct_specifier = handle_struct_specifier(child, src);
                result += format!("{};\n\n", struct_specifier.trim_end()).as_str();
                last_group_kind = "struct_specifier";
            },
            "union_specifier" => {
                let union_specifier = handle_union_specifier(child, src);
                result += format!("{};\n\n", union_specifier.trim_end()).as_str();
                last_group_kind = "union_specifier";
            },
            "enum_specifier" => {
                let enum_specifier = handle_enum_specifier(child, src);
                result += format!("{};\n\n", enum_specifier.trim_end()).as_str();
                last_group_kind = "enum_specifier";
            },
            "preproc_def" => {
                if !last_group_kind.is_empty()
                    && last_group_kind != "comment"
                    && last_group_kind != "declaration"
                    && (!last_group_kind.contains("preproc") || last_group_kind == "preproc_include" || last_group_kind == "preproc_call")
                {
                    result += "\n";
                }
                let (preproc_def, end_b) = utils::consume_define_node(child, src);
                skip_until_byte = end_b;
                result += format!("{}\n", preproc_def).as_str();
                last_group_kind = "preproc_def";
            },
            "preproc_ifdef" => {
                let preproc_ifdef = handle_preproc_ifdef(child, src);
                result += format!("{}\n\n", preproc_ifdef).as_str();
                last_group_kind = "preproc_ifdef";
            },
            "preproc_if" => {
                let preproc_if = handle_preproc_if(child, src);
                result += format!("{}\n\n", preproc_if).as_str();
                last_group_kind = "preproc_if";
            },
            "preproc_elif" => {
                let preproc_elif = handle_preproc_elif(child, src);
                result += format!("{}\n\n", preproc_elif).as_str();
                last_group_kind = "preproc_elif";
            },
            "preproc_function_def" => {
                if !last_group_kind.is_empty()
                    && last_group_kind != "comment"
                    && last_group_kind != "declaration"
                    && (!last_group_kind.contains("preproc") || last_group_kind == "preproc_include" || last_group_kind == "preproc_call")
                {
                    result += "\n";
                }
                let (preproc_function_def, end_b) = utils::consume_define_node(child, src);
                skip_until_byte = end_b;
                result += format!("{}\n", preproc_function_def).as_str();
                last_group_kind = "preproc_function_def";
            },
            "preproc_call" => {
                let preproc_call = handle_preproc_call(child, src);
                result += format!("{}\n\n", preproc_call).as_str();
                last_group_kind = "preproc_call";
            },
            "sized_type_specifier" => {
                let sized_type_specifier = handle_sized_type_specifier(child, src);
                result += format!("{}\n\n", sized_type_specifier).as_str();
                last_group_kind = "sized_type_specifier";
            },
            "declaration_list" => {
                let declaration_list = handle_declaration_list(child, src);
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
    result = utils::sort_include_groups(&result);
    utils::format_else_lines(&mut result, &style);
    utils::close_empty_curly_brace_blocks(&mut result);
    result = utils::fix_stars(&result);
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
        let cur_slice = &src[rem_start_byte..cur_end_byte].trim_end();
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
    let tree = c_format::parse_existing_c_file(&wrapped);
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

    let formatted_compound = handle_compound_statement(compound, &wrapped);
    let inner = formatted_compound
        .strip_prefix("{\n")
        .unwrap_or(&formatted_compound)
        .strip_suffix('}')
        .unwrap_or(&formatted_compound);
    let first_formatted = first_line.trim();
    Some((format!("\t{}\n{}", first_formatted, inner), end_idx))
}

fn handle_compound_statement(root: Node, src: &str) -> String {
    let mut result = String::new();
    let children: Vec<Node> = root.children(&mut root.walk()).collect();
    let mut idx = 0usize;
    while idx < children.len() {
        let node = children[idx];
        if matches!(
            node.grammar_name(),
            "function_definition" | "declaration" | "expression_statement" | "ERROR"
        ) {
            if let Some((split_out, new_end_idx)) = try_split_leading_statement_macro_c(&children, idx, src) {
                result += &split_out;
                idx = new_end_idx + 1;
                continue;
            }
        }
        idx += 1;
        match node.grammar_name() {
            "preproc_include" => {
                let preproc_include = handle_preproc_include(node, src);
                result += format!("\t{}\n", preproc_include).as_str();
            },
            "declaration" => {
                let mut declaration = handle_declaration(node, src);
                declaration = utils::add_all_leading_tabs(&declaration);
                result += format!("{}\n", declaration).as_str();
            },
            "function_definition" => {
                let function_definition = handle_function_definition(node, src);
                result += format!("\t{}\n", function_definition).as_str();
            },
            "expression_statement" => {
                let expression_statement = handle_expression_statement(node, src);
                result += format!("\t{}\n", expression_statement).as_str();
            },
            "compound_statement" => {
                let compound_statement = handle_inner_compound_statement(node, src);
                result += format!("\t{}\n", compound_statement).as_str();
            },
            "if_statement" => {
                let mut if_statement = handle_if_statement(node, src);
                if_statement = utils::add_all_leading_tabs(&if_statement);
                result += format!("{}\n", if_statement).as_str();
            },
            "switch_statement" => {
                let mut switch_statement = handle_switch_statement(node, src);
                switch_statement = utils::remove_blank_lines(switch_statement.split("\n").collect::<Vec<&str>>());
                switch_statement = utils::add_all_leading_tabs(&switch_statement);
                result += format!("{}\n", switch_statement).as_str();
            },
            "case_statement" => {
                let case_statement = handle_case_statement(node, src);
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
                let goto_statement = handle_goto_statement(node, src);
                result += format!("\t{}\n", goto_statement).as_str();
            },
            "primitive_type" => {
                let primitive_type = handle_primitive_type(node, src);
                result += format!("\t{}\n", primitive_type).as_str();
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(node, src);
                result += format!("\t{}\n", parenthesized_expression).as_str();
            },
            "identifier" => {
                let identifier = handle_identifier(node, src);
                result += format!("\t{}\n", identifier).as_str();
            },
            "return_statement" => {
                let return_statement = handle_return_statement(node, src);
                result += format!("\t{}\n", return_statement).as_str();
            },
            "comment" => {
                let comment = handle_comment(node, src);
                utils::append_comment_to_result(&mut result, &comment, node.start_byte(), src, "\t", "\n", true);
            },
            "labeled_statement" => {
                let labeled_statement = handle_labeled_statement(node, src);
                result += format!("\t{}\n", labeled_statement).as_str();
            },
            "while_statement" => {
                let mut while_statement = handle_while_statement(node, src);
                while_statement = utils::add_all_leading_tabs(&while_statement);
                result += format!("{}\n", while_statement).as_str();
            },
            "for_statement" => {
                let mut for_statement = handle_for_statement(node, src);
                for_statement = utils::add_all_leading_tabs(&for_statement);
                result += format!("{}\n", for_statement).as_str();
            },
            "preproc_def" => {
                let (preproc_def, end_b) = utils::consume_define_node(node, src);
                while idx < children.len() && children[idx].start_byte() < end_b {
                    idx += 1;
                }
                result += format!("{}\n", preproc_def).as_str();
            },
            "type_definition" => {
                let type_definition = handle_type_definition(node, src);
                result += format!("\t{}\n", type_definition).as_str();
            },
            "struct_specifier" => {
                let struct_specifier = handle_struct_specifier(node, src);
                result += format!("\t{}\n", struct_specifier).as_str();
            },
            "union_specifier" => {
                let union_specifier = handle_union_specifier(node, src);
                result += format!("\t{}\n", union_specifier).as_str();
            },
            "enum_specifier" => {
                let enum_specifier = handle_enum_specifier(node, src);
                result += format!("\t{}\n", enum_specifier).as_str();
            },
            "init_declarator" => {
                let init_declarator = handle_init_declarator(node, src);
                result += format!("\t{}\n", init_declarator).as_str();
            },
            "ERROR" => {
                let error = handle_error(node, src);
                result += format!("\t{}\n", error).as_str();
            },
            "parameter_list" => {
                dbg!("Here (parameter_list)");
                let parameter_list = handle_parameter_list(node, src);
                result += format!("\t{}\n", parameter_list).as_str();
            },
            "preproc_ifdef" => {
                let preproc_ifdef = handle_preproc_ifdef(node, src);
                result += format!("\t{}\n", preproc_ifdef).as_str();
            },
            "preproc_if" => {
                let mut preproc_if = handle_preproc_if(node, src);
                preproc_if = utils::add_all_leading_tabs(&preproc_if);
                result += format!("{}\n", preproc_if).as_str();
            },
            "preproc_function_def" => {
                let (preproc_function_def, end_b) = utils::consume_define_node(node, src);
                while idx < children.len() && children[idx].start_byte() < end_b {
                    idx += 1;
                }
                result += format!("{}\n", preproc_function_def).as_str();
            },
            "preproc_call" => {
                let preproc_call = handle_preproc_call(node, src);
                result += format!("\t{}\n", preproc_call).as_str();
            },
            "do_statement" => {
                let mut do_statement = handle_do_statement(node, src);
                do_statement = utils::add_all_leading_tabs(&do_statement);
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

fn handle_preproc_include(root: Node, src: &str) -> String {
    let mut vec: Vec<&str> = Vec::new();
    for node in root.children(&mut root.walk()) {
        vec.push(node.utf8_text(src.as_bytes()).unwrap_or(""));
    }
    vec.join(" ")
}

fn handle_declaration(root: Node, src: &str) -> String {
    let mut result: String;
    let mut parts = Vec::<Cow<'_, str>>::new();
    let in_for_loop = root.parent().map(|p| p.grammar_name() == "for_statement").unwrap_or(false);
    let mut has_comma = false;
    for node in root.children(&mut root.walk()) {
        if node.grammar_name() == "," {
            has_comma = true;
            break;
        }
    }
    let mut specifiers = Vec::<Cow<'_, str>>::new();
    let mut declarators = Vec::<Cow<'_, str>>::new();
    let mut current_decl = Vec::<Cow<'_, str>>::new();
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
            "compound_statement" => Some(Cow::from(handle_compound_statement(node, src))),
            "init_declarator" => Some(Cow::from(handle_init_declarator(node, src))),
            "primitive_type" => Some(Cow::from(handle_primitive_type(node, src).to_string())),
            "identifier" => Some(Cow::from(handle_identifier(node, src).to_string())),
            "function_declarator" => Some(Cow::from(handle_function_declarator(node, src))),
            "parenthesized_declarator" => Some(Cow::from(handle_parenthesized_declarator(node, src))),
            "type_qualifier" => Some(Cow::from(handle_type_qualifier(node, src).to_string())),
            "ERROR" => Some(Cow::from(handle_error(node, src))),
            "pointer_declarator" => Some(Cow::from(handle_pointer_declarator(node, src))),
            "array_declarator" => Some(Cow::from(handle_array_declarator(node, src))),
            "storage_class_specifier" => Some(Cow::from(handle_storage_class_specifier(node, src).to_string())),
            "struct_specifier" => Some(Cow::from(handle_struct_specifier(node, src))),
            "union_specifier" => Some(Cow::from(handle_union_specifier(node, src))),
            "enum_specifier" => Some(Cow::from(handle_enum_specifier(node, src))),
            "sized_type_specifier" => Some(Cow::from(handle_sized_type_specifier(node, src))),
            "macro_type_specifier" => Some(Cow::from(handle_macro_type_specifier(node, src))),
            ";" => {
                if !current_decl.is_empty() {
                    declarators.push(Cow::from(current_decl.join(" ")));
                    current_decl.clear();
                }
                if !node.is_missing() {
                    parts.push(Cow::Borrowed(";"));
                }
                None
            },
            "," => {
                if !current_decl.is_empty() {
                    declarators.push(Cow::from(current_decl.join(" ")));
                    current_decl.clear();
                }
                parts.push(Cow::Borrowed(","));
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
                line = utils::remove_pointer_spaces(&line);
            }
            lines.push(line);
        }
        return lines.join("\n");
    }

    result = parts.join(" ");
    if result.contains(",") { result = utils::remove_whitespace_before_commas(&result); }
    result = utils::remove_unnecessary_spaces(&result);
    if result.contains("*") { result = utils::remove_pointer_spaces(&result); }
    return result;
}

fn handle_function_definition(root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => {
                let compound_statement = handle_compound_statement(node, src);
                result = result.trim_end().to_string();
                result += format!("\n{}\n", compound_statement).as_str();
            },
            "primitive_type" => {
                let primitive_type = handle_primitive_type(node, src);
                result += format!("{} ", primitive_type).as_str();
            },
            "type_qualifier" => {
                let type_qualifier = handle_type_qualifier(node, src);
                result += format!("{} ", type_qualifier).as_str();
            },
            "identifier" => {
                let identifier = handle_identifier(node, src);
                result += format!("{} ", identifier).as_str();
            },
            "ERROR" => {
                // Usually an unknown specifier, like a custom alias for `static`.
                let error = handle_error(node, src);
                result += format!("{} ", error).as_str();
            },
            "function_declarator" => {
                let function_declarator = handle_function_declarator(node, src);
                result += format!("{}\n", function_declarator).as_str();
            },
            "pointer_declarator" => {
                let pointer_declarator = handle_pointer_declarator(node, src);
                result = result.trim_end().to_string();
                result += format!("{}\n", pointer_declarator).as_str();
            },
            "parenthesized_declarator" => {
                let parenthesized_declarator = handle_parenthesized_declarator(node, src);
                result += format!("{}\n", parenthesized_declarator).as_str();
            },
            "struct_specifier" => {
                let struct_specifier = handle_struct_specifier(node, src);
                result += format!("{} ", struct_specifier.trim_end()).as_str();
            },
            "union_specifier" => {
                let union_specifier = handle_union_specifier(node, src);
                result += format!("{} ", union_specifier.trim_end()).as_str();
            },
            "enum_specifier" => {
                let enum_specifier = handle_enum_specifier(node, src);
                result += format!("{} ", enum_specifier.trim_end()).as_str();
            },
            "storage_class_specifier" => {
                let storage_class_specifier = handle_storage_class_specifier(node, src);
                result += format!("{} ", storage_class_specifier).as_str();
            },
            "sized_type_specifier" => {
                let sized_type_specifier = handle_sized_type_specifier(node, src);
                result += format!("{} ", sized_type_specifier).as_str();
            },
            "comment" => {
                let comment = handle_comment(node, src);
                utils::append_comment_to_result(&mut result, &comment, node.start_byte(), src, "", "\n", true);
            },
            _ => println!("You shouldn't be here (c function_definition): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    result = utils::ensure_space_after_char(&result, '=');
    result = result.trim_end().to_string();
    return result;
}

fn handle_expression_statement(root: Node, src: &str) -> String {
    let mut result = "".to_string();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "assignment_expression" => {
                result = handle_assignment_expression(node, src);
            },
            "update_expression" => {
                result = handle_update_expression(node, src);
            },
            "call_expression" => {
                result = handle_call_expression(node, src);
            },
            "identifier" => {
                result = handle_identifier(node, src).to_string();
            },
            "binary_expression" => {
                result = handle_binary_expression(node, src);
            },
            "unary_expression" => {
                result = handle_unary_expression(node, src);
            },
            "pointer_expression" => {
                result = handle_pointer_expression(node, src);
            },
            "cast_expression" => {
                result = handle_cast_expression(node, src);
            },
            "comma_expression" => {
                result = handle_comma_expression(node, src);
            },
            "conditional_expression" => {
                result = handle_conditional_expression(node, src);
            },
            "parenthesized_expression" => {
                result = handle_parenthesized_expression(node, src);
            },
            "field_expression" => {
                result = handle_field_expression(node, src);
            },
            "subscript_expression" => {
                result = handle_subscript_expression(node, src);
            },
            "compound_literal_expression" => {
                result = handle_compound_literal_expression(node, src);
            },
            "gnu_asm_expression" => {
                result = handle_gnu_asm_expression(node, src).to_string();
            },
            "number_literal" => {
                result = handle_number_literal(node, src).to_string();
            },
            "string_literal" => {
                result = handle_string_literal(node, src);
            },
            "char_literal" => {
                result = handle_char_literal(node, src).to_string();
            },
            "true" => result += "true",
            "false" => result += "false",
            "null" => result += "NULL",
            "concatenated_string" => result += handle_concatenated_string(node, src).as_str(),
            "raw_string_literal" => result += handle_raw_string_literal(node, src),
            "comment" => {
                let comment = handle_comment(node, src);
                utils::append_expr_comment(&mut result, &comment, node.start_byte(), node.end_byte(), src);
            },
            "ERROR" => {
                println!(
                    "Warning: encountered ERROR node, likely due to preprocessor macro interrupt.\n\t--> {:?}\n",
                    node.utf8_text(src.as_bytes()).unwrap_or("[unreadable]")
                );
            },
            ";" => {
                if !node.is_missing() {
                    result = result.trim_end().to_string();
                    result += ";";
                }
            },
            _ => println!("You shouldn't be here (c expression_statement): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    if result.ends_with("\n\t") {
        result.truncate(result.len() - 2);
    }
    result = utils::remove_unnecessary_spaces(&result);
    return result;
}

fn handle_assignment_expression(root: Node, src: &str) -> String {
    let mut parts = Vec::<Cow<'_, str>>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            ";" => parts.push(Cow::Borrowed(";")),
            "=" => parts.push(Cow::Borrowed("=")),
            "+=" => parts.push(Cow::Borrowed("+=")),
            "-=" => parts.push(Cow::Borrowed("-=")),
            "*=" => parts.push(Cow::Borrowed("*=")),
            "/=" => parts.push(Cow::Borrowed("/=")),
            "%=" => parts.push(Cow::Borrowed("%=")),
            "<<=" => parts.push(Cow::Borrowed("<<=")),
            ">>=" => parts.push(Cow::Borrowed(">>=")),
            "&=" => parts.push(Cow::Borrowed("&=")),
            "|=" => parts.push(Cow::Borrowed("|=")),
            "^=" => parts.push(Cow::Borrowed("^=")),
            "null" => parts.push(Cow::Borrowed("NULL")),
            "true" => parts.push(Cow::Borrowed("true")),
            "false" => parts.push(Cow::Borrowed("false")),
            "assignment_expression" => {
                let assignment_expression = handle_inner_assignment_expression(node, src);
                parts.push(Cow::from(assignment_expression));
            },
            "call_expression" => {
                let call_expression = handle_call_expression(node, src);
                parts.push(Cow::from(call_expression));
            },
            "identifier" => {
                let identifier = handle_identifier(node, src);
                parts.push(Cow::from(identifier));
            },
            "subscript_expression" => {
                let subscript_expression = handle_subscript_expression(node, src);
                parts.push(Cow::from(subscript_expression));
            },
            "field_expression" => {
                let field_expression = handle_field_expression(node, src);
                parts.push(Cow::from(field_expression));
            },
            "number_literal" => {
                let number_literal = handle_number_literal(node, src);
                parts.push(Cow::from(number_literal));
            },
            "char_literal" => {
                let char_literal = handle_char_literal(node, src);
                parts.push(Cow::from(char_literal));
            },
            "string_literal" => {
                let string_literal = handle_string_literal(node, src);
                parts.push(Cow::from(string_literal));
            },
            "concatenated_string" => {
                let concatenated_string = handle_concatenated_string(node, src);
                parts.push(Cow::from(concatenated_string));
            },
            "cast_expression" => {
                let cast_expression = handle_cast_expression(node, src);
                parts.push(Cow::from(cast_expression));
            },
            "binary_expression" => {
                let binary_expression = handle_binary_expression(node, src);
                parts.push(Cow::from(binary_expression));
            },
            "unary_expression" => {
                let unary_expression = handle_unary_expression(node, src);
                parts.push(Cow::from(unary_expression));
            },
            "update_expression" => {
                let update_expression = handle_update_expression(node, src);
                parts.push(Cow::from(update_expression));
            },
            "pointer_expression" => {
                let pointer_expression = handle_pointer_expression(node, src);
                parts.push(Cow::from(pointer_expression));
            },
            "conditional_expression" => {
                let conditional_expression = handle_conditional_expression(node, src);
                parts.push(Cow::from(conditional_expression));
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(node, src);
                parts.push(Cow::from(parenthesized_expression));
            },
            "sizeof_expression" => {
                let sizeof_expression = handle_sizeof_expression(node, src);
                parts.push(Cow::from(sizeof_expression));
            },
            "compound_literal_expression" => {
                let compound_literal_expression = handle_compound_literal_expression(node, src);
                parts.push(Cow::from(compound_literal_expression));
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

fn handle_inner_assignment_expression(root: Node, src: &str) -> String {
    let mut parts = Vec::<Cow<'_, str>>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = handle_identifier(node, src);
                parts.push(Cow::from(identifier));
            },
            "call_expression" => {
                let call_expression = handle_call_expression(node, src);
                parts.push(Cow::from(call_expression));
            },
            "assignment_expression" => {
                let assignment_expression = handle_assignment_expression(node, src);
                parts.push(Cow::from(assignment_expression));
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(node, src);
                parts.push(Cow::from(parenthesized_expression));
            },
            "pointer_expression" => {
                let pointer_expression = handle_pointer_expression(node, src);
                parts.push(Cow::from(pointer_expression));
            },
            "conditional_expression" => {
                let conditional_expression = handle_conditional_expression(node, src);
                parts.push(Cow::from(conditional_expression));
            },
            "number_literal" => {
                let number_literal = handle_number_literal(node, src);
                parts.push(Cow::from(number_literal));
            },
            "char_literal" => {
                let char_literal = handle_char_literal(node, src);
                parts.push(Cow::from(char_literal));
            },
            "subscript_expression" => {
                let subscript_expression = handle_subscript_expression(node, src);
                parts.push(Cow::from(subscript_expression));
            },
            "field_expression" => {
                let field_expression = handle_field_expression(node, src);
                parts.push(Cow::from(field_expression));
            },
            "binary_expression" => {
                let binary_expression = handle_binary_expression(node, src);
                parts.push(Cow::from(binary_expression));
            },
            "unary_expression" => {
                let unary_expression = handle_unary_expression(node, src);
                parts.push(Cow::from(unary_expression));
            },
            "cast_expression" => {
                let cast_expression = handle_cast_expression(node, src);
                parts.push(Cow::from(cast_expression));
            },
            "sizeof_expression" => {
                let sizeof_expression = handle_sizeof_expression(node, src);
                parts.push(Cow::from(sizeof_expression));
            },
            "string_literal" => {
                let string_literal = handle_string_literal(node, src);
                parts.push(Cow::from(string_literal));
            },
            "concatenated_string" => {
                let concatenated_string = handle_concatenated_string(node, src);
                parts.push(Cow::from(concatenated_string));
            },
            "update_expression" => {
                let update_expression = handle_update_expression(node, src);
                parts.push(Cow::from(update_expression));
            },
            "compound_literal_expression" => {
                let compound_literal_expression = handle_compound_literal_expression(node, src);
                parts.push(Cow::from(compound_literal_expression));
            },
            "ERROR" => {
            },
            "true" => parts.push(Cow::Borrowed("true")),
            "false" => parts.push(Cow::Borrowed("false")),
            "null" => parts.push(Cow::Borrowed("NULL")),
            "=" => parts.push(Cow::Borrowed("=")),
            "+=" => parts.push(Cow::Borrowed("+=")),
            "-=" => parts.push(Cow::Borrowed("-=")),
            "*=" => parts.push(Cow::Borrowed("*=")),
            "/=" => parts.push(Cow::Borrowed("/=")),
            "%=" => parts.push(Cow::Borrowed("%=")),
            "|=" => parts.push(Cow::Borrowed("|=")),
            "&=" => parts.push(Cow::Borrowed("&=")),
            "^=" => parts.push(Cow::Borrowed("^=")),
            ">>=" => parts.push(Cow::Borrowed(">>=")),
            "<<=" => parts.push(Cow::Borrowed("<<=")),
            _ => println!("You shouldn't be here (c inner_assignment_expression): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let result = parts.join(" ").to_string();
    return result;
}

fn handle_inner_compound_statement(root: Node, src: &str) -> String {
    let mut parts = Vec::<Cow<'_, str>>::new();
    let children: Vec<Node> = root.children(&mut root.walk()).collect();
    let mut idx = 0usize;
    while idx < children.len() {
        let node = children[idx];
        if matches!(
            node.grammar_name(),
            "function_definition" | "declaration" | "expression_statement" | "ERROR"
        ) {
            if let Some((split_out, new_end_idx)) = try_split_leading_statement_macro_c(&children, idx, src) {
                parts.push(Cow::from(split_out.trim_end_matches('\n').to_string()));
                idx = new_end_idx + 1;
                continue;
            }
        }
        idx += 1;
        match node.grammar_name() {
            "expression_statement" => {
                let expression_statement = handle_expression_statement(node, src);
                parts.push(Cow::from(format!("\t{}", expression_statement)));
            },
            "return_statement" => {
                let return_statement = handle_return_statement(node, src);
                parts.push(Cow::from(format!("\t{}", return_statement)));
            },
            "goto_statement" => {
                let goto_statement = handle_goto_statement(node, src);
                parts.push(Cow::from(format!("\t{}", goto_statement)));
            },
            "if_statement" => {
                let mut if_statement = handle_if_statement(node, src);
                if_statement = utils::add_all_leading_tabs(&if_statement);
                parts.push(Cow::from(if_statement));
            },
            "switch_statement" => {
                let mut switch_statement = handle_switch_statement(node, src);
                switch_statement = utils::add_all_leading_tabs(&switch_statement);
                parts.push(Cow::from(switch_statement));
            },
            "break_statement" => {
                parts.push(Cow::Borrowed("\tbreak;"));
            },
            "continue_statement" => {
                parts.push(Cow::Borrowed("\tcontinue;"));
            },
            "case_statement" => {
                let mut case_statement = handle_case_statement(node, src);
                // Without the line below, "case" vertically aligns with "switch".
                // case_statement = utils::add_all_leading_tabs(&case_statement);
                case_statement = utils::remove_blank_lines(case_statement.split("\n").collect::<Vec<&str>>());
                parts.push(Cow::from(case_statement));
            },
            "declaration" => {
                let mut declaration = handle_declaration(node, src);
                declaration = utils::add_all_leading_tabs(&declaration);
                parts.push(Cow::from(declaration));
            },
            "comment" => {
                let comment = handle_comment(node, src);
                utils::push_comment_to_parts(&mut parts, &comment, node.start_byte(), src, true);
            },
            "compound_statement" => {
                let compound_statement = handle_nested_inner_compound_statement(node, src);
                parts.push(Cow::from(format!("\t{}", compound_statement)));
            },
            "while_statement" => {
                let mut while_statement = handle_while_statement(node, src);
                while_statement = utils::add_all_leading_tabs(&while_statement);
                parts.push(Cow::from(while_statement));
            },
            "preproc_call" => {
                let preproc_call = handle_preproc_call(node, src);
                parts.push(Cow::from(format!("\t{}", preproc_call)));
            },
            "preproc_ifdef" => {
                let mut preproc_ifdef = handle_preproc_ifdef(node, src);
                let has_case = node.children(&mut node.walk()).any(|c| c.grammar_name() == "case_statement");
                if !has_case {
                    preproc_ifdef = utils::add_all_leading_tabs(&preproc_ifdef);
                }
                parts.push(Cow::from(preproc_ifdef));
            },
            "preproc_if" => {
                let mut preproc_if = handle_preproc_if(node, src);
                let has_case = node.children(&mut node.walk()).any(|c| c.grammar_name() == "case_statement");
                if !has_case {
                    preproc_if = utils::add_all_leading_tabs(&preproc_if);
                }
                parts.push(Cow::from(preproc_if));
            },
            "preproc_def" => {
                let (preproc_def, end_b) = utils::consume_define_node(node, src);
                while idx < children.len() && children[idx].start_byte() < end_b {
                    idx += 1;
                }
                parts.push(Cow::from(preproc_def));
            },
            "preproc_function_def" => {
                let (preproc_function_def, end_b) = utils::consume_define_node(node, src);
                while idx < children.len() && children[idx].start_byte() < end_b {
                    idx += 1;
                }
                parts.push(Cow::from(preproc_function_def));
            },
            "preproc_include" => {
                let preproc_include = handle_preproc_include(node, src);
                parts.push(Cow::from(format!("\t{}", preproc_include)));
            },
            "type_definition" => {
                let type_definition = handle_type_definition(node, src);
                parts.push(Cow::from(format!("\t{}", type_definition)));
            },
            "struct_specifier" => {
                let struct_specifier = handle_struct_specifier(node, src);
                parts.push(Cow::from(format!("\t{}", struct_specifier)));
            },
            "union_specifier" => {
                let union_specifier = handle_union_specifier(node, src);
                parts.push(Cow::from(format!("\t{}", union_specifier)));
            },
            "enum_specifier" => {
                let enum_specifier = handle_enum_specifier(node, src);
                parts.push(Cow::from(format!("\t{}", enum_specifier)));
            },
            "do_statement" => {
                let mut do_statement = handle_do_statement(node, src);
                do_statement = utils::add_all_leading_tabs(&do_statement);
                parts.push(Cow::from(do_statement));
            },
            "for_statement" => {
                let mut for_statement = handle_for_statement(node, src);
                for_statement = utils::add_all_leading_tabs(&for_statement);
                parts.push(Cow::from(for_statement));
            },
            "labeled_statement" => {
                let mut labeled_statement = handle_labeled_statement(node, src);
                labeled_statement = utils::add_all_leading_tabs(&labeled_statement);
                parts.push(Cow::from(labeled_statement));
            },
            "attributed_statement" => {
                let mut attributed_statement = handle_attributed_statement(node, src);
                attributed_statement = utils::add_all_leading_tabs(&attributed_statement);
                parts.push(Cow::from(attributed_statement));
            },
            "identifier" => {
                let identifier = handle_identifier(node, src);
                parts.push(Cow::from(format!("\t{}", identifier)));
            },
            "ERROR" => {
                let mut error = handle_error(node, src);
                error = utils::add_all_leading_tabs(&error);
                parts.push(Cow::from(error));
            },
            ";" => parts.push(Cow::Borrowed(";")),
            "{" => parts.push(Cow::Borrowed("{")),
            "}" => parts.push(Cow::Borrowed("}")),
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

fn handle_update_expression(root: Node, src: &str) -> String {
    utils::remove_all_spaces(&root.utf8_text(src.as_bytes()).unwrap_or(""))
}

fn handle_call_expression(root: Node, src: &str) -> String {
    let mut result = "".to_string();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "call_expression" => {
                let call_expression = handle_call_expression(node, src);
                result += &call_expression;
            },
            "identifier" => {
                let identifier = handle_identifier(node, src);
                result += &identifier;
            },
            "field_expression" => {
                let field_expression = handle_field_expression(node, src);
                result += &field_expression;
            },
            "subscript_expression" => {
                let subscript_expression = handle_subscript_expression(node, src);
                result += &subscript_expression;
            },
            "pointer_expression" => {
                let pointer_expression = handle_pointer_expression(node, src);
                result += &pointer_expression;
            },
            "cast_expression" => {
                let cast_expression = handle_cast_expression(node, src);
                result += &cast_expression;
            },
            "argument_list" => {
                let argument_list = handle_argument_list(node, src);
                result += &argument_list;
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(node, src);
                result += &parenthesized_expression;
            },
            "string_literal" => result += handle_string_literal(node, src).as_str(),
            "concatenated_string" => result += handle_concatenated_string(node, src).as_str(),
            "ERROR" => result += handle_error(node, src).as_str(),
            ";" => result += ";",
            _ => println!("You shouldn't be here (c call_expression): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_if_statement(root: Node, src: &str) -> String {
    let mut parts = Vec::<Cow<'_, str>>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "if" => parts.push(Cow::Borrowed("if")),
            "compound_statement" => {
                let inner_compound_statement = handle_inner_compound_statement(node, src);
                parts.push(Cow::from(inner_compound_statement));
            },
            "return_statement" => {
                let return_statement = handle_return_statement(node, src);
                parts.push(Cow::from(return_statement));
            },
            "expression_statement" => {
                let expression_statement = handle_expression_statement(node, src);
                parts.push(Cow::from(expression_statement));
            },
            "if_statement" => {
                let if_statement = handle_if_statement(node, src);
                parts.push(Cow::from(if_statement));
            },
            "for_statement" => {
                let for_statement = handle_for_statement(node, src);
                parts.push(Cow::from(for_statement));
            },
            "while_statement" => {
                let while_statement = handle_while_statement(node, src);
                parts.push(Cow::from(while_statement));
            },
            "do_statement" => {
                let do_statement = handle_do_statement(node, src);
                parts.push(Cow::from(do_statement));
            },
            "continue_statement" => {
                parts.push(Cow::Borrowed("continue;"));
            },
            "comment" => {
                let comment = handle_comment(node, src);
                utils::push_expr_comment(&mut parts, &comment, node.start_byte(), node.end_byte(), src);
            },
            "else_clause" => {
                let else_clause = handle_else_clause(node, src);
                parts.push(Cow::from(else_clause));
            },
            "goto_statement" => {
                let goto_statement = handle_goto_statement(node, src);
                parts.push(Cow::from(goto_statement));
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(node, src);
                parts.push(Cow::from(parenthesized_expression));
            },
            "switch_statement" => {
                let switch_statement = handle_switch_statement(node, src);
                parts.push(Cow::from(switch_statement));
            },
            "break_statement" => {
                parts.push(Cow::Borrowed("break;"));
            },
            ";" => parts.push(Cow::Borrowed(";")),
            _ => println!("You shouldn't be here (c if_statement): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let result = parts.join(" ");
    return result;
}

fn handle_else_clause(root: Node, src: &str) -> String {
    let mut pieces = Vec::<Cow<'_, str>>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => {
                let inner_compound_statement = handle_inner_compound_statement(node, src);
                pieces.push(Cow::from(inner_compound_statement));
            },
            "else" => {
                pieces.push(Cow::Borrowed("else"));
            },
            "switch_statement" => {
                let switch_statement = handle_switch_statement(node, src);
                pieces.push(Cow::from(switch_statement));
            },
            "if_statement" => {
                let if_statement = handle_if_statement(node, src);
                pieces.push(Cow::from(if_statement));
            },
            "return_statement" => {
                let return_statement = handle_return_statement(node, src);
                pieces.push(Cow::from(return_statement));
            },
            "expression_statement" => {
                let expression_statement = handle_expression_statement(node, src);
                pieces.push(Cow::from(expression_statement));
            },
            "for_statement" => {
                let for_statement = handle_for_statement(node, src);
                pieces.push(Cow::from(for_statement));
            },
            "while_statement" => {
                let while_statement = handle_while_statement(node, src);
                pieces.push(Cow::from(while_statement));
            },
            "do_statement" => {
                let do_statement = handle_do_statement(node, src);
                pieces.push(Cow::from(do_statement));
            },
            "continue_statement" => {
                pieces.push(Cow::Borrowed("continue;"));
            },
            "break_statement" => {
                pieces.push(Cow::Borrowed("break;"));
            },
            "goto_statement" => {
                let goto_statement = handle_goto_statement(node, src);
                pieces.push(Cow::from(goto_statement));
            },
            "comment" => {
                let comment = handle_comment(node, src);
                utils::push_expr_comment(&mut pieces, &comment, node.start_byte(), node.end_byte(), src);
            },
            ";" => pieces.push(Cow::Borrowed(";")),
            _ => println!("You shouldn't be here (c else_clause): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let result = pieces.join(" ");
    return result;
}

fn handle_switch_statement(root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "switch" => result += "switch ",
            "compound_statement" => {
                let compound_statement = handle_inner_compound_statement(node, src);
                result += &compound_statement;
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(node, src);
                result += format!("{} ", parenthesized_expression).as_str();
            },
            _ => println!("You shouldn't be here (c switch_statement): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_case_statement(root: Node, src: &str) -> String {
    let mut result = String::new();
    let mut opened_inline_brace = false;
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "expression_statement" => {
                let expression_statement = handle_expression_statement(node, src);
                result += format!("\t{}\n", expression_statement).as_str();
            },
            "compound_statement" => {
                let compound_statement = handle_inner_compound_statement(node, src);
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
                let mut declaration = handle_declaration(node, src);
                declaration = utils::add_all_leading_tabs(&declaration);
                result += format!("{}\n", declaration).as_str();
            },
            "return_statement" => {
                let return_statement = handle_return_statement(node, src);
                result += format!("\t{}\n", return_statement).as_str();
            },
            "if_statement" => {
                let if_statement = handle_if_statement(node, src);
                result += format!("\t{}\n", if_statement).as_str();
            },
            "while_statement" => {
                let while_statement = handle_while_statement(node, src);
                result += format!("\t{}\n", while_statement).as_str();
            },
            "for_statement" => {
                let for_statement = handle_for_statement(node, src);
                result += format!("\t{}\n", for_statement).as_str();
            },
            "do_statement" => {
                let do_statement = handle_do_statement(node, src);
                result += format!("\t{}\n", do_statement).as_str();
            },
            "goto_statement" => {
                let goto_statement = handle_goto_statement(node, src);
                result += format!("\t{}\n", goto_statement).as_str();
            },
            "identifier" => {
                let identifier = handle_identifier(node, src);
                result += &identifier;
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(node, src);
                result += &parenthesized_expression;
            },
            "binary_expression" => {
                let binary_expression = handle_binary_expression(node, src);
                result += &binary_expression;
            },
            "unary_expression" => {
                let unary_expression = handle_unary_expression(node, src);
                result += &unary_expression;
            },
            "cast_expression" => {
                let cast_expression = handle_cast_expression(node, src);
                result += &cast_expression;
            },
            "switch_statement" => {
                let switch_statement = handle_switch_statement(node, src);
                result += format!("{}\n", switch_statement).as_str();
            },
            "default" => {
                result += "default"
            },
            "number_literal" => {
                let number_literal = handle_number_literal(node, src);
                result += &number_literal;
            },
            "char_literal" => {
                let char_literal = handle_char_literal(node, src);
                result += &char_literal;
            },
            "comment" => {
                let comment = handle_comment(node, src);
                utils::append_comment_to_result(&mut result, &comment, node.start_byte(), src, "\t", "\n", true);
            },
            "labeled_statement" => {
                let labeled_statment = handle_labeled_statement(node, src);
                result += format!("\t{}\n", labeled_statment).as_str();
            },
            "attributed_statement" => {
                let mut attributed_statement = handle_attributed_statement(node, src);
                attributed_statement = utils::add_all_leading_tabs(&attributed_statement);
                result += format!("{}\n", attributed_statement).as_str();
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

fn handle_goto_statement(root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "goto" => {
                result += "goto ";
            },
            "identifier" => {
                let identifier = handle_identifier(node, src);
                result += &identifier;
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

fn handle_while_statement(root: Node, src: &str) -> String {
    let mut parts = Vec::<Cow<'_, str>>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => {
                let compound_statement = handle_compound_statement(node, src);
                parts.push(Cow::from(compound_statement));
            },
            "while" => {
                parts.push(Cow::Borrowed("while"));
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(node, src);
                parts.push(Cow::from(parenthesized_expression));
            },
            "expression_statement" => {
                let expression_statement = handle_expression_statement(node, src);
                parts.push(Cow::from(expression_statement));
            },
            "switch_statement" => {
                let switch_statement = handle_switch_statement(node, src);
                parts.push(Cow::from(switch_statement));
            },
            "if_statement" => {
                let if_statement = handle_if_statement(node, src);
                parts.push(Cow::from(if_statement));
            },
            "for_statement" => {
                let for_statement = handle_for_statement(node, src);
                parts.push(Cow::from(for_statement));
            },
            "while_statement" => {
                let while_statement = handle_while_statement(node, src);
                parts.push(Cow::from(while_statement));
            },
            "do_statement" => {
                let do_statement = handle_do_statement(node, src);
                parts.push(Cow::from(do_statement));
            },
            "return_statement" => {
                let return_statement = handle_return_statement(node, src);
                parts.push(Cow::from(return_statement));
            },
            "break_statement" => parts.push(Cow::Borrowed("break;")),
            "continue_statement" => parts.push(Cow::Borrowed("continue;")),
            "goto_statement" => {
                let goto_statement = handle_goto_statement(node, src);
                parts.push(Cow::from(goto_statement));
            },
            ";" => parts.push(Cow::Borrowed(";")),
            _ => println!("You shouldn't be here (c while_statement): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let result = parts.join(" ");
    return result;
}

fn handle_argument_list(root: Node, src: &str) -> String {
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
                let identifier = handle_identifier(node, src);
                result += &identifier;
            },
            "call_expression" => {
                let call_expression = handle_call_expression(node, src);
                result += &call_expression;
            },
            "cast_expression" => {
                let cast_expression = handle_cast_expression(node, src);
                result += &cast_expression;
            },
            "field_expression" => {
                let field_expression = handle_field_expression(node, src);
                result += &field_expression;
            },
            "binary_expression" => {
                let binary_expression = handle_binary_expression(node, src);
                result += &binary_expression;
            },
            "pointer_expression" => {
                let pointer_expression = handle_pointer_expression(node, src);
                result += &pointer_expression;
            },
            "subscript_expression" => {
                let subscript_expression = handle_subscript_expression(node, src);
                result += &subscript_expression;
            },
            "sizeof_expression" => {
                let sizeof_expression = handle_sizeof_expression(node, src);
                result += &sizeof_expression;
            },
            "update_expression" => {
                let update_expression = handle_update_expression(node, src);
                result += &update_expression;
            },
            "number_literal" => {
                let number_literal = handle_number_literal(node, src);
                result += &number_literal;
            },
            "char_literal" => {
                let char_literal = handle_char_literal(node, src);
                result += &char_literal;
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(node, src);
                result += &parenthesized_expression;
            },
            "conditional_expression" => {
                let conditional_expression = handle_conditional_expression(node, src);
                result += &conditional_expression;
            },
            "unary_expression" => {
                let unary_expression = handle_unary_expression(node, src);
                result += &unary_expression;
            },
            "assignment_expression" => {
                let assignment_expression = handle_assignment_expression(node, src);
                result += &assignment_expression;
            },
            "string_literal" => {
                let string_literal = handle_string_literal(node, src);
                result += &string_literal;
            },
            "concatenated_string" => {
                let concatenated_string = handle_concatenated_string(node, src);
                result += &concatenated_string;
            },
            "compound_literal_expression" => {
                let compound_literal_expression = handle_compound_literal_expression(node, src);
                result += &compound_literal_expression;
            },
            "offsetof_expression" => {
                let offsetof_expression = handle_offsetof_expression(node, src);
                result += &offsetof_expression;
            },
            "type_descriptor" => {
                let type_descriptor = handle_type_descriptor(node, src);
                result += &type_descriptor;
            },
            "comment" => {
                let comment = handle_comment(node, src);
                utils::append_expr_comment(&mut result, &comment, node.start_byte(), node.end_byte(), src);
            },
            "ERROR" => {
                let error = handle_error(node, src);
                result += &error;
            },
            _ => println!("You shouldn't be here (c argument_list): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_return_statement(root: Node, src: &str) -> String {
    let mut result: String;
    let mut parts = Vec::<Cow<'_, str>>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => {
                let compound_statement = handle_compound_statement(node, src);
                parts.push(Cow::from(compound_statement));
            },
            "call_expression" => {
                let call_expression = handle_call_expression(node, src);
                parts.push(Cow::from(call_expression));
            },
            "binary_expression" => {
                let binary_expression = handle_binary_expression(node, src);
                parts.push(Cow::from(binary_expression));
            },
            "unary_expression" => {
                let unary_expression = handle_unary_expression(node, src);
                parts.push(Cow::from(unary_expression));
            },
            "identifier" => {
                let identifier = handle_identifier(node, src);
                parts.push(Cow::from(identifier));
            },
            "conditional_expression" => {
                let conditional_expression = handle_conditional_expression(node, src);
                parts.push(Cow::from(conditional_expression));
            },
            "field_expression" => {
                let field_expression = handle_field_expression(node, src);
                parts.push(Cow::from(field_expression));
            },
            "cast_expression" => {
                let cast_expression = handle_cast_expression(node, src);
                parts.push(Cow::from(cast_expression));
            },
            "number_literal" => {
                let number_literal = handle_number_literal(node, src);
                parts.push(Cow::from(number_literal));
            },
            "char_literal" => {
                let char_literal = handle_char_literal(node, src);
                parts.push(Cow::from(char_literal));
            },
            "string_literal" => {
                let string_literal = handle_string_literal(node, src);
                parts.push(Cow::from(string_literal));
            },
            "subscript_expression" => {
                let subscript_expression = handle_subscript_expression(node, src);
                parts.push(Cow::from(subscript_expression));
            },
            "pointer_expression" => {
                let pointer_expression = handle_pointer_expression(node, src);
                parts.push(Cow::from(pointer_expression));
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(node, src);
                parts.push(Cow::from(parenthesized_expression));
            },
            "sizeof_expression" => {
                let sizeof_expression = handle_sizeof_expression(node, src);
                parts.push(Cow::from(sizeof_expression));
            },
            "update_expression" => {
                let update_expression = handle_update_expression(node, src);
                parts.push(Cow::from(update_expression));
            },
            "assignment_expression" => {
                let assignment_expression = handle_assignment_expression(node, src);
                parts.push(Cow::from(assignment_expression));
            },
            "comma_expression" => {
                let comma_expression = handle_comma_expression(node, src);
                parts.push(Cow::from(comma_expression));
            },
            "compound_literal_expression" => {
                let compound_literal_expression = handle_compound_literal_expression(node, src);
                parts.push(Cow::from(compound_literal_expression));
            },
            "concatenated_string" => {
                let concatenated_string = handle_concatenated_string(node, src);
                parts.push(Cow::from(concatenated_string));
            },
            "return" => {
                parts.push(Cow::Borrowed("return"));
            },
            "null" => {
                parts.push(Cow::Borrowed("NULL"));
            },
            "false" => {
                parts.push(Cow::Borrowed("false"));
            },
            "true" => {
                parts.push(Cow::Borrowed("true"));
            },
            ";" => {
                parts.push(Cow::Borrowed(";"));
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

fn handle_comment(root: Node, src: &str) -> String {
    let mut parts = Vec::<Cow<'_, str>>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "(" => (),
            ")" => (),
            "," => (),
            "parameter_declaration" => (),
            "comment" => {
                let comment = extract_comment(node, src);
                parts.push(Cow::from(comment));
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
    let (ref_byte, is_top_level) = find_comment_ref_byte_c(root, src);
    return utils::format_standalone_comment_indent(&result, root.start_byte(), ref_byte, src, is_top_level);
}

fn extract_comment<'a>(root: Node, src: &'a str) -> &'a str {
    root.utf8_text(src.as_bytes()).unwrap_or("")
}

fn handle_primitive_type<'a>(root: Node, src: &'a str) -> &'a str {
    root.utf8_text(src.as_bytes()).unwrap_or("")
}

fn handle_function_declarator(root: Node, src: &str) -> String {
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
                        let pointer_declarator = handle_pointer_declarator(subnode, src);
                        temp += &pointer_declarator;
                    },
                    "parenthesized_declarator" => {
                        let parenthesized_declarator = handle_parenthesized_declarator(subnode, src);
                        temp += &parenthesized_declarator;
                    },
                    "parameter_declaration" => {
                        let mut parameter_declaration = handle_parameter_declaration(subnode, src);
                        parameter_declaration = utils::remove_pointer_spaces(&parameter_declaration);
                        temp += &parameter_declaration;
                    },
                    "identifier" => {
                        let identifier = handle_identifier(subnode, src);
                        temp += &identifier;
                    },
                    "variadic_parameter" => {
                        let variadic_parameter = handle_variadic_parameter(subnode, src);
                        temp += &variadic_parameter;
                    },
                    "comment" => {
                        let comment = handle_comment(subnode, src);
                        temp += format!("{} ", comment).as_str();
                    },
                    "ERROR" => {
                        let error = handle_error(subnode, src);
                        temp += &error;
                    },
                    _ => println!("You shouldn't be here (c function_declarator): {}: {}\n", subnode.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
                }
            }
            result += &temp;
        }
    }
    return result;
}

fn handle_pointer_declarator(root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.kind() {
            "*" => {
                result += "*";
                let child_result = handle_pointer_declarator(node.named_child(0).unwrap_or(node), src);
                result += &child_result;
            },
            "pointer_declarator" => {
                let child_result = handle_pointer_declarator(node, src);
                result += &child_result;
            },
            "identifier" => {
                let identifier = handle_identifier(node, src);
                result += format!(" {}", identifier).as_str();
            },
            "function_declarator" => {
                let function_declarator = handle_function_declarator(node, src);
                result += format!(" {}", function_declarator).as_str();
            },
            "parenthesized_declarator" => {
                let parenthesized_declarator = handle_parenthesized_declarator(node, src);
                result += format!(" {}", parenthesized_declarator).as_str();
            },
            "type_identifier" => {
                let type_identifier = handle_type_identifier(node, src);
                result += format!(" {}", type_identifier).as_str();
            },
            "type_qualifier" => {
                let type_qualifier = handle_type_qualifier(node, src);
                result += format!(" {}", type_qualifier).as_str();
            },
            "field_identifier" => {
                let field_identifier = handle_field_identifier(node, src);
                result += format!(" {}", field_identifier).as_str();
            },
            "array_declarator" => {
                let array_declarator = handle_array_declarator(node, src);
                result += format!(" {}", array_declarator).as_str();
            },
            "ms_pointer_modifier" => {
                let modifier = node.utf8_text(src.as_bytes()).unwrap_or("").trim();
                result += format!(" {}", modifier).as_str();
            },
            "ERROR" => {
                let error = handle_error(node, src);
                result += format!(" {}", error).as_str();
            },
            _ => println!("You shouldn't be here (c pointer_declarator): {}: {}\n", node.kind(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_identifier<'a>(root: Node, src: &'a str) -> &'a str {
    if root.child_count() == 0 {
        return root.utf8_text(src.as_bytes()).unwrap_or("");
    }
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => return handle_identifier(node, src),
            ";" => return ";",
            _ => println!("You shouldn't be here (c identifier): {}: {}\n", root.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    root.utf8_text(src.as_bytes()).unwrap_or("")
}

fn handle_nested_inner_compound_statement(root: Node, src: &str) -> String {
    return handle_inner_compound_statement(root, src);
}

fn handle_binary_expression(root: Node, src: &str) -> String {
    let mut parts = Vec::<Cow<'_, str>>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = handle_identifier(node, src);
                parts.push(Cow::from(identifier));
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(node, src);
                parts.push(Cow::from(parenthesized_expression));
            },
            "field_expression" => {
                let field_expression = handle_field_expression(node, src);
                parts.push(Cow::from(field_expression));
            },
            "number_literal" => {
                let number_literal = handle_number_literal(node, src);
                parts.push(Cow::from(number_literal));
            },
            "sizeof_expression" => {
                let sizeof_expression = handle_sizeof_expression(node, src);
                parts.push(Cow::from(sizeof_expression));
            },
            "subscript_expression" => {
                let subscript_expression = handle_subscript_expression(node, src);
                parts.push(Cow::from(subscript_expression));
            },
            "call_expression" => {
                let call_expression = handle_call_expression(node, src);
                parts.push(Cow::from(call_expression));
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
                // let error = handle_error(node, src);
                // parts.push(Cow::from(error));
            },
            "update_expression" => {
                let update_expression = handle_update_expression(node, src);
                parts.push(Cow::from(update_expression));
            },
            "char_literal" => {
                let char_literal = handle_char_literal(node, src);
                parts.push(Cow::from(char_literal));
            },
            "binary_expression" => {
                let binary_expression = handle_inner_binary_expression(node, src);
                parts.push(Cow::from(binary_expression));
            },
            "pointer_expression" => {
                let mut pointer_expression = handle_pointer_expression(node, src);
                pointer_expression = utils::remove_dereference_spaces(&pointer_expression);
                parts.push(Cow::from(pointer_expression));
            },
            "cast_expression" => {
                let cast_expression = handle_cast_expression(node, src);
                parts.push(Cow::from(cast_expression));
            },
            "unary_expression" => {
                let unary_expression = handle_unary_expression(node, src);
                parts.push(Cow::from(unary_expression));
            },
            "preproc_defined" => {
                let preproc_defined = handle_preproc_defined(node, src);
                parts.push(Cow::from(preproc_defined));
            },
            "string_literal" => {
                let string_literal = handle_string_literal(node, src);
                parts.push(Cow::from(string_literal));
            },
            "conditional_expression" => {
                let conditional_expression = handle_conditional_expression(node, src);
                parts.push(Cow::from(conditional_expression));
            },
            "offsetof_expression" => {
                let offsetof_expression = handle_offsetof_expression(node, src);
                parts.push(Cow::from(offsetof_expression));
            },
            "comment" => {
                let comment = handle_comment(node, src);
                utils::push_expr_comment(&mut parts, &comment, node.start_byte(), node.end_byte(), src);
            },
            "true" => parts.push(Cow::Borrowed("true")),
            "false" => parts.push(Cow::Borrowed("false")),
            "null" => parts.push(Cow::Borrowed("NULL")),
            "+" => parts.push(Cow::Borrowed("+")),
            "-" => parts.push(Cow::Borrowed("-")),
            "*" => parts.push(Cow::Borrowed("*")),
            "/" => parts.push(Cow::Borrowed("/")),
            "%" => parts.push(Cow::Borrowed("%")),
            ">" => parts.push(Cow::Borrowed(">")),
            "<" => parts.push(Cow::Borrowed("<")),
            ">=" => parts.push(Cow::Borrowed(">=")),
            "<=" => parts.push(Cow::Borrowed("<=")),
            "!=" => parts.push(Cow::Borrowed("!=")),
            "==" => parts.push(Cow::Borrowed("==")),
            ">>" => parts.push(Cow::Borrowed(">>")),
            "<<" => parts.push(Cow::Borrowed("<<")),
            "|" => parts.push(Cow::Borrowed("|")),
            "||" => parts.push(Cow::Borrowed("||")),
            "&" => parts.push(Cow::Borrowed("&")),
            "&&" => parts.push(Cow::Borrowed("&&")),
            "^" => parts.push(Cow::Borrowed("^")),
            ";" => parts.push(Cow::Borrowed(";")),
            "concatenated_string" => parts.push(Cow::from(handle_concatenated_string(node, src))),
            "raw_string_literal" => parts.push(Cow::Borrowed(handle_raw_string_literal(node, src))),
            "assignment_expression" => parts.push(Cow::from(handle_assignment_expression(node, src))),
            _ => println!("You shouldn't be here (c binary_expression): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let mut result = parts.join(" ");
    result = utils::remove_unnecessary_spaces(&result);
    return result;
}

fn handle_inner_binary_expression(root: Node, src: &str) -> String {
    let mut parts = Vec::<Cow<'_, str>>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = handle_identifier(node, src);
                parts.push(Cow::from(identifier));
            },
            "field_expression" => {
                let field_expression = handle_field_expression(node, src);
                parts.push(Cow::from(field_expression));
            },
            "call_expression" => {
                let call_expression = handle_call_expression(node, src);
                parts.push(Cow::from(call_expression));
            },
            "pointer_expression" => {
                let pointer_expression = handle_pointer_expression(node, src);
                parts.push(Cow::from(pointer_expression));
            },
            "update_expression" => {
                let update_expression = handle_update_expression(node, src);
                parts.push(Cow::from(update_expression));
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(node, src);
                parts.push(Cow::from(parenthesized_expression));
            },
            "sizeof_expression" => {
                let sizeof_expression = handle_sizeof_expression(node, src);
                parts.push(Cow::from(sizeof_expression));
            },
            "number_literal" => {
                let number_literal = handle_number_literal(node, src);
                parts.push(Cow::from(number_literal));
            },
            "char_literal" => {
                let char_literal = handle_char_literal(node, src);
                parts.push(Cow::from(char_literal));
            },
            "binary_expression" => {
                let binary_expression = handle_binary_expression(node, src);
                parts.push(Cow::from(binary_expression));
            },
            "subscript_expression" => {
                let subscript_expression = handle_subscript_expression(node, src);
                parts.push(Cow::from(subscript_expression));
            },
            "cast_expression" => {
                let cast_expression = handle_cast_expression(node, src);
                parts.push(Cow::from(cast_expression));
            },
            "unary_expression" => {
                let unary_expression = handle_unary_expression(node, src);
                parts.push(Cow::from(unary_expression));
            },
            "conditional_expression" => {
                let conditional_expression = handle_conditional_expression(node, src);
                parts.push(Cow::from(conditional_expression));
            },
            "string_literal" => {
                let string_literal = handle_string_literal(node, src);
                parts.push(Cow::from(string_literal));
            },
            "preproc_defined" => {
                let preproc_defined = handle_preproc_defined(node, src);
                parts.push(Cow::from(preproc_defined));
            },
            "offsetof_expression" => {
                let offsetof_expression = handle_offsetof_expression(node, src);
                parts.push(Cow::from(offsetof_expression));
            },
            "comment" => {
                let comment = handle_comment(node, src);
                utils::push_expr_comment(&mut parts, &comment, node.start_byte(), node.end_byte(), src);
            },
            "null" => parts.push(Cow::Borrowed("NULL")),
            "true" => parts.push(Cow::Borrowed("true")),
            "false" => parts.push(Cow::Borrowed("false")),
            "+" => parts.push(Cow::Borrowed("+")),
            "-" => parts.push(Cow::Borrowed("-")),
            "*" => parts.push(Cow::Borrowed("*")),
            "/" => parts.push(Cow::Borrowed("/")),
            "%" => parts.push(Cow::Borrowed("%")),
            ">" => parts.push(Cow::Borrowed(">")),
            "<" => parts.push(Cow::Borrowed("<")),
            ">=" => parts.push(Cow::Borrowed(">=")),
            "<=" => parts.push(Cow::Borrowed("<=")),
            "!=" => parts.push(Cow::Borrowed("!=")),
            "==" => parts.push(Cow::Borrowed("==")),
            ">>" => parts.push(Cow::Borrowed(">>")),
            "<<" => parts.push(Cow::Borrowed("<<")),
            "|" => parts.push(Cow::Borrowed("|")),
            "||" => parts.push(Cow::Borrowed("||")),
            "&" => parts.push(Cow::Borrowed("&")),
            "&&" => parts.push(Cow::Borrowed("&&")),
            "^" => parts.push(Cow::Borrowed("^")),
            ";" => parts.push(Cow::Borrowed(";")),
            "concatenated_string" => parts.push(Cow::from(handle_concatenated_string(node, src))),
            "raw_string_literal" => parts.push(Cow::Borrowed(handle_raw_string_literal(node, src))),
            "assignment_expression" => parts.push(Cow::from(handle_assignment_expression(node, src))),
            "ERROR" => parts.push(Cow::from(handle_error(node, src))),
            _ => println!("You shouldn't be here (c inner_binary_expression): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let result = parts.join(" ");
    return result;
}


fn handle_parenthesized_declarator(root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = handle_identifier(node, src);
                result += &identifier;
            },
            "pointer_declarator" => {
                let pointer_declarator = handle_pointer_declarator(node, src);
                result += &pointer_declarator;
            },
            "function_declarator" => {
                let function_declarator = handle_function_declarator(node, src);
                result += &function_declarator;
            },
            "array_declarator" => {
                let array_declarator = handle_array_declarator(node, src);
                result += &array_declarator;
            },
            "parenthesized_declarator" => {
                let parenthesized_declarator = handle_parenthesized_declarator(node, src);
                result += &parenthesized_declarator;
            },
            "(" => result += "(",
            ")" => result += ")",
            _ => println!("You shouldn't be here (c parenthesized_declarator): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_parenthesized_expression(root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "call_expression" => {
                let call_expression = handle_call_expression(node, src);
                result += &call_expression;
            },
            "identifier" => {
                let identifier = handle_identifier(node, src);
                result += &identifier;
            },
            "binary_expression" => {
                let binary_expression = handle_binary_expression(node, src);
                result += &binary_expression;
            },
            "assignment_expression" => {
                let assignment_expression = handle_assignment_expression(node, src);
                result += &assignment_expression;
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(node, src);
                result += &parenthesized_expression;
            },
            "pointer_expression" => {
                let pointer_expression = handle_pointer_expression(node, src);
                result += &pointer_expression;
            },
            "cast_expression" => {
                let cast_expression = handle_cast_expression(node, src);
                result += &cast_expression;
            },
            "field_expression" => {
                let field_expression = handle_field_expression(node, src);
                result += &field_expression;
            },
            "unary_expression" => {
                let unary_expression = handle_unary_expression(node, src);
                result += &unary_expression;
            },
            "conditional_expression" => {
                let conditional_expression = handle_conditional_expression(node, src);
                result += &conditional_expression;
            },
            "sizeof_expression" => {
                let sizeof_expression = handle_sizeof_expression(node, src);
                result += &sizeof_expression;
            },
            "comma_expression" => {
                let comma_expression = handle_comma_expression(node, src);
                result += &comma_expression;
            },
            "compound_literal_expression" => {
                let compound_literal_expression = handle_compound_literal_expression(node, src);
                result += &compound_literal_expression;
            },
            "string_literal" => {
                let string_literal = handle_string_literal(node, src);
                result += &string_literal;
            },
            "concatenated_string" => {
                let concatenated_string = handle_concatenated_string(node, src);
                result += &concatenated_string;
            },
            "char_literal" => {
                let char_literal = handle_char_literal(node, src);
                result += &char_literal;
            },
            "number_literal" => {
                let number_literal = handle_number_literal(node, src);
                result += &number_literal;
            },
            "update_expression" => {
                let update_expression = handle_update_expression(node, src);
                result += &update_expression;
            },
            "subscript_expression" => {
                let subscript_expression = handle_subscript_expression(node, src);
                result += &subscript_expression;
            },
            "true" => result += "true",
            "false" => result += "false",
            "null" => result += "NULL",
            "comment" => {
                let comment = handle_comment(node, src);
                utils::append_expr_comment(&mut result, &comment, node.start_byte(), node.end_byte(), src);
            },
            "ERROR" => {
                let error = handle_error(node, src);
                result += format!("{} ", error).as_str();
            },
            "(" => result += "(",
            ")" => result += ")",
            _ => println!("You shouldn't be here (c parenthesized_expression): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_pointer_expression(root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = handle_identifier(node, src);
                result += &identifier;
            },
            "pointer_expression" => {
                let pointer_expression = handle_inner_pointer_expression(node, src);
                result += &pointer_expression;
            },
            "subscript_expression" => {
                let subscript_expression = handle_subscript_expression(node, src);
                result += &subscript_expression;
            },
            "update_expression" => {
                let update_expression = handle_update_expression(node, src);
                result += &update_expression;
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(node, src);
                result += &parenthesized_expression;
            },
            "field_expression" => {
                let field_expression = handle_field_expression(node, src);
                result += &field_expression;
            },
            "cast_expression" => {
                let cast_expression = handle_cast_expression(node, src);
                result += &cast_expression;
            },
            "call_expression" => {
                let call_expression = handle_call_expression(node, src);
                result += &call_expression;
            },
            "binary_expression" => {
                let binary_expression = handle_binary_expression(node, src);
                result += &binary_expression;
            },
            "unary_expression" => {
                let unary_expression = handle_unary_expression(node, src);
                result += &unary_expression;
            },
            "*" => result += "*",
            "&" => result += "&",
            ";" => result += ";",
            "ERROR" => result += handle_error(node, src).as_str(),
            _ => println!("You shouldn't be here (c pointer_expression): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_inner_pointer_expression(root: Node, src: &str) -> String {
    return handle_pointer_expression(root, src);
}

fn handle_parameter_declaration(root: Node, src: &str) -> String {
    let mut parts = Vec::<Cow<'_, str>>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = handle_identifier(node, src);
                parts.push(Cow::from(identifier));
            },
            "pointer_declarator" => {
                let pointer_declarator = handle_pointer_declarator(node, src);
                parts.push(Cow::from(pointer_declarator));
            },
            "function_declarator" => {
                let function_declarator = handle_function_declarator(node, src);
                parts.push(Cow::from(function_declarator));
            },
            "parenthesized_declarator" => {
                let parenthesized_declarator = handle_parenthesized_declarator(node, src);
                parts.push(Cow::from(parenthesized_declarator));
            },
            "primitive_type" => {
                let primitive_type = handle_primitive_type(node, src);
                parts.push(Cow::from(primitive_type));
            },
            "type_qualifier" => {
                let type_qualifier = handle_type_qualifier(node, src);
                parts.push(Cow::from(type_qualifier));
            },
            "storage_class_specifier" => {
                let storage_class_specifier = handle_storage_class_specifier(node, src);
                parts.push(Cow::from(storage_class_specifier));
            },
            "struct_specifier" => {
                let struct_specifier = handle_struct_specifier(node, src);
                parts.push(Cow::from(struct_specifier));
            },
            "abstract_array_declarator" => {
                let abstract_array_declarator = handle_abstract_array_declarator(node, src);
                parts.push(Cow::from(abstract_array_declarator));
            },
            "abstract_pointer_declarator" => {
                let abstract_pointer_declarator = handle_abstract_pointer_declarator(node, src);
                parts.push(Cow::from(abstract_pointer_declarator));
            },
            "abstract_function_declarator" => {
                let abstract_function_declarator = handle_abstract_function_declarator(node, src);
                parts.push(Cow::from(abstract_function_declarator));
            },
            "sized_type_specifier" => {
                let sized_type_specifier = handle_sized_type_specifier(node, src);
                parts.push(Cow::from(sized_type_specifier));
            },
            "array_declarator" => {
                let array_declarator = handle_array_declarator(node, src);
                parts.push(Cow::from(array_declarator));
            },
            "enum_specifier" => {
                let enum_specifier = handle_enum_specifier(node, src);
                parts.push(Cow::from(enum_specifier));
            },
            "union_specifier" => {
                let union_specifier = handle_union_specifier(node, src);
                parts.push(Cow::from(union_specifier));
            },
            "ERROR" => {
                let error = handle_error(node, src);
                parts.push(Cow::from(error));
            },
            _ => println!("You shouldn't be here (c parameter_declaration): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let mut result = parts.join(" ");
    result = utils::remove_unnecessary_spaces(&result);
    return result;
}

fn handle_subscript_expression(root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "[" => result += "[",
            "]" => result += "]",
            "identifier" => {
                let identifier = handle_identifier(node, src);
                result += &identifier;
            },
            "update_expression" => {
                let update_expression = handle_update_expression(node, src);
                result += &update_expression;
            },
            "binary_expression" => {
                let binary_expression = handle_binary_expression(node, src);
                result += &binary_expression;
            },
            "number_literal" => {
                let number_literal = handle_number_literal(node, src);
                result += &number_literal;
            },
            "field_expression" => {
                let field_expression = handle_field_expression(node, src);
                result += &field_expression;
            },
            "call_expression" => {
                let call_expression = handle_call_expression(node, src);
                result += &call_expression;
            },
            "cast_expression" => {
                let cast_expression = handle_cast_expression(node, src);
                result += &cast_expression;
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(node, src);
                result += &parenthesized_expression;
            },
            "subscript_expression" => {
                let subscript_expression = handle_subscript_expression(node, src);
                result += &subscript_expression;
            },
            "pointer_expression" => {
                let pointer_expression = handle_pointer_expression(node, src);
                result += &pointer_expression;
            },
            "unary_expression" => {
                let unary_expression = handle_unary_expression(node, src);
                result += &unary_expression;
            },
            "char_literal" => {
                let char_literal = handle_char_literal(node, src);
                result += &char_literal;
            },
            "conditional_expression" => {
                let conditional_expression = handle_conditional_expression(node, src);
                result += &conditional_expression;
            },
            _ => println!("You shouldn't be here (c subscript_expression): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_field_expression(root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = handle_identifier(node, src);
                result += &identifier;
            },
            "subscript_expression" => {
                let subscript_expression = handle_subscript_expression(node, src);
                result += &subscript_expression;
            },
            "call_expression" => {
                let call_expression = handle_call_expression(node, src);
                result += &call_expression;
            },
            "field_expression" => {
                let field_expression = handle_inner_field_expression(node, src);
                result += &field_expression;
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(node, src);
                result += &parenthesized_expression;
            },
            "pointer_expression" => {
                let pointer_expression = handle_pointer_expression(node, src);
                result += &pointer_expression;
            },
            "cast_expression" => {
                let cast_expression = handle_cast_expression(node, src);
                result += &cast_expression;
            },
            "compound_literal_expression" => {
                let compound_literal_expression = handle_compound_literal_expression(node, src);
                result += &compound_literal_expression;
            },
            "->" => result += "->",
            "." => result += ".",
            _ => println!("You shouldn't be here (c field_expression): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_error(root: Node, src: &str) -> String {
    let raw = root.utf8_text(src.as_bytes()).unwrap_or("");
    if root.child_count() == 0 || (!raw.contains("#if") && !raw.contains("#endif")) {
        return raw.to_string();
    }
    let mut parts = Vec::<Cow<'_, str>>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                parts.push(Cow::from(handle_identifier(node, src).to_string()));
            },
            "binary_expression" => {
                parts.push(Cow::from(handle_binary_expression(node, src)));
            },
            "compound_statement" => {
                parts.push(Cow::from(handle_compound_statement(node, src)));
            },
            /*NOTE
                These two become errors when they are used mid-expression.
                This usually breaks the surrounding syntax, so they are simply removed. 
                *This is a destructive action and code behavior **will** change*
            */
            "#ifdef" => {},
            "#endif" => {},
            "ERROR" => {
                parts.push(Cow::from(node.utf8_text(src.as_bytes()).unwrap_or("").to_string()));
            },
            "parenthesized_expression" => {
                parts.push(Cow::from(handle_parenthesized_expression(node, src)));
            },
            "primitive_type" => {
                parts.push(Cow::from(handle_primitive_type(node, src).to_string()));
            },
            "type_qualifier" => {
                parts.push(Cow::from(handle_type_qualifier(node, src).to_string()));
            },
            "switch" => {
                parts.push(Cow::Borrowed("switch"));
            },
            "{" => parts.push(Cow::Borrowed("{")),
            "}" => parts.push(Cow::Borrowed("}")),
            ";" => parts.push(Cow::Borrowed(";")),
            "=" => parts.push(Cow::Borrowed("=")),
            "(" => parts.push(Cow::Borrowed("(")),
            ")" => parts.push(Cow::Borrowed(")")),
            "-" => parts.push(Cow::Borrowed("-")),
            ">" => parts.push(Cow::Borrowed(">")),
            "*" => parts.push(Cow::Borrowed("*")),
            "," => parts.push(Cow::Borrowed(",")),
            _ => parts.push(Cow::from(node.utf8_text(src.as_bytes()).unwrap_or("").to_string())),
        }
    }
    return parts.join(" ");
}

fn handle_cast_expression(root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "(" => result += "(",
            ")" => result += ")",
            "null" => result += "NULL",
            "true" => result += "true",
            "false" => result += "false",
            "type_descriptor" => {
                let type_descriptor = handle_type_descriptor(node, src);
                result += &type_descriptor;
            },
            "identifier" => {
                let identifier = handle_identifier(node, src);
                result += &identifier;
            },
            "call_expression" => {
                let call_expression = handle_call_expression(node, src);
                result += &call_expression;
            },
            "cast_expression" => {
                let cast_expression = handle_cast_expression(node, src);
                result += &cast_expression;
            },
            "pointer_expression" => {
                let pointer_expression = handle_pointer_expression(node, src);
                result += &pointer_expression;
            },
            "subscript_expression" => {
                let subscript_expression = handle_subscript_expression(node, src);
                result += &subscript_expression;
            },
            "field_expression" => {
                let field_expression = handle_field_expression(node, src);
                result += &field_expression;
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(node, src);
                result += &parenthesized_expression;
            },
            "number_literal" => {
                let number_literal = handle_number_literal(node, src);
                result += &number_literal;
            },
            "char_literal" => {
                let char_literal = handle_char_literal(node, src);
                result += &char_literal;
            },
            "string_literal" => {
                let string_literal = handle_string_literal(node, src);
                result += &string_literal;
            },
            "concatenated_string" => {
                let concatenated_string = handle_concatenated_string(node, src);
                result += &concatenated_string;
            },
            "unary_expression" => {
                let unary_expression = handle_unary_expression(node, src);
                result += &unary_expression;
            },
            "binary_expression" => {
                let binary_expression = handle_binary_expression(node, src);
                result += &binary_expression;
            },
            "sizeof_expression" => {
                let sizeof_expression = handle_sizeof_expression(node, src);
                result += &sizeof_expression;
            },
            "conditional_expression" => {
                let conditional_expression = handle_conditional_expression(node, src);
                result += &conditional_expression;
            },
            "update_expression" => {
                let update_expression = handle_update_expression(node, src);
                result += &update_expression;
            },
            _ => println!("You shouldn't be here (c cast_expression): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_init_declarator(root: Node, src: &str) -> String {
    let mut parts = Vec::<Cow<'_, str>>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = handle_identifier(node, src);
                parts.push(Cow::from(identifier));
            },
            "number_literal" => {
                let number_literal = handle_number_literal(node, src);
                parts.push(Cow::from(number_literal));
            },
            "char_literal" => {
                let char_literal = handle_char_literal(node, src);
                parts.push(Cow::from(char_literal));
            },
            "string_literal" => {
                let string_literal = handle_string_literal(node, src);
                parts.push(Cow::from(string_literal));
            },
            "concatenated_string" => {
                let concatenated_string = handle_concatenated_string(node, src);
                parts.push(Cow::from(concatenated_string));
            },
            "call_expression" => {
                let call_expression = handle_call_expression(node, src);
                parts.push(Cow::from(call_expression));
            },
            "field_expression" => {
                let field_expression = handle_field_expression(node, src);
                parts.push(Cow::from(field_expression));
            },
            "subscript_expression" => {
                let subscript_expression = handle_subscript_expression(node, src);
                parts.push(Cow::from(subscript_expression));
            },
            "pointer_declarator" => {
                let pointer_declarator = handle_pointer_declarator(node, src);
                parts.push(Cow::from(pointer_declarator));
            },
            "function_declarator" => {
                let function_declarator = handle_function_declarator(node, src);
                parts.push(Cow::from(function_declarator));
            },
            "parenthesized_declarator" => {
                let parenthesized_declarator = handle_parenthesized_declarator(node, src);
                parts.push(Cow::from(parenthesized_declarator));
            },
            "pointer_expression" => {
                let pointer_expression = handle_pointer_expression(node, src);
                parts.push(Cow::from(pointer_expression));
            },
            "cast_expression" => {
                let cast_expression = handle_cast_expression(node, src);
                parts.push(Cow::from(cast_expression));
            },
            "binary_expression" => {
                let binary_expression = handle_binary_expression(node, src);
                parts.push(Cow::from(binary_expression));
            },
            "initializer_list" => {
                let initializer_list = handle_initializer_list(node, src);
                parts.push(Cow::from(initializer_list));
            },
            "compound_literal_expression" => {
                let compound_literal_expression = handle_compound_literal_expression(node, src);
                parts.push(Cow::from(compound_literal_expression));
            },
            "sizeof_expression" => {
                let sizeof_expression = handle_sizeof_expression(node, src);
                parts.push(Cow::from(sizeof_expression));
            },
            "offsetof_expression" => {
                let offsetof_expression = handle_offsetof_expression(node, src);
                parts.push(Cow::from(offsetof_expression));
            },
            "unary_expression" => {
                let unary_expression = handle_unary_expression(node, src);
                parts.push(Cow::from(unary_expression));
            },
            "conditional_expression" => {
                let conditional_expression = handle_conditional_expression(node, src);
                parts.push(Cow::from(conditional_expression));
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(node, src);
                parts.push(Cow::from(parenthesized_expression));
            },
            "array_declarator" => {
                let array_declarator = handle_array_declarator(node, src);
                parts.push(Cow::from(array_declarator));
            },
            "update_expression" => {
                let update_expression = handle_update_expression(node, src);
                parts.push(Cow::from(update_expression));
            },
            "ERROR" => {
                let error = handle_error(node, src);
                parts.push(Cow::from(error));
            },
            "=" => parts.push(Cow::Borrowed("=")),
            "false" => parts.push(Cow::Borrowed("false")),
            "true" => parts.push(Cow::Borrowed("true")),
            "null" => parts.push(Cow::Borrowed("NULL")),
            _ => println!("You shouldn't be here (c init_declarator): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let result = parts.join(" ");
    return result;
}

fn handle_type_descriptor(root: Node, src: &str) -> String {
    let mut parts = Vec::<Cow<'_, str>>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = handle_identifier(node, src);
                parts.push(Cow::from(identifier));
            },
            "primitive_type" => {
                let primitive_type = handle_primitive_type(node, src);
                parts.push(Cow::from(primitive_type));
            },
            "type_qualifier" => {
                let type_qualifier = handle_type_qualifier(node, src);
                parts.push(Cow::from(type_qualifier));
            },
            "abstract_pointer_declarator" => {
                let abstract_pointer_declaration = handle_abstract_pointer_declarator(node, src);
                parts.push(Cow::from(abstract_pointer_declaration));
            },
            "abstract_array_declarator" => {
                let abstract_array_declarator = handle_abstract_array_declarator(node, src);
                parts.push(Cow::from(abstract_array_declarator));
            },
            "abstract_function_declarator" => {
                let abstract_function_declarator = handle_abstract_function_declarator(node, src);
                parts.push(Cow::from(abstract_function_declarator));
            },
            "struct_specifier" => {
                let struct_specifier = handle_struct_specifier(node, src);
                parts.push(Cow::from(struct_specifier.trim_end().to_string()));
            },
            "union_specifier" => {
                let union_specifier = handle_union_specifier(node, src);
                parts.push(Cow::from(union_specifier.trim_end().to_string()));
            },
            "enum_specifier" => {
                let enum_specifier = handle_enum_specifier(node, src);
                parts.push(Cow::from(enum_specifier.trim_end().to_string()));
            },
            "sized_type_specifier" => {
                let sized_type_specifier = handle_sized_type_specifier(node, src);
                parts.push(Cow::from(sized_type_specifier));
            },
            "ERROR" => {
                let error = handle_error(node, src);
                parts.push(Cow::from(error));
            },
            _ => println!("You shouldn't be here (c type_descriptor): {}", node.grammar_name()),
        }
    }
    let mut result = parts.join(" ");
    result = utils::remove_pointer_spaces(&result);
    return result;
}

fn handle_initializer_list(root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = handle_identifier(node, src);
                result += &identifier;
            },
            "number_literal" => {
                let number_literal = handle_number_literal(node, src);
                result += &number_literal;
            },
            "char_literal" => {
                let char_literal = handle_char_literal(node, src);
                result += &char_literal;
            },
            "comment" => {
                let comment = handle_comment(node, src);
                utils::append_expr_comment(&mut result, &comment, node.start_byte(), node.end_byte(), src);
            },
            "initializer_list" => {
                let initializer_list = handle_initializer_list(node, src);
                result += &initializer_list;
            },
            "initializer_pair" => {
                let initializer_pair = handle_initializer_pair(node, src);
                result += &initializer_pair;
            },
            "string_literal" => {
                let string_literal = handle_string_literal(node, src);
                result += &string_literal;
            },
            "concatenated_string" => {
                let concatenated_string = handle_concatenated_string(node, src);
                result += &concatenated_string;
            },
            "cast_expression" => {
                let cast_expression = handle_cast_expression(node, src);
                result += &cast_expression;
            },
            "binary_expression" => {
                let binary_expression = handle_binary_expression(node, src);
                result += &binary_expression;
            },
            "unary_expression" => {
                let unary_expression = handle_unary_expression(node, src);
                result += &unary_expression;
            },
            "call_expression" => {
                let call_expression = handle_call_expression(node, src);
                result += &call_expression;
            },
            "field_expression" => {
                let field_expression = handle_field_expression(node, src);
                result += &field_expression;
            },
            "subscript_expression" => {
                let subscript_expression = handle_subscript_expression(node, src);
                result += &subscript_expression;
            },
            "pointer_expression" => {
                let pointer_expression = handle_pointer_expression(node, src);
                result += &pointer_expression;
            },
            "conditional_expression" => {
                let conditional_expression = handle_conditional_expression(node, src);
                result += &conditional_expression;
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(node, src);
                result += &parenthesized_expression;
            },
            "sizeof_expression" => {
                let sizeof_expression = handle_sizeof_expression(node, src);
                result += &sizeof_expression;
            },
            "compound_literal_expression" => {
                let compound_literal_expression = handle_compound_literal_expression(node, src);
                result += &compound_literal_expression;
            },
            "ERROR" => {
                let error = handle_error(node, src);
                let trimmed = error.trim();
                if trimmed.starts_with('#') {
                    while result.ends_with(' ') || result.ends_with('\t') {
                        result.pop();
                    }
                    if !result.ends_with('\n') {
                        result.push('\n');
                    }
                    result += trimmed;
                    if !trimmed.ends_with('(') {
                        result.push_str("\n\t");
                    }
                } else if trimmed == ")" && result.rsplit('\n').next().unwrap_or("").trim_start().starts_with('#') {
                    result.push_str(")\n\t");
                } else {
                    result += &error;
                }
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

fn handle_sizeof_expression(root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "sizeof" => result += "sizeof",
            "(" => result += "(",
            ")" => result += ")",
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(node, src);
                result += &parenthesized_expression;
            },
            "type_descriptor" => {
                let type_descriptor = handle_type_descriptor(node, src);
                result += &type_descriptor;
            },
            "identifier" => {
                let identifier = handle_identifier(node, src);
                result += format!(" {}", identifier).as_str();
            },
            "field_expression" => {
                let field_expression = handle_field_expression(node, src);
                result += format!(" {}", field_expression).as_str();
            },
            "subscript_expression" => {
                let subscript_expression = handle_subscript_expression(node, src);
                result += format!(" {}", subscript_expression).as_str();
            },
            "pointer_expression" => {
                let pointer_expression = handle_pointer_expression(node, src);
                result += format!(" {}", pointer_expression).as_str();
            },
            "unary_expression" => {
                let unary_expression = handle_unary_expression(node, src);
                result += format!(" {}", unary_expression).as_str();
            },
            "call_expression" => {
                let call_expression = handle_call_expression(node, src);
                result += format!(" {}", call_expression).as_str();
            },
            "string_literal" => {
                let string_literal = handle_string_literal(node, src);
                result += format!(" {}", string_literal).as_str();
            },
            _ => println!("You shouldn't be here (c sizeof_expression): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_type_qualifier<'a>(root: Node, src: &'a str) -> &'a str {
    let mut result = "";
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "const" => result = "const",
            "volatile" => result = "volatile",
            "restrict" => result = "restrict",
            "__restrict" => result = "__restrict",
            "__restrict__" => result = "__restrict__",
            "_Atomic" => result = "_Atomic",
            _ => println!("You shouldn't be here (c type_qualifier): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    if result.is_empty() {
        return root.utf8_text(src.as_bytes()).unwrap_or("");
    }
    result
}

fn handle_abstract_pointer_declarator(root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "abstract_pointer_declarator" => {
                let inner = handle_abstract_pointer_declarator(node, src);
                result += &inner;
            },
            "abstract_function_declarator" => {
                let inner = handle_abstract_function_declarator(node, src);
                result += &inner;
            },
            "abstract_array_declarator" => {
                let inner = handle_abstract_array_declarator(node, src);
                result += &inner;
            },
            "abstract_parenthesized_declarator" => {
                let inner = handle_abstract_parenthesized_declarator(node, src);
                result += &inner;
            },
            "ms_pointer_modifier" => {
                let modifier = node.utf8_text(src.as_bytes()).unwrap_or("").trim();
                result += format!(" {}", modifier).as_str();
            },
            "type_qualifier" => {
                let type_qualifier = handle_type_qualifier(node, src);
                result += format!(" {}", type_qualifier).as_str();
            },
            "*" => result += "*",
            _ => println!("You shouldn't be here (c abstract_pointer_declarator): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_conditional_expression(root: Node, src: &str) -> String {
    let mut parts = Vec::<Cow<'_, str>>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = handle_identifier(node, src);
                parts.push(Cow::from(identifier));
            },
            "call_expression" => {
                let call_expression = handle_call_expression(node, src);
                parts.push(Cow::from(call_expression));
            },
            "cast_expression" => {
                let cast_expression = handle_cast_expression(node, src);
                parts.push(Cow::from(cast_expression));
            },
            "field_expression" => {
                let field_expression = handle_field_expression(node, src);
                parts.push(Cow::from(field_expression));
            },
            "binary_expression" => {
                let binary_expression = handle_binary_expression(node, src);
                parts.push(Cow::from(binary_expression));
            },
            "unary_expression" => {
                let unary_expression = handle_unary_expression(node, src);
                parts.push(Cow::from(unary_expression));
            },
            "pointer_expression" => {
                let pointer_expression = handle_pointer_expression(node, src);
                parts.push(Cow::from(pointer_expression));
            },
            "sizeof_expression" => {
                let sizeof_expression = handle_sizeof_expression(node, src);
                parts.push(Cow::from(sizeof_expression));
            },
            "conditional_expression" => {
                let conditional_expression = handle_conditional_expression(node, src);
                parts.push(Cow::from(conditional_expression));
            },
            "assignment_expression" => {
                let assignment_expression = handle_assignment_expression(node, src);
                parts.push(Cow::from(assignment_expression));
            },
            "update_expression" => {
                let update_expression = handle_update_expression(node, src);
                parts.push(Cow::from(update_expression));
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(node, src);
                parts.push(Cow::from(parenthesized_expression));
            },
            "comment" => {
                let comment = handle_comment(node, src);
                utils::push_expr_comment(&mut parts, &comment, node.start_byte(), node.end_byte(), src);
            },
            "subscript_expression" => {
                let subscript_expression = handle_subscript_expression(node, src);
                parts.push(Cow::from(subscript_expression));
            },
            "number_literal" => {
                let number_literal = handle_number_literal(node, src);
                parts.push(Cow::from(number_literal));
            },
            "char_literal" => {
                let char_literal = handle_char_literal(node, src);
                parts.push(Cow::from(char_literal));
            },
            "string_literal" => {
                let string_literal = handle_string_literal(node, src);
                parts.push(Cow::from(string_literal));
            },
            "concatenated_string" => {
                let concatenated_string = handle_concatenated_string(node, src);
                parts.push(Cow::from(concatenated_string));
            },
            "null" => parts.push(Cow::Borrowed("NULL")),
            "true" => parts.push(Cow::Borrowed("true")),
            "false" => parts.push(Cow::Borrowed("false")),
            ":" => parts.push(Cow::Borrowed(":")),
            "?" => parts.push(Cow::Borrowed("?")),
            _ => println!("You shouldn't be here (c conditional_expression): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let result = parts.join(" ");
    return result;
}

fn handle_char_literal<'a>(root: Node, src: &'a str) -> &'a str {
    root.utf8_text(src.as_bytes()).unwrap_or("")
}

fn handle_number_literal<'a>(root: Node, src: &'a str) -> &'a str {
    root.utf8_text(src.as_bytes()).unwrap_or("")
}

fn handle_unary_expression(root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = handle_identifier(node, src);
                result += &identifier;
            },
            "call_expression" => {
                let call_expression = handle_call_expression(node, src);
                result += &call_expression;
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(node, src);
                result += &parenthesized_expression;
            },
            "field_expression" => {
                let field_expression = handle_field_expression(node, src);
                result += &field_expression;
            },
            "pointer_expression" => {
                let pointer_expression = handle_pointer_expression(node, src);
                result += &pointer_expression;
            },
            "number_literal" => {
                let number_literal = handle_number_literal(node, src);
                result += &number_literal;
            },
            "assignment_expression" => {
                let assignment_expression = handle_assignment_expression(node, src);
                result += &assignment_expression;
            },
            "subscript_expression" => {
                let subscript_expression = handle_subscript_expression(node, src);
                result += &subscript_expression;
            },
            "preproc_defined" => {
                let preproc_defined = handle_preproc_defined(node, src);
                result += &preproc_defined;
            },
            "cast_expression" => {
                let cast_expression = handle_cast_expression(node, src);
                result += &cast_expression;
            },
            "unary_expression" => {
                let unary_expression = handle_unary_expression(node, src);
                result += &unary_expression;
            },
            "sizeof_expression" => {
                let sizeof_expression = handle_sizeof_expression(node, src);
                result += &sizeof_expression;
            },
            "char_literal" => {
                let char_literal = handle_char_literal(node, src);
                result += &char_literal;
            },
            "compound_literal_expression" => {
                let compound_literal_expression = handle_compound_literal_expression(node, src);
                result += &compound_literal_expression;
            },
            "true" => result += "true",
            "false" => result += "false",
            "ERROR" => {
                let error = handle_error(node, src);
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

fn handle_array_declarator(root: Node, src: &str) -> String {
    let mut parts = Vec::<Cow<'_, str>>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "number_literal" => {
                let number_literal = handle_number_literal(node, src);
                parts.push(Cow::from(number_literal));
            },
            "char_literal" => {
                let char_literal = handle_char_literal(node, src);
                parts.push(Cow::from(char_literal));
            },
            "identifier" => {
                let identifier = handle_identifier(node, src);
                parts.push(Cow::from(identifier));
            },
            "binary_expression" => {
                let binary_expression = handle_binary_expression(node, src);
                parts.push(Cow::from(binary_expression));
            },
            "cast_expression" => {
                let cast_expression = handle_cast_expression(node, src);
                parts.push(Cow::from(cast_expression));
            },
            "call_expression" => {
                let call_expression = handle_call_expression(node, src);
                parts.push(Cow::from(call_expression));
            },
            "sizeof_expression" => {
                let sizeof_expression = handle_sizeof_expression(node, src);
                parts.push(Cow::from(sizeof_expression));
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(node, src);
                parts.push(Cow::from(parenthesized_expression));
            },
            "pointer_declarator" => {
                let pointer_declarator = handle_pointer_declarator(node, src);
                parts.push(Cow::from(pointer_declarator));
            },
            "parenthesized_declarator" => {
                let parenthesized_declarator = handle_parenthesized_declarator(node, src);
                parts.push(Cow::from(parenthesized_declarator));
            },
            "array_declarator" => {
                let array_declarator = handle_array_declarator(node, src);
                parts.push(Cow::from(array_declarator));
            },
            "type_qualifier" => {
                let type_qualifier = handle_type_qualifier(node, src);
                parts.push(Cow::from(type_qualifier));
            },
            "ERROR" => {
                let error = handle_error(node, src);
                parts.push(Cow::from(error));
            },
            "static" => parts.push(Cow::Borrowed("static")),
            "*" => parts.push(Cow::Borrowed("*")),
            "[" => parts.push(Cow::Borrowed("[")),
            "]" => parts.push(Cow::Borrowed("]")),
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

fn handle_string_literal(root: Node, src: &str) -> String {
    let mut parts = Vec::<Cow<'_, str>>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "string_content" | "escape_sequence" | "\"" | "L\"" | "u\"" | "U\"" | "u8\"" => {
                parts.push(Cow::Borrowed(node.utf8_text(src.as_bytes()).unwrap_or("")));
            },
            "ERROR" => {
                parts.push(Cow::from(handle_error(node, src)));
            },
            _ => println!("You shouldn't be here (c string_literal): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let result = parts.join("");
    return result;
}

fn handle_concatenated_string(root: Node, src: &str) -> String {
    let mut parts = Vec::<Cow<'_, str>>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "string_literal" => {
                let string_literal = handle_string_literal(node, src);
                parts.push(Cow::from(string_literal));
            },
            "raw_string_literal" => {
                let raw_string_literal = handle_raw_string_literal(node, src);
                parts.push(Cow::Borrowed(raw_string_literal));
            },
            "identifier" => {
                let identifier = handle_identifier(node, src);
                parts.push(Cow::Borrowed(identifier));
            },
            "call_expression" => {
                let call_expression = handle_call_expression(node, src);
                parts.push(Cow::from(call_expression));
            },
            "comment" => {
                let comment = handle_comment(node, src);
                utils::push_expr_comment(&mut parts, &comment, node.start_byte(), node.end_byte(), src);
            },
            "ERROR" => {
                let error = handle_error(node, src);
                parts.push(Cow::from(error));
            },
            _ => println!("You shouldn't be here (c concatenated_string): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let result = parts.join(" ");
    return result;
}

fn handle_labeled_statement(root: Node, src: &str) -> String {
    let same_line = root.start_position().row == root.end_position().row;
    let sep = if same_line { " " } else { "\n\t" };
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "expression_statement" => {
                let expression_statement = handle_expression_statement(node, src);
                if same_line && expression_statement.trim() == ";" {
                    result += ";";
                } else {
                    result += format!("{}{}", sep, expression_statement).as_str();
                }
            },
            "identifier" => {
                let identifier = handle_identifier(node, src);
                result += &identifier;
            },
            "return_statement" => {
                let return_statement = handle_return_statement(node, src);
                result += format!("{}{}", sep, return_statement).as_str();
            },
            "if_statement" => {
                let if_statement = handle_if_statement(node, src);
                result += format!("{}{}", sep, if_statement).as_str();
            },
            "switch_statement" => {
                let switch_statement = handle_switch_statement(node, src);
                result += format!("{}{}", sep, switch_statement).as_str();
            },
            "while_statement" => {
                let while_statement = handle_while_statement(node, src);
                result += format!("{}{}", sep, while_statement).as_str();
            },
            "for_statement" => {
                let for_statement = handle_for_statement(node, src);
                result += format!("{}{}", sep, for_statement).as_str();
            },
            "do_statement" => {
                let do_statement = handle_do_statement(node, src);
                result += format!("{}{}", sep, do_statement).as_str();
            },
            "goto_statement" => {
                let goto_statement = handle_goto_statement(node, src);
                result += format!("{}{}", sep, goto_statement).as_str();
            },
            "declaration" => {
                let declaration = handle_declaration(node, src);
                result += format!("{}{}", sep, declaration).as_str();
            },
            "compound_statement" => {
                let compound_statement = handle_inner_compound_statement(node, src);
                result += format!("{}{}", sep, compound_statement).as_str();
            },
            "labeled_statement" => {
                let labeled_statement = handle_labeled_statement(node, src);
                result += format!("{}{}", sep, labeled_statement).as_str();
            },
            "comment" => {
                let comment = handle_comment(node, src);
                utils::append_comment_to_result(&mut result, &comment, node.start_byte(), src, "\n\t", "", false);
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

fn handle_for_statement(root: Node, src: &str) -> String {
    let mut vec = Vec::<Cow<'_, str>>::new();
    let mut temp = String::new();
    let mut reached_header_end = false;
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => {
                let compound_statement = handle_compound_statement(node, src);
                vec.push(Cow::from(compound_statement));
            },
            "if_statement" => {
                let if_statement = handle_if_statement(node, src);
                vec.push(Cow::from(if_statement));
            },
            "while_statement" => {
                let while_statement = handle_while_statement(node, src);
                vec.push(Cow::from(while_statement));
            },
            "for_statement" => {
                let for_statement = handle_for_statement(node, src);
                vec.push(Cow::from(for_statement));
            },
            "do_statement" => {
                let do_statement = handle_do_statement(node, src);
                vec.push(Cow::from(do_statement));
            },
            "switch_statement" => {
                let switch_statement = handle_switch_statement(node, src);
                vec.push(Cow::from(switch_statement));
            },
            "return_statement" => {
                let return_statement = handle_return_statement(node, src);
                vec.push(Cow::from(return_statement));
            },
            "break_statement" => vec.push(Cow::Borrowed("break;")),
            "continue_statement" => vec.push(Cow::Borrowed("continue;")),
            "goto_statement" => {
                let goto_statement = handle_goto_statement(node, src);
                vec.push(Cow::from(goto_statement));
            },
            "update_expression" => {
                let update_expression = handle_update_expression(node, src);
                if !reached_header_end {
                    temp += &update_expression;
                } else {
                    vec.push(Cow::from(update_expression));
                }
            },
            "assignment_expression" => {
                let assignment_expression = handle_assignment_expression(node, src);
                if !reached_header_end {
                    temp += &assignment_expression;
                } else {
                    vec.push(Cow::from(assignment_expression));
                }
            },
            "binary_expression" => {
                let binary_expression = handle_binary_expression(node, src);
                if !reached_header_end {
                    temp += &binary_expression;
                } else {
                    vec.push(Cow::from(binary_expression));
                }
            },
            "call_expression" => {
                let call_expression = handle_call_expression(node, src);
                if !reached_header_end {
                    temp += &call_expression;
                } else {
                    vec.push(Cow::from(call_expression));
                }
            },
            "unary_expression" => {
                let unary_expression = handle_unary_expression(node, src);
                if !reached_header_end {
                    temp += &unary_expression;
                } else {
                    vec.push(Cow::from(unary_expression));
                }
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(node, src);
                if !reached_header_end {
                    temp += &parenthesized_expression;
                } else {
                    vec.push(Cow::from(parenthesized_expression));
                }
            },
            "field_expression" => {
                let field_expression = handle_field_expression(node, src);
                if !reached_header_end {
                    temp += &field_expression;
                } else {
                    vec.push(Cow::from(field_expression));
                }
            },
            "subscript_expression" => {
                let subscript_expression = handle_subscript_expression(node, src);
                if !reached_header_end {
                    temp += &subscript_expression;
                } else {
                    vec.push(Cow::from(subscript_expression));
                }
            },
            "identifier" => {
                let identifier = handle_identifier(node, src);
                if !reached_header_end {
                    temp += &identifier;
                } else {
                    vec.push(Cow::from(identifier));
                }
            },
            "number_literal" => {
                let number_literal = handle_number_literal(node, src);
                if !reached_header_end {
                    temp += &number_literal;
                } else {
                    vec.push(Cow::from(number_literal));
                }
            },
            "declaration" => {
                let declaration = handle_declaration(node, src);
                if !reached_header_end {
                    temp += format!("{} ", declaration).as_str();
                } else {
                    vec.push(Cow::from(declaration));
                }
            },
            "expression_statement" => {
                let expression_statement = handle_expression_statement(node, src);
                vec.push(Cow::from(expression_statement));
            },
            "pointer_expression" => {
                let mut pointer_expression = handle_pointer_expression(node, src);
                pointer_expression = utils::remove_dereference_spaces(&pointer_expression);
                temp += &pointer_expression;
            },
            "comma_expression" => {
                let comma_expression = handle_comma_expression(node, src);
                temp += &comma_expression;
            },
            "true" => temp += "true",
            "false" => temp += "false",
            ";" => {
                if !reached_header_end {
                    temp += "; ";
                } else {
                    vec.push(Cow::Borrowed(";"));
                }
            },
            "(" => temp += "(",
            ")" => {
                reached_header_end = true;
                temp = temp.trim_end().to_string();
                temp += ")";
                vec.push(Cow::from(temp));
                temp = "".to_string();
            },
            "for" => temp += "for ",
            _ => println!("You shouldn't be here (c for_statement): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let result = vec.join(" ");
    return result;
}

fn handle_struct_specifier(root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = handle_identifier(node, src);
                result += format!(" {}", identifier).as_str();
            },
            "field_declaration_list" => {
                let field_declaration_list = handle_field_declaration_list(node, src);
                if result.contains("\n") {
                    result += &field_declaration_list;
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

fn handle_type_definition(root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = handle_identifier(node, src);
                result += format!(" {}", identifier).as_str();
            },
            "typedef" => {
                result += "typedef ";
            },
            "primitive_type" => {
                let primitive_type = handle_primitive_type(node, src);
                result += &primitive_type;
            },
            "type_qualifier" => {
                let type_qualifier = handle_type_qualifier(node, src);
                result += format!("{} ", type_qualifier).as_str();
            },
            "sized_type_specifier" => {
                let sized_type_specifier = handle_sized_type_specifier(node, src);
                result += &sized_type_specifier;
            },
            "function_declarator" => {
                let function_declarator = handle_function_declarator(node, src);
                result += format!(" {}", function_declarator).as_str();
            },
            "parenthesized_declarator" => {
                let parenthesized_declarator = handle_parenthesized_declarator(node, src);
                result += format!(" {}", parenthesized_declarator).as_str();
            },
            "array_declarator" => {
                let array_declarator = handle_array_declarator(node, src);
                result += format!(" {}", array_declarator).as_str();
            },
            "struct_specifier" => {
                let struct_specifier = handle_struct_specifier(node, src);
                result += &struct_specifier;
            },
            "union_specifier" => {
                let union_specifier = handle_union_specifier(node, src);
                result += &union_specifier;
            },
            "pointer_declarator" => {
                let pointer_declarator = handle_pointer_declarator(node, src);
                result = result.trim_end().to_string();
                result += &pointer_declarator;
            },
            "enum_specifier" => {
                let enum_specifier = handle_enum_specifier(node, src);
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

fn handle_field_declaration_list(root: Node, src: &str) -> String {
    let mut parts = Vec::<Cow<'_, str>>::new();
    let mut pending_incomplete_decl = false;
    let mut skip_until_byte: usize = 0;
    for node in root.children(&mut root.walk()) {
        if node.start_byte() < skip_until_byte {
            continue;
        }
        match node.grammar_name() {
            "field_declaration" => {
                let missing_semi = node_has_missing_semicolon_c(node);
                let mut field_declaration = handle_field_declaration(node, src);
                if field_declaration.contains("*") {
                    field_declaration = utils::remove_pointer_spaces(&field_declaration);
                }
                if pending_incomplete_decl {
                    utils::merge_incomplete_decl_into_parts(&mut parts, &field_declaration);
                } else {
                    parts.push(Cow::from(utils::add_all_leading_tabs(&field_declaration)));
                }
                pending_incomplete_decl = missing_semi;
            },
            "ERROR" => {
                let error = handle_error(node, src);
                if pending_incomplete_decl {
                    utils::merge_incomplete_decl_into_parts(&mut parts, &error);
                    if error.trim_end().ends_with(';') {
                        pending_incomplete_decl = false;
                    }
                } else {
                    parts.push(Cow::from(utils::add_all_leading_tabs(&error)));
                }
            },
            "comment" => {
                let comment = handle_comment(node, src);
                utils::push_comment_to_parts(&mut parts, &comment, node.start_byte(), src, true);
            },
            "preproc_ifdef" => {
                pending_incomplete_decl = false;
                let preproc_ifdef = handle_preproc_ifdef(node, src);
                parts.push(Cow::from(preproc_ifdef));
            },
            "preproc_if" => {
                pending_incomplete_decl = false;
                let preproc_if = handle_preproc_if(node, src);
                parts.push(Cow::from(preproc_if));
            },
            "preproc_def" => {
                pending_incomplete_decl = false;
                let (preproc_def, end_b) = utils::consume_define_node(node, src);
                skip_until_byte = end_b;
                parts.push(Cow::from(preproc_def));
            },
            "preproc_function_def" => {
                pending_incomplete_decl = false;
                let (preproc_function_def, end_b) = utils::consume_define_node(node, src);
                skip_until_byte = end_b;
                parts.push(Cow::from(preproc_function_def));
            },
            "preproc_call" => {
                pending_incomplete_decl = false;
                let preproc_call = handle_preproc_call(node, src);
                parts.push(Cow::from(preproc_call));
            },
            "{" => {
                pending_incomplete_decl = false;
                parts.push(Cow::Borrowed("{"));
            },
            "}" => {
                pending_incomplete_decl = false;
                parts.push(Cow::Borrowed("}"));
            },
            ";" => {
                if pending_incomplete_decl && !node.is_missing() {
                    if let Some(last) = parts.last_mut() {
                        last.to_mut().push(';');
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

fn handle_field_declaration(root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = handle_identifier(node, src);
                result += format!("{} ", identifier).as_str();
            },
            "primitive_type" => {
                let primitive_type = handle_primitive_type(node, src);
                result += format!("{} ", primitive_type).as_str();
            },
            "type_qualifier" => {
                let type_qualifier = handle_type_qualifier(node, src);
                result += format!("{} ", type_qualifier).as_str();
            },
            "sized_type_specifier" => {
                let sized_type_specifier = handle_sized_type_specifier(node, src);
                result += format!("{} ", sized_type_specifier).as_str();
            },
            "storage_class_specifier" => {
                let storage_class_specifier = handle_storage_class_specifier(node, src);
                result += format!("{} ", storage_class_specifier).as_str();
            },
            "struct_specifier" => {
                let struct_specifier = handle_struct_specifier(node, src);
                result += format!("{} ", struct_specifier).as_str();
            },
            "union_specifier" => {
                let union_specifier = handle_union_specifier(node, src);
                result += format!("{} ", union_specifier).as_str();
            },
            "enum_specifier" => {
                let enum_specifier = handle_enum_specifier(node, src);
                result += format!("{} ", enum_specifier).as_str();
            },
            "array_declarator" => {
                let array_declarator = handle_array_declarator(node, src);
                result += format!("{} ", array_declarator).as_str();
            },
            "pointer_declarator" => {
                let pointer_declarator = handle_pointer_declarator(node, src);
                result += format!("{} ", pointer_declarator).as_str();
            },
            "function_declarator" => {
                let function_declarator = handle_function_declarator(node, src);
                result += format!("{} ", function_declarator).as_str();
            },
            "parenthesized_declarator" => {
                let parenthesized_declarator = handle_parenthesized_declarator(node, src);
                result += format!("{} ", parenthesized_declarator).as_str();
            },
            "bitfield_clause" => {
                let bitfield_clause = handle_bitfield_clause(node, src);
                result += format!("{} ", bitfield_clause).as_str();
            },
            "ERROR" => {
                let error = handle_error(node, src);
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

fn handle_preproc_def(root: Node, src: &str) -> String {
    utils::consume_define_node(root, src).0
}

fn handle_abstract_array_declarator(root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "number_literal" => {
                let number_literal = handle_number_literal(node, src);
                result += &number_literal;
            },
            "identifier" => {
                let identifier = handle_identifier(node, src);
                result += &identifier;
            },
            "binary_expression" => {
                let binary_expression = handle_binary_expression(node, src);
                result += &binary_expression;
            },
            "subscript_expression" => {
                let subscript_expression = handle_subscript_expression(node, src);
                result += &subscript_expression;
            },
            "field_expression" => {
                let field_expression = handle_field_expression(node, src);
                result += &field_expression;
            },
            "call_expression" => {
                let call_expression = handle_call_expression(node, src);
                result += &call_expression;
            },
            "sizeof_expression" => {
                let sizeof_expression = handle_sizeof_expression(node, src);
                result += &sizeof_expression;
            },
            "cast_expression" => {
                let cast_expression = handle_cast_expression(node, src);
                result += &cast_expression;
            },
            "unary_expression" => {
                let unary_expression = handle_unary_expression(node, src);
                result += &unary_expression;
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(node, src);
                result += &parenthesized_expression;
            },
            "abstract_parenthesized_declarator" => {
                let abstract_parenthesized_declarator = handle_abstract_parenthesized_declarator(node, src);
                result += &abstract_parenthesized_declarator;
            },
            "abstract_array_declarator" => {
                let abstract_array_declarator = handle_abstract_array_declarator(node, src);
                result += &abstract_array_declarator;
            },
            "abstract_pointer_declarator" => {
                let abstract_pointer_declarator = handle_abstract_pointer_declarator(node, src);
                result += &abstract_pointer_declarator;
            },
            "*" => result += "*",
            "[" => result += "[",
            "]" => result += "]",
            _ => println!("You shouldn't be here (c abstract_array_declarator): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_sized_type_specifier(root: Node, src: &str) -> String {
    let mut parts = Vec::<Cow<'_, str>>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "primitive_type" => {
                let primitive_type = handle_primitive_type(node, src);
                parts.push(Cow::from(primitive_type));
            },
            "identifier" => {
                let identifier = handle_identifier(node, src);
                parts.push(Cow::from(identifier));
            },
            "long" => parts.push(Cow::Borrowed("long")),
            "short" => parts.push(Cow::Borrowed("short")),
            "unsigned" => parts.push(Cow::Borrowed("unsigned")),
            "signed" => parts.push(Cow::Borrowed("signed")),
            _ => println!("You shouldn't be here (c sized_type_specifier): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let result = parts.join(" ");
    return result;
}

fn handle_enum_specifier(root: Node, src: &str) -> String {
    let single_line = root.start_position().row == root.end_position().row;
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = handle_identifier(node, src);
                result += format!(" {}", identifier).as_str();
            },
            "primitive_type" => {
                let primitive_type = handle_primitive_type(node, src);
                result += format!(" {}", primitive_type).as_str();
            },
            "sized_type_specifier" => {
                let sized_type_specifier = handle_sized_type_specifier(node, src);
                result += format!(" {}", sized_type_specifier).as_str();
            },
            ":" => result += " :",
            "enumerator_list" => {
                let enumerator_list = handle_enumerator_list(node, src);
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
                let field_declaration_list = handle_field_declaration_list(node, src);
                if result.contains("\n") {
                    result += &field_declaration_list;
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

fn handle_union_specifier(root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = handle_identifier(node, src);
                result += format!(" {}", identifier).as_str();
            },
            "field_declaration_list" => {
                let field_declaration_list = handle_field_declaration_list(node, src);
                if result.contains("\n") {
                    result += &field_declaration_list;
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

fn handle_enumerator_list(root: Node, src: &str) -> String {
    let mut parts = Vec::<Cow<'_, str>>::new();
    let mut temp = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "enumerator" => {
                let enumerator = handle_enumerator(node, src);
                temp += format!("\t{}", enumerator).as_str();
            },
            "comment" => {
                let comment = handle_comment(node, src);
                if !temp.is_empty() {
                    if let Some(gap) = utils::trailing_comment_gap(node.start_byte(), src) {
                        let clean = comment
                            .trim_start_matches(utils::COMMENT_OFFSET_SENTINEL)
                            .trim_start_matches(utils::COMMENT_COL0_SENTINEL);
                        if clean.trim_start().starts_with("/*") && utils::has_code_after_on_line(node.end_byte(), src) {
                            temp += format!("{}{}", gap, clean).as_str();
                        } else {
                            temp += format!("{}{}", gap, clean).as_str();
                            parts.push(Cow::from(temp));
                            temp = String::new();
                        }
                    } else {
                        parts.push(Cow::from(temp));
                        temp = String::new();
                        parts.push(Cow::from(utils::add_all_leading_tabs(&comment)));
                    }
                } else {
                    utils::push_comment_to_parts(&mut parts, &comment, node.start_byte(), src, true);
                }
            },
            "preproc_if" => {
                if !temp.is_empty() {
                    parts.push(Cow::from(temp));
                    temp = String::new();
                }
                let preproc_if = handle_preproc_if(node, src);
                parts.push(Cow::from(preproc_if));
            },
            "preproc_ifdef" => {
                if !temp.is_empty() {
                    parts.push(Cow::from(temp));
                    temp = String::new();
                }
                let preproc_ifdef = handle_preproc_ifdef(node, src);
                parts.push(Cow::from(preproc_ifdef));
            },
            "preproc_def" => {
                if !temp.is_empty() {
                    parts.push(Cow::from(temp));
                    temp = String::new();
                }
                let preproc_def = handle_preproc_def(node, src);
                parts.push(Cow::from(preproc_def));
            },
            "," => {
                temp += ",";
                parts.push(Cow::from(temp));
                temp = String::new();
            },
            "{" => parts.push(Cow::Borrowed("{")),
            "}" => {
                if !temp.is_empty() {
                    parts.push(Cow::from(temp));
                    temp = String::new();
                }
                parts.push(Cow::Borrowed("}"));
            },
            _ => println!("You shouldn't be here (c enumerator_list): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let result = parts.join("\n");
    return result;
}

fn handle_enumerator(root: Node, src: &str) -> String {
    let mut parts = Vec::<Cow<'_, str>>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => parts.push(Cow::from(handle_identifier(node, src).to_string())),
            "=" => parts.push(Cow::Borrowed("=")),
            "number_literal" => parts.push(Cow::from(handle_number_literal(node, src).to_string())),
            "char_literal" => parts.push(Cow::from(handle_char_literal(node, src).to_string())),
            "binary_expression" => parts.push(Cow::from(handle_binary_expression(node, src))),
            "unary_expression" => parts.push(Cow::from(handle_unary_expression(node, src))),
            "parenthesized_expression" => parts.push(Cow::from(handle_parenthesized_expression(node, src))),
            "cast_expression" => parts.push(Cow::from(handle_cast_expression(node, src))),
            "call_expression" => parts.push(Cow::from(handle_call_expression(node, src))),
            "sizeof_expression" => parts.push(Cow::from(handle_sizeof_expression(node, src))),
            "conditional_expression" => parts.push(Cow::from(handle_conditional_expression(node, src))),
            _ => parts.push(Cow::from(node.utf8_text(src.as_bytes()).unwrap_or("").to_string())),
        }
    }
    return parts.join(" ");
}

fn handle_storage_class_specifier<'a>(root: Node, src: &'a str) -> &'a str {
    let mut result = "";
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "static" => result = "static",
            "extern" => result = "extern",
            "register" => result = "register",
            "auto" => result = "auto",
            "inline" => result = "inline",
            "__inline" => result = "__inline",
            "__inline__" => result = "__inline__",
            "_Thread_local" => result = "_Thread_local",
            "thread_local" => result = "thread_local",
            _ => println!("You shouldn't be here (c storage_class_specifier): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    if result.is_empty() {
        return root.utf8_text(src.as_bytes()).unwrap_or("");
    }
    result
}


fn handle_preproc_ifdef(root: Node, src: &str) -> String {
    let mut parts = Vec::<Cow<'_, str>>::new();
    let mut temp = String::new();
    let mut last_kind = "";
    let mut skip_until_byte: usize = 0;
    for node in root.children(&mut root.walk()) {
        if node.start_byte() < skip_until_byte {
            continue;
        }
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
                parts.push(Cow::Borrowed("#endif"));
                last_kind = "#endif";
            },
            "identifier" => {
                let identifier = handle_identifier(node, src);
                temp += &identifier;
                parts.push(Cow::from(temp));
                temp = String::new();
                last_kind = "identifier";
            },
            "preproc_if" => {
                let preproc_if = handle_preproc_if(node, src);
                parts.push(Cow::from(preproc_if));
                last_kind = "preproc_if";
            },
            "preproc_ifdef" => {
                let preproc_ifdef = handle_preproc_ifdef(node, src);
                parts.push(Cow::from(preproc_ifdef));
                last_kind = "preproc_ifdef";
            },
            "preproc_elif" => {
                let preproc_elif = handle_preproc_elif(node, src);
                parts.push(Cow::from(preproc_elif));
                last_kind = "preproc_elif";
            },
            "preproc_function_def" => {
                let (preproc_function_def, end_b) = utils::consume_define_node(node, src);
                skip_until_byte = end_b;
                parts.push(Cow::from(preproc_function_def));
                last_kind = "preproc_function_def";
            },
            "preproc_def" => {
                let (preproc_def, end_b) = utils::consume_define_node(node, src);
                skip_until_byte = end_b;
                parts.push(Cow::from(preproc_def));
                last_kind = "preproc_def";
            },
            "preproc_call" => {
                let preproc_call = handle_preproc_call(node, src);
                parts.push(Cow::from(preproc_call));
                last_kind = "preproc_call";
            },
            "preproc_include" => {
                let preproc_include = handle_preproc_include(node, src);
                parts.push(Cow::from(preproc_include));
                last_kind = "preproc_include";
            },
            "comment" => {
                let comment = handle_comment(node, src);
                if !temp.is_empty() {
                    temp += format!(" {}", comment).as_str();
                    parts.push(Cow::from(temp));
                    temp = String::new();
                } else {
                    utils::push_comment_to_parts(&mut parts, &comment, node.start_byte(), src, false);
                }
                last_kind = "comment";
            },
            "expression_statement" => {
                let expression_statement = handle_expression_statement(node, src);
                parts.push(Cow::from(expression_statement));
                last_kind = "expression_statement";
            },
            "function_definition" => {
                let function_definition = handle_function_definition(node, src);
                parts.push(Cow::from(function_definition));
                last_kind = "function_definition";
            },
            "type_definition" => {
                let type_definition = handle_type_definition(node, src);
                parts.push(Cow::from(type_definition));
                last_kind = "type_definition";
            },
            "struct_specifier" => {
                let struct_specifier = handle_struct_specifier(node, src);
                parts.push(Cow::from(format!("{};", struct_specifier.trim_end())));
                last_kind = "struct_specifier";
            },
            "union_specifier" => {
                let union_specifier = handle_union_specifier(node, src);
                parts.push(Cow::from(format!("{};", union_specifier.trim_end())));
                last_kind = "union_specifier";
            },
            "enum_specifier" => {
                let enum_specifier = handle_enum_specifier(node, src);
                parts.push(Cow::from(format!("{};", enum_specifier.trim_end())));
                last_kind = "enum_specifier";
            },
            "field_declaration" => {
                let field_declaration = handle_field_declaration(node, src);
                parts.push(Cow::from(format!("\t{}", field_declaration)));
                last_kind = "field_declaration";
            },
            "if_statement" => {
                let if_statement = handle_if_statement(node, src);
                parts.push(Cow::from(if_statement));
                last_kind = "if_statement";
            },
            "for_statement" => {
                let for_statement = handle_for_statement(node, src);
                parts.push(Cow::from(for_statement));
                last_kind = "for_statement";
            },
            "while_statement" => {
                let while_statement = handle_while_statement(node, src);
                parts.push(Cow::from(while_statement));
                last_kind = "while_statement";
            },
            "do_statement" => {
                let do_statement = handle_do_statement(node, src);
                parts.push(Cow::from(do_statement));
                last_kind = "do_statement";
            },
            "switch_statement" => {
                let switch_statement = handle_switch_statement(node, src);
                parts.push(Cow::from(switch_statement));
                last_kind = "switch_statement";
            },
            "return_statement" => {
                let return_statement = handle_return_statement(node, src);
                parts.push(Cow::from(return_statement));
                last_kind = "return_statement";
            },
            "break_statement" => {
                parts.push(Cow::Borrowed("break;"));
                last_kind = "break_statement";
            },
            "continue_statement" => {
                parts.push(Cow::Borrowed("continue;"));
                last_kind = "continue_statement";
            },
            "goto_statement" => {
                let goto_statement = handle_goto_statement(node, src);
                parts.push(Cow::from(goto_statement));
                last_kind = "goto_statement";
            },
            "labeled_statement" => {
                let labeled_statement = handle_labeled_statement(node, src);
                parts.push(Cow::from(labeled_statement));
                last_kind = "labeled_statement";
            },
            "compound_statement" => {
                let compound_statement = handle_compound_statement(node, src);
                parts.push(Cow::from(compound_statement));
                last_kind = "compound_statement";
            },
            "preproc_else" => {
                let preproc_else = handle_preproc_else(node, src);
                parts.push(Cow::from(preproc_else));
                last_kind = "preproc_else";
            },
            "declaration" => {
                let mut declaration = handle_declaration(node, src);
                if declaration.contains("*") {
                    declaration = utils::remove_pointer_spaces(&declaration);
                }
                let _ = last_kind;
                parts.push(Cow::from(declaration));
                last_kind = "declaration";
            },
            "case_statement" => {
                let mut case_statement = handle_case_statement(node, src);
                case_statement = utils::remove_blank_lines(case_statement.split("\n").collect::<Vec<&str>>());
                parts.push(Cow::from(case_statement));
                last_kind = "case_statement";
            },
            "attributed_statement" => {
                parts.push(Cow::from(handle_attributed_statement(node, src)));
                last_kind = "attributed_statement";
            },
            "ERROR" => {
                parts.push(Cow::from(handle_error(node, src)));
                last_kind = "ERROR";
            },
            "," => (),
            ";" => (),
            ":" => (),
            "\n" => (),
            "}" => parts.push(Cow::Borrowed("}")),
            _ => println!("You shouldn't be here (c preproc_ifdef): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let result = parts.join("\n");
    return result;
}

fn handle_preproc_if(root: Node, src: &str) -> String {
    let mut parts = Vec::<Cow<'_, str>>::new();
    let mut temp = String::new();
    let mut skip_until_byte: usize = 0;
    for node in root.children(&mut root.walk()) {
        if node.start_byte() < skip_until_byte {
            continue;
        }
        match node.grammar_name() {
            "#if" => {
                temp += "#if ";
            },
            "#endif" => {
                parts.push(Cow::Borrowed("#endif"));
            },
            "binary_expression" => {
                let binary_expression = handle_binary_expression(node, src);
                if !temp.is_empty() {
                    temp += &binary_expression;
                    parts.push(Cow::from(temp));
                    temp = String::new();
                } else {
                    parts.push(Cow::from(binary_expression));
                }
            },
            "unary_expression" => {
                let unary_expression = handle_unary_expression(node, src);
                if !temp.is_empty() {
                    temp += &unary_expression;
                    parts.push(Cow::from(temp));
                    temp = String::new();
                } else {
                    parts.push(Cow::from(unary_expression));
                }
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(node, src);
                if !temp.is_empty() {
                    temp += &parenthesized_expression;
                    parts.push(Cow::from(temp));
                    temp = String::new();
                } else {
                    parts.push(Cow::from(parenthesized_expression));
                }
            },
            "call_expression" => {
                let call_expression = handle_call_expression(node, src);
                if !temp.is_empty() {
                    temp += &call_expression;
                    parts.push(Cow::from(temp));
                    temp = String::new();
                } else {
                    parts.push(Cow::from(call_expression));
                }
            },
            "identifier" => {
                let identifier = handle_identifier(node, src);
                if !temp.is_empty() {
                    temp += &identifier;
                    parts.push(Cow::from(temp));
                    temp = String::new();
                } else {
                    parts.push(Cow::from(identifier));
                }
            },
            "number_literal" => {
                let number_literal = handle_number_literal(node, src);
                if !temp.is_empty() {
                    temp += &number_literal;
                    parts.push(Cow::from(temp));
                    temp = String::new();
                } else {
                    parts.push(Cow::from(number_literal));
                }
            },
            "preproc_defined" => {
                let preproc_defined = handle_preproc_defined(node, src);
                if !temp.is_empty() {
                    temp += &preproc_defined;
                    parts.push(Cow::from(temp));
                    temp = String::new();
                } else {
                    parts.push(Cow::from(preproc_defined));
                }
            },
            "preproc_include" => {
                let preproc_include = handle_preproc_include(node, src);
                parts.push(Cow::from(preproc_include));
            },
            "preproc_def" => {
                let (preproc_def, end_b) = utils::consume_define_node(node, src);
                skip_until_byte = end_b;
                parts.push(Cow::from(preproc_def));
            },
            "preproc_if" => {
                let preproc_if = handle_preproc_if(node, src);
                parts.push(Cow::from(preproc_if));
            },
            "preproc_ifdef" => {
                let preproc_ifdef = handle_preproc_ifdef(node, src);
                parts.push(Cow::from(preproc_ifdef));
            },
            "preproc_elif" => {
                let preproc_elif = handle_preproc_elif(node, src);
                parts.push(Cow::from(preproc_elif));
            },
            "preproc_else" => {
                let preproc_else = handle_preproc_else(node, src);
                parts.push(Cow::from(preproc_else));
            },
            "preproc_call" => {
                let preproc_call = handle_preproc_call(node, src);
                parts.push(Cow::from(preproc_call));
            },
            "preproc_function_def" => {
                let (preproc_function_def, end_b) = utils::consume_define_node(node, src);
                skip_until_byte = end_b;
                parts.push(Cow::from(preproc_function_def));
            },
            "declaration" => {
                let mut declaration = handle_declaration(node, src);
                if declaration.contains("*") {
                    declaration = utils::remove_pointer_spaces(&declaration);
                }
                parts.push(Cow::from(declaration));
            },
            "field_declaration" => {
                let field_declaration = handle_field_declaration(node, src);
                parts.push(Cow::from(format!("\t{}", field_declaration)));
            },
            "function_definition" => {
                let function_definition = handle_function_definition(node, src);
                parts.push(Cow::from(function_definition));
            },
            "type_definition" => {
                let type_definition = handle_type_definition(node, src);
                parts.push(Cow::from(type_definition));
            },
            "struct_specifier" => {
                let struct_specifier = handle_struct_specifier(node, src);
                parts.push(Cow::from(format!("{};", struct_specifier.trim_end())));
            },
            "union_specifier" => {
                let union_specifier = handle_union_specifier(node, src);
                parts.push(Cow::from(format!("{};", union_specifier.trim_end())));
            },
            "enum_specifier" => {
                let enum_specifier = handle_enum_specifier(node, src);
                parts.push(Cow::from(format!("{};", enum_specifier.trim_end())));
            },
            "expression_statement" => {
                let expression_statement = handle_expression_statement(node, src);
                parts.push(Cow::from(expression_statement));
            },
            "if_statement" => {
                let if_statement = handle_if_statement(node, src);
                parts.push(Cow::from(if_statement));
            },
            "for_statement" => {
                let for_statement = handle_for_statement(node, src);
                parts.push(Cow::from(for_statement));
            },
            "while_statement" => {
                let while_statement = handle_while_statement(node, src);
                parts.push(Cow::from(while_statement));
            },
            "do_statement" => {
                let do_statement = handle_do_statement(node, src);
                parts.push(Cow::from(do_statement));
            },
            "switch_statement" => {
                let switch_statement = handle_switch_statement(node, src);
                parts.push(Cow::from(switch_statement));
            },
            "return_statement" => {
                let return_statement = handle_return_statement(node, src);
                parts.push(Cow::from(return_statement));
            },
            "break_statement" => parts.push(Cow::Borrowed("break;")),
            "continue_statement" => parts.push(Cow::Borrowed("continue;")),
            "goto_statement" => {
                let goto_statement = handle_goto_statement(node, src);
                parts.push(Cow::from(goto_statement));
            },
            "labeled_statement" => {
                let labeled_statement = handle_labeled_statement(node, src);
                parts.push(Cow::from(labeled_statement));
            },
            "compound_statement" => {
                let compound_statement = handle_compound_statement(node, src);
                parts.push(Cow::from(compound_statement));
            },
            "comment" => {
                let comment = handle_comment(node, src);
                if !temp.is_empty() {
                    temp += format!(" {}", comment).as_str();
                    parts.push(Cow::from(temp));
                    temp = String::new();
                } else {
                    utils::push_comment_to_parts(&mut parts, &comment, node.start_byte(), src, false);
                }
            },
            "case_statement" => {
                let mut case_statement = handle_case_statement(node, src);
                case_statement = utils::remove_blank_lines(case_statement.split("\n").collect::<Vec<&str>>());
                parts.push(Cow::from(case_statement));
            },
            "attributed_statement" => parts.push(Cow::from(handle_attributed_statement(node, src))),
            "ERROR" => parts.push(Cow::from(handle_error(node, src))),
            "," => (),
            "\n" => (),
            ";" => (),
            ":" => (),
            "}" => parts.push(Cow::Borrowed("}")),
            _ => println!("You shouldn't be here (c preproc_if): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let result = parts.join("\n");
    return result;
}

fn handle_preproc_function_def(root: Node, src: &str) -> String {
    utils::consume_define_node(root, src).0
}

fn handle_preproc_else(root: Node, src: &str) -> String {
    let mut parts = Vec::<Cow<'_, str>>::new();
    let mut skip_until_byte: usize = 0;
    for node in root.children(&mut root.walk()) {
        if node.start_byte() < skip_until_byte {
            continue;
        }
        match node.grammar_name() {
            "#else" => {
                parts.push(Cow::Borrowed("#else"));
            },
            "return_statement" => {
                let return_statement = handle_return_statement(node, src);
                parts.push(Cow::from(return_statement));
            },
            "comment" => {
                let comment = handle_comment(node, src);
                utils::push_comment_to_parts(&mut parts, &comment, node.start_byte(), src, false);
            },
            "expression_statement" => {
                let expression_statement = handle_expression_statement(node, src);
                parts.push(Cow::from(expression_statement));
            },
            "declaration" => {
                let mut declaration = handle_declaration(node, src);
                if declaration.contains("*") {
                    declaration = utils::remove_pointer_spaces(&declaration);
                }
                parts.push(Cow::from(declaration));
            },
            "field_declaration" => {
                let field_declaration = handle_field_declaration(node, src);
                parts.push(Cow::from(format!("\t{}", field_declaration)));
            },
            "function_definition" => {
                let function_definition = handle_function_definition(node, src);
                parts.push(Cow::from(function_definition));
            },
            "type_definition" => {
                let type_definition = handle_type_definition(node, src);
                parts.push(Cow::from(type_definition));
            },
            "struct_specifier" => {
                let struct_specifier = handle_struct_specifier(node, src);
                parts.push(Cow::from(format!("{};", struct_specifier.trim_end())));
            },
            "union_specifier" => {
                let union_specifier = handle_union_specifier(node, src);
                parts.push(Cow::from(format!("{};", union_specifier.trim_end())));
            },
            "enum_specifier" => {
                let enum_specifier = handle_enum_specifier(node, src);
                parts.push(Cow::from(format!("{};", enum_specifier.trim_end())));
            },
            "if_statement" => {
                let if_statement = handle_if_statement(node, src);
                parts.push(Cow::from(if_statement));
            },
            "for_statement" => {
                let for_statement = handle_for_statement(node, src);
                parts.push(Cow::from(for_statement));
            },
            "while_statement" => {
                let while_statement = handle_while_statement(node, src);
                parts.push(Cow::from(while_statement));
            },
            "do_statement" => {
                let do_statement = handle_do_statement(node, src);
                parts.push(Cow::from(do_statement));
            },
            "switch_statement" => {
                let switch_statement = handle_switch_statement(node, src);
                parts.push(Cow::from(switch_statement));
            },
            "break_statement" => parts.push(Cow::Borrowed("break;")),
            "continue_statement" => parts.push(Cow::Borrowed("continue;")),
            "goto_statement" => {
                let goto_statement = handle_goto_statement(node, src);
                parts.push(Cow::from(goto_statement));
            },
            "compound_statement" => {
                let compound_statement = handle_compound_statement(node, src);
                parts.push(Cow::from(compound_statement));
            },
            "preproc_include" => {
                let preproc_include = handle_preproc_include(node, src);
                parts.push(Cow::from(preproc_include));
            },
            "preproc_def" => {
                let (preproc_def, end_b) = utils::consume_define_node(node, src);
                skip_until_byte = end_b;
                parts.push(Cow::from(preproc_def));
            },
            "preproc_if" => {
                let preproc_if = handle_preproc_if(node, src);
                parts.push(Cow::from(preproc_if));
            },
            "preproc_ifdef" => {
                let preproc_ifdef = handle_preproc_ifdef(node, src);
                parts.push(Cow::from(preproc_ifdef));
            },
            "preproc_elif" => {
                let preproc_elif = handle_preproc_elif(node, src);
                parts.push(Cow::from(preproc_elif));
            },
            "preproc_call" => {
                let preproc_call = handle_preproc_call(node, src);
                parts.push(Cow::from(preproc_call));
            },
            "binary_expression" => {
                let binary_expression = handle_binary_expression(node, src);
                parts.push(Cow::from(binary_expression));
            },
            "preproc_defined" => {
                let preproc_defined = handle_preproc_defined(node, src);
                parts.push(Cow::from(preproc_defined));
            },
            "preproc_function_def" => {
                let (preproc_function_def, end_b) = utils::consume_define_node(node, src);
                skip_until_byte = end_b;
                parts.push(Cow::from(preproc_function_def));
            },
            "case_statement" => {
                let mut case_statement = handle_case_statement(node, src);
                case_statement = utils::remove_blank_lines(case_statement.split("\n").collect::<Vec<&str>>());
                parts.push(Cow::from(case_statement));
            },
            "attributed_statement" => parts.push(Cow::from(handle_attributed_statement(node, src))),
            "labeled_statement" => parts.push(Cow::from(handle_labeled_statement(node, src))),
            "ERROR" => parts.push(Cow::from(handle_error(node, src))),
            "," => (),
            ";" => (),
            ":" => (),
            "\n" => (),
            "}" => parts.push(Cow::Borrowed("}")),
            _ => println!("You shouldn't be here (c preproc_else): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let result = parts.join("\n");
    return result;
}

fn handle_variadic_parameter(root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "..." => result += "...",
            _ => println!("You shouldn't be here (c variadic_parameter): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_comma_expression(root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = handle_identifier(node, src);
                result += &identifier;
            },
            "number_literal" => {
                let number_literal = handle_number_literal(node, src);
                result += &number_literal;
            },
            "assignment_expression" => {
                let assignment_expression = handle_assignment_expression(node, src);
                result += &assignment_expression;
            },
            "call_expression" => {
                let call_expression = handle_call_expression(node, src);
                result += &call_expression;
            },
            "binary_expression" => {
                let binary_expression = handle_binary_expression(node, src);
                result += &binary_expression;
            },
            "unary_expression" => {
                let unary_expression = handle_unary_expression(node, src);
                result += &unary_expression;
            },
            "pointer_expression" => {
                let pointer_expression = handle_pointer_expression(node, src);
                result += &pointer_expression;
            },
            "comma_expression" => {
                let comma_expression = handle_comma_expression(node, src);
                result += &comma_expression;
            },
            "update_expression" => {
                let update_expression = handle_update_expression(node, src);
                result += &update_expression;
            },
            "compound_literal_expression" => {
                let compound_literal_expression = handle_compound_literal_expression(node, src);
                result += &compound_literal_expression;
            },
            "," => result += ", ",
            _ => println!("You shouldn't be here (c comma_expression): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_preproc_params(root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = handle_identifier(node, src);
                result += &identifier;
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

fn handle_preproc_call(root: Node, src: &str) -> String {
    let mut parts = Vec::<Cow<'_, str>>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "preproc_directive" => {
                parts.push(Cow::from(node.utf8_text(src.as_bytes()).unwrap().trim().to_string()));
            },
            "preproc_arg" => {
                parts.push(Cow::from(node.utf8_text(src.as_bytes()).unwrap().trim().to_string()));
            },
            "comment" => {
                parts.push(Cow::from(handle_comment(node, src)));
            },
            _ => println!("You shouldn't be here (c preproc_call): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return parts.join(" ");
}

fn handle_do_statement(root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "do" => result += "do ",
            "while" => result += " while ",
            ";" => result += ";",
            "compound_statement" => {
                let compound_statement = handle_compound_statement(node, src);
                result += &compound_statement;
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(node, src);
                result += &parenthesized_expression;
            },
            "expression_statement" => {
                let expression_statement = handle_expression_statement(node, src);
                result += &expression_statement;
            },
            _ => println!("You shouldn't be here (c do_statement): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_inner_field_expression(root: Node, src: &str) -> String {
    return handle_field_expression(root, src);
}

fn handle_parameter_list(root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "(" => result += "(",
            ")" => result += ")",
            "," => result += ", ",
            "parameter_declaration" => {
                let mut parameter_declaration = handle_parameter_declaration(node, src);
                parameter_declaration = utils::remove_pointer_spaces(&parameter_declaration);
                result += &parameter_declaration;
            },
            "variadic_parameter" => {
                let variadic_parameter = handle_variadic_parameter(node, src);
                result += &variadic_parameter;
            },
            "comment" => {
                let comment = handle_comment(node, src);
                utils::append_expr_comment(&mut result, &comment, node.start_byte(), node.end_byte(), src);
            },
            _ => println!("You shouldn't be here (c parameter_list): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_initializer_pair(root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "field_designator" => {
                let field_designator = handle_field_designator(node, src);
                result += &field_designator;
            },
            "subscript_designator" => {
                let subscript_designator = handle_subscript_designator(node, src);
                result += &subscript_designator;
            },
            "identifier" => {
                let identifier = handle_identifier(node, src);
                result += &identifier;
            },
            "number_literal" => {
                let number_literal = handle_number_literal(node, src);
                result += &number_literal;
            },
            "char_literal" => {
                let char_literal = handle_char_literal(node, src);
                result += &char_literal;
            },
            "string_literal" => {
                let string_literal = handle_string_literal(node, src);
                result += &string_literal;
            },
            "concatenated_string" => {
                let concatenated_string = handle_concatenated_string(node, src);
                result += &concatenated_string;
            },
            "binary_expression" => {
                let binary_expression = handle_binary_expression(node, src);
                result += &binary_expression;
            },
            "unary_expression" => {
                let unary_expression = handle_unary_expression(node, src);
                result += &unary_expression;
            },
            "cast_expression" => {
                let cast_expression = handle_cast_expression(node, src);
                result += &cast_expression;
            },
            "call_expression" => {
                let call_expression = handle_call_expression(node, src);
                result += &call_expression;
            },
            "field_expression" => {
                let field_expression = handle_field_expression(node, src);
                result += &field_expression;
            },
            "subscript_expression" => {
                let subscript_expression = handle_subscript_expression(node, src);
                result += &subscript_expression;
            },
            "pointer_expression" => {
                let pointer_expression = handle_pointer_expression(node, src);
                result += &pointer_expression;
            },
            "conditional_expression" => {
                let conditional_expression = handle_conditional_expression(node, src);
                result += &conditional_expression;
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(node, src);
                result += &parenthesized_expression;
            },
            "sizeof_expression" => {
                let sizeof_expression = handle_sizeof_expression(node, src);
                result += &sizeof_expression;
            },
            "initializer_list" => {
                let initializer_list = handle_initializer_list(node, src);
                result += &initializer_list;
            },
            "compound_literal_expression" => {
                let compound_literal_expression = handle_compound_literal_expression(node, src);
                result += &compound_literal_expression;
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

fn handle_field_designator(root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "." => result += ".",
            "identifier" => {
                let identifier = handle_identifier(node, src);
                result += &identifier;
            },
            _ => println!("You shouldn't be here (c field_designator): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_subscript_designator(root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "[" => result += "[",
            "]" => result += "]",
            "number_literal" => result += handle_number_literal(node, src),
            "identifier" => result += handle_identifier(node, src),
            "binary_expression" => result += handle_binary_expression(node, src).as_str(),
            "char_literal" => result += handle_char_literal(node, src),
            _ => println!("You shouldn't be here (c subscript_designator): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_compound_literal_expression(root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "(" => result += "(",
            ")" => result += ")",
            "type_descriptor" => result += handle_type_descriptor(node, src).as_str(),
            "initializer_list" => result += handle_initializer_list(node, src).as_str(),
            "ERROR" => result += handle_error(node, src).as_str(),
            _ => println!("You shouldn't be here (c compound_literal_expression): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_offsetof_expression(root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "offsetof" => result += "offsetof",
            "(" => result += "(",
            ")" => result += ")",
            "," => result += ", ",
            "type_descriptor" => result += handle_type_descriptor(node, src).as_str(),
            "identifier" => result += handle_identifier(node, src),
            "field_expression" => result += handle_field_expression(node, src).as_str(),
            _ => println!("You shouldn't be here (c offsetof_expression): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_bitfield_clause(root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            ":" => result += ": ",
            "number_literal" => result += handle_number_literal(node, src),
            "identifier" => result += handle_identifier(node, src),
            "binary_expression" => result += handle_binary_expression(node, src).as_str(),
            "parenthesized_expression" => result += handle_parenthesized_expression(node, src).as_str(),
            _ => println!("You shouldn't be here (c bitfield_clause): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_abstract_function_declarator(root: Node, src: &str) -> String {
    let mut parts = Vec::<Cow<'_, str>>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "parameter_list" => parts.push(Cow::from(handle_parameter_list(node, src))),
            "abstract_parenthesized_declarator" => parts.push(Cow::from(handle_abstract_parenthesized_declarator(node, src))),
            "ERROR" => parts.push(Cow::from(handle_error(node, src))),
            _ => println!("You shouldn't be here (c abstract_function_declarator): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return parts.join("");
}

fn handle_abstract_parenthesized_declarator(root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "(" => result += "(",
            ")" => result += ")",
            "abstract_pointer_declarator" => result += handle_abstract_pointer_declarator(node, src).as_str(),
            "abstract_function_declarator" => result += handle_abstract_function_declarator(node, src).as_str(),
            "abstract_array_declarator" => result += handle_abstract_array_declarator(node, src).as_str(),
            _ => println!("You shouldn't be here (c abstract_parenthesized_declarator): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_macro_type_specifier(root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => result += handle_identifier(node, src),
            "(" => result += "(",
            ")" => result += ")",
            "type_descriptor" => result += handle_type_descriptor(node, src).as_str(),
            _ => println!("You shouldn't be here (c macro_type_specifier): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_type_identifier<'a>(root: Node, src: &'a str) -> &'a str {
    root.utf8_text(src.as_bytes()).unwrap_or("")
}

fn handle_field_identifier<'a>(root: Node, src: &'a str) -> &'a str {
    root.utf8_text(src.as_bytes()).unwrap_or("")
}

fn handle_gnu_asm_expression<'a>(root: Node, src: &'a str) -> &'a str {
    root.utf8_text(src.as_bytes()).unwrap_or("")
}

fn handle_preproc_defined(root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "defined" => result += "defined",
            "(" => result += "(",
            ")" => result += ")",
            "identifier" => {
                let identifier = handle_identifier(node, src);
                result += format!(" {}", identifier).as_str();
            },
            _ => println!("You shouldn't be here (c preproc_defined): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    result = result.replace("( ", "(");
    return result;
}

fn handle_preproc_elif(root: Node, src: &str) -> String {
    let mut parts = Vec::<Cow<'_, str>>::new();
    let mut temp = String::new();
    let mut skip_until_byte: usize = 0;
    for node in root.children(&mut root.walk()) {
        if node.start_byte() < skip_until_byte {
            continue;
        }
        match node.grammar_name() {
            "#elif" => temp += "#elif ",
            "\n" => (),
            "binary_expression" => {
                let binary_expression = handle_binary_expression(node, src);
                if !temp.is_empty() {
                    temp += &binary_expression;
                    parts.push(Cow::from(temp));
                    temp = String::new();
                } else {
                    parts.push(Cow::from(binary_expression));
                }
            },
            "unary_expression" => {
                let unary_expression = handle_unary_expression(node, src);
                if !temp.is_empty() {
                    temp += &unary_expression;
                    parts.push(Cow::from(temp));
                    temp = String::new();
                } else {
                    parts.push(Cow::from(unary_expression));
                }
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(node, src);
                if !temp.is_empty() {
                    temp += &parenthesized_expression;
                    parts.push(Cow::from(temp));
                    temp = String::new();
                } else {
                    parts.push(Cow::from(parenthesized_expression));
                }
            },
            "call_expression" => {
                let call_expression = handle_call_expression(node, src);
                if !temp.is_empty() {
                    temp += &call_expression;
                    parts.push(Cow::from(temp));
                    temp = String::new();
                } else {
                    parts.push(Cow::from(call_expression));
                }
            },
            "identifier" => {
                let identifier = handle_identifier(node, src);
                if !temp.is_empty() {
                    temp += &identifier;
                    parts.push(Cow::from(temp));
                    temp = String::new();
                } else {
                    parts.push(Cow::from(identifier));
                }
            },
            "number_literal" => {
                let number_literal = handle_number_literal(node, src);
                if !temp.is_empty() {
                    temp += &number_literal;
                    parts.push(Cow::from(temp));
                    temp = String::new();
                } else {
                    parts.push(Cow::from(number_literal));
                }
            },
            "preproc_defined" => {
                let preproc_defined = handle_preproc_defined(node, src);
                if !temp.is_empty() {
                    temp += &preproc_defined;
                    parts.push(Cow::from(temp));
                    temp = String::new();
                } else {
                    parts.push(Cow::from(preproc_defined));
                }
            },
            "preproc_def" => {
                let (preproc_def, end_b) = utils::consume_define_node(node, src);
                skip_until_byte = end_b;
                parts.push(Cow::from(preproc_def));
            },
            "preproc_include" => {
                let preproc_include = handle_preproc_include(node, src);
                parts.push(Cow::from(preproc_include));
            },
            "preproc_if" => {
                let preproc_if = handle_preproc_if(node, src);
                parts.push(Cow::from(preproc_if));
            },
            "preproc_ifdef" => {
                let preproc_ifdef = handle_preproc_ifdef(node, src);
                parts.push(Cow::from(preproc_ifdef));
            },
            "preproc_elif" => {
                let preproc_elif = handle_preproc_elif(node, src);
                parts.push(Cow::from(preproc_elif));
            },
            "preproc_else" => {
                let preproc_else = handle_preproc_else(node, src);
                parts.push(Cow::from(preproc_else));
            },
            "preproc_call" => {
                let preproc_call = handle_preproc_call(node, src);
                parts.push(Cow::from(preproc_call));
            },
            "preproc_function_def" => {
                let (preproc_function_def, end_b) = utils::consume_define_node(node, src);
                skip_until_byte = end_b;
                parts.push(Cow::from(preproc_function_def));
            },
            "declaration" => {
                let mut declaration = handle_declaration(node, src);
                if declaration.contains("*") {
                    declaration = utils::remove_pointer_spaces(&declaration);
                }
                parts.push(Cow::from(declaration));
            },
            "function_definition" => {
                let function_definition = handle_function_definition(node, src);
                parts.push(Cow::from(function_definition));
            },
            "type_definition" => {
                let type_definition = handle_type_definition(node, src);
                parts.push(Cow::from(type_definition));
            },
            "expression_statement" => {
                let expression_statement = handle_expression_statement(node, src);
                parts.push(Cow::from(expression_statement));
            },
            "return_statement" => {
                let return_statement = handle_return_statement(node, src);
                parts.push(Cow::from(return_statement));
            },
            "if_statement" => {
                let if_statement = handle_if_statement(node, src);
                parts.push(Cow::from(if_statement));
            },
            "for_statement" => parts.push(Cow::from(handle_for_statement(node, src))),
            "while_statement" => parts.push(Cow::from(handle_while_statement(node, src))),
            "do_statement" => parts.push(Cow::from(handle_do_statement(node, src))),
            "switch_statement" => parts.push(Cow::from(handle_switch_statement(node, src))),
            "case_statement" => {
                let mut case_statement = handle_case_statement(node, src);
                case_statement = utils::remove_blank_lines(case_statement.split("\n").collect::<Vec<&str>>());
                parts.push(Cow::from(case_statement));
            },
            "compound_statement" => parts.push(Cow::from(handle_compound_statement(node, src))),
            "break_statement" => parts.push(Cow::Borrowed("break;")),
            "continue_statement" => parts.push(Cow::Borrowed("continue;")),
            "goto_statement" => parts.push(Cow::from(handle_goto_statement(node, src))),
            "labeled_statement" => parts.push(Cow::from(handle_labeled_statement(node, src))),
            "attributed_statement" => parts.push(Cow::from(handle_attributed_statement(node, src))),
            "struct_specifier" => parts.push(Cow::from(format!("{};", handle_struct_specifier(node, src).trim_end()))),
            "union_specifier" => parts.push(Cow::from(format!("{};", handle_union_specifier(node, src).trim_end()))),
            "enum_specifier" => parts.push(Cow::from(format!("{};", handle_enum_specifier(node, src).trim_end()))),
            "field_declaration" => parts.push(Cow::from(format!("\t{}", handle_field_declaration(node, src)))),
            "ERROR" => parts.push(Cow::from(handle_error(node, src))),
            "comment" => {
                let comment = handle_comment(node, src);
                if !temp.is_empty() {
                    temp += format!(" {}", comment).as_str();
                    parts.push(Cow::from(temp));
                    temp = String::new();
                } else {
                    utils::push_comment_to_parts(&mut parts, &comment, node.start_byte(), src, false);
                }
            },
            "," => (),
            ";" => (),
            ":" => (),
            "}" => parts.push(Cow::Borrowed("}")),
            _ => println!("You shouldn't be here (c preproc_elif): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return parts.join("\n");
}

//NOTE Technically only C++, but some compilers support it, so just in case. 
fn handle_raw_string_literal<'a>(root: Node, src: &'a str) -> &'a str {
    root.utf8_text(src.as_bytes()).unwrap_or("")
}

fn handle_declaration_list(root: Node, src: &str) -> String {
    let mut parts = Vec::<Cow<'_, str>>::new();
    let mut skip_until_byte: usize = 0;
    for node in root.children(&mut root.walk()) {
        if node.start_byte() < skip_until_byte {
            continue;
        }
        match node.grammar_name() {
            "{" => parts.push(Cow::Borrowed("{")),
            "}" => parts.push(Cow::Borrowed("}")),
            ";" => (),
            "comment" => {
                let comment = handle_comment(node, src);
                utils::push_comment_to_parts(&mut parts, &comment, node.start_byte(), src, false);
            },
            "declaration" => {
                let mut declaration = handle_declaration(node, src);
                if declaration.contains("*") {
                    declaration = utils::remove_pointer_spaces(&declaration);
                }
                parts.push(Cow::from(declaration));
            },
            "function_definition" => {
                let function_definition = handle_function_definition(node, src);
                parts.push(Cow::from(format!("{}\n", function_definition)));
            },
            "type_definition" => {
                let td = handle_type_definition(node, src);
                if td.contains('{') {
                    parts.push(Cow::from(format!("{}\n", td)));
                } else {
                    parts.push(Cow::from(td));
                }
            },
            "struct_specifier" => parts.push(Cow::from(format!("{};\n", handle_struct_specifier(node, src).trim_end()))),
            "union_specifier" => parts.push(Cow::from(format!("{};\n", handle_union_specifier(node, src).trim_end()))),
            "enum_specifier" => parts.push(Cow::from(format!("{};\n", handle_enum_specifier(node, src).trim_end()))),
            "preproc_include" => parts.push(Cow::from(handle_preproc_include(node, src))),
            "preproc_def" => {
                let (preproc_def, end_b) = utils::consume_define_node(node, src);
                skip_until_byte = end_b;
                parts.push(Cow::from(preproc_def));
            },
            "preproc_function_def" => {
                let (preproc_function_def, end_b) = utils::consume_define_node(node, src);
                skip_until_byte = end_b;
                parts.push(Cow::from(preproc_function_def));
            },
            "preproc_call" => parts.push(Cow::from(handle_preproc_call(node, src))),
            "preproc_ifdef" => parts.push(Cow::from(handle_preproc_ifdef(node, src))),
            "preproc_if" => parts.push(Cow::from(handle_preproc_if(node, src))),
            "preproc_elif" => parts.push(Cow::from(handle_preproc_elif(node, src))),
            "preproc_else" => parts.push(Cow::from(handle_preproc_else(node, src))),
            "ERROR" => parts.push(Cow::from(handle_error(node, src))),
            _ => println!("You shouldn't be here (c declaration_list): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let result = parts.join("\n");
    return result;
}

fn handle_attribute<'a>(root: Node, src: &'a str) -> &'a str {
    root.utf8_text(src.as_bytes()).unwrap_or("")
}

fn handle_attributed_statement(root: Node, src: &str) -> String {
    let mut attrs = Vec::<Cow<'_, str>>::new();
    let mut stmt = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "attribute_declaration" | "attribute_specifier" => {
                attrs.push(Cow::Borrowed(handle_attribute(node, src)));
            },
            "expression_statement" => stmt = handle_expression_statement(node, src),
            "return_statement" => stmt = handle_return_statement(node, src),
            "if_statement" => stmt = handle_if_statement(node, src),
            "switch_statement" => stmt = handle_switch_statement(node, src),
            "for_statement" => stmt = handle_for_statement(node, src),
            "while_statement" => stmt = handle_while_statement(node, src),
            "do_statement" => stmt = handle_do_statement(node, src),
            "compound_statement" => stmt = handle_inner_compound_statement(node, src),
            "break_statement" => stmt = "break;".to_string(),
            "continue_statement" => stmt = "continue;".to_string(),
            "goto_statement" => stmt = handle_goto_statement(node, src),
            "case_statement" => stmt = handle_case_statement(node, src),
            "labeled_statement" => stmt = handle_labeled_statement(node, src),
            "attributed_statement" => stmt = handle_attributed_statement(node, src),
            ";" => {
                if !stmt.ends_with(';') {
                    stmt += ";";
                }
            },
            "comment" => {
                let comment = handle_comment(node, src);
                utils::append_expr_comment(&mut stmt, &comment, node.start_byte(), node.end_byte(), src);
            },
            _ => println!("You shouldn't be here (c attributed_statement): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let attr_str = attrs.join(" ");
    let trimmed = stmt.trim();
    if trimmed.is_empty() || trimmed == ";" {
        format!("{};", attr_str)
    } else {
        format!("{} {}", attr_str, stmt)
    }
}


