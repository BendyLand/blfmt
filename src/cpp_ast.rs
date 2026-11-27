use tree_sitter::{Tree, Node};
use crate::{cpp_format, utils};

//FIX: 
/*
This case:
```cpp
CPPSPMD_DECL(const uint32_t, g_lane_masks_128[4][4]) = 
{ 
	{ UINT32_MAX, 0, 0, 0 },
	{ 0, UINT32_MAX, 0, 0 },
	{ 0, 0, UINT32_MAX, 0 },
	{ 0, 0, 0, UINT32_MAX },
};
```

Turns into:
```cpp
CPPSPMD_DECL(const uint32_t, g_lane_masks_128[4][4])

=

{
};
```
*/
fn node_has_missing_semicolon_cpp(root: Node) -> bool {
    for child in root.children(&mut root.walk()) {
        if child.grammar_name() == ";" && child.is_missing() {
            return true;
        }
    }
    false
}

pub fn traverse_cpp_ast(ast: Tree, src: String, style: utils::Style) -> String {
    let root = ast.root_node();
    let mut result = String::new();
    let mut last_group_kind = String::new();
    let mut pending_incomplete_decl = false;
    let lines_before_blank_lines = utils::scan_for_lines_before_blank_lines(src.clone());
    for child in root.children(&mut root.walk()) {
        match child.grammar_name() {
            "preproc_include" => {
                pending_incomplete_decl = false;
                if !last_group_kind.contains("preproc") { result += "\n"; }
                let preproc_include = handle_preproc_include(child, src.clone());
                result += format!("{}\n", preproc_include).as_str();
                last_group_kind = "preproc_include".to_string();
            },
            "declaration" => {
                let missing_semi = node_has_missing_semicolon_cpp(child);
                let declaration = handle_declaration(child, src.clone());
                if pending_incomplete_decl {
                    utils::merge_incomplete_decl_into_result(&mut result, &declaration);
                } else {
                    if last_group_kind.contains("preproc") { result += "\n"; }
                    result += format!("{}\n", declaration).as_str();
                }
                pending_incomplete_decl = missing_semi;
                last_group_kind = "declaration".to_string();
            },
            "function_definition" => {
                pending_incomplete_decl = false;
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
                let missing_semi = node_has_missing_semicolon_cpp(child);
                let expression_statement = handle_expression_statement(child, src.clone());
                if pending_incomplete_decl {
                    utils::merge_incomplete_decl_into_result(&mut result, &expression_statement);
                    pending_incomplete_decl = missing_semi;
                } else {
                    result += format!("{}\n\n", expression_statement).as_str();
                    pending_incomplete_decl = missing_semi;
                    last_group_kind = "expression_statement".to_string();
                }
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
                if !result.trim().is_empty() && utils::trailing_comment_gap(child.start_byte(), &src).is_some() {
                    utils::append_comment_to_result(&mut result, &comment, child.start_byte(), &src, "", "\n", true);
                } else {
                    if last_group_kind.contains("preproc") { result += "\n"; }
                    result += format!("{}\n", comment).as_str();
                    last_group_kind = "comment".to_string();
                }
            },
            "ERROR" => {
                let error = handle_error(child, src.clone());
                result += format!("{}\n\n", error).as_str();
                last_group_kind = "ERROR".to_string();
            },
            "type_definition" => {
                if last_group_kind.contains("preproc") { result += "\n"; }
                let type_definition = handle_type_definition(child, src.clone());
                result += format!("{}\n\n", type_definition).as_str();
                last_group_kind = "type_definition".to_string();
            },
            "struct_specifier" => {
                if last_group_kind.contains("preproc") { result += "\n"; }
                let struct_specifier = handle_struct_specifier(child, src.clone());
                result += format!("{};\n\n", struct_specifier.trim_end()).as_str();
                last_group_kind = "struct_specifier".to_string();
            },
            "union_specifier" => {
                if last_group_kind.contains("preproc") { result += "\n"; }
                let union_specifier = handle_union_specifier(child, src.clone());
                result += format!("{};\n\n", union_specifier.trim_end()).as_str();
                last_group_kind = "union_specifier".to_string();
            },
            "preproc_def" => {
                if !last_group_kind.contains("preproc") || last_group_kind == "preproc_include" || last_group_kind == "preproc_call" {
                    result += "\n";
                }
                let preproc_def = handle_preproc_def(child, src.clone());
                result += format!("{}\n", preproc_def).as_str();
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
            "preproc_elif" => {
                if !last_group_kind.contains("preproc") { result += "\n"; }
                let preproc_elif = handle_preproc_elif(child, src.clone());
                result += format!("{}\n\n", preproc_elif).as_str();
                last_group_kind = "preproc_elif".to_string();
            },
            "preproc_function_def" => {
                if !last_group_kind.contains("preproc") || last_group_kind == "preproc_include" || last_group_kind == "preproc_call" {
                    result += "\n";
                }
                let preproc_function_def = handle_preproc_function_def(child, src.clone());
                result += format!("{}\n", preproc_function_def).as_str();
                last_group_kind = "preproc_function_def".to_string();
            },
            "preproc_call" => {
                if !last_group_kind.contains("preproc") || last_group_kind == "preproc_include" {
                    result += "\n";
                }
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
                if last_group_kind.contains("preproc") { result += "\n"; }
                let template_declaration = handle_template_declaration(child, src.clone());
                result += format!("{}\n\n", template_declaration).as_str();
                last_group_kind = "template_declaration".to_string();
            },
            "template_instantiation" => {
                let template_instantiation = handle_template_instantiation(child, src.clone());
                result += format!("{}\n", template_instantiation).as_str();
                last_group_kind = "template_instantiation".to_string();
            },
            "linkage_specification" => {
                let linkage_specification = handle_linkage_specification(child, src.clone());
                result += format!("{}\n\n", linkage_specification).as_str();
                last_group_kind = "linkage_specification".to_string();
            },
            "friend_declaration" => {
                let friend_declaration = handle_friend_declaration(child, src.clone());
                result += format!("{}\n", friend_declaration).as_str();
                last_group_kind = "friend_declaration".to_string();
            },
            "namespace_alias_definition" => {
                if last_group_kind != "namespace_alias_definition" { result += "\n"; }
                let namespace_alias_definition = handle_namespace_alias_definition(child, src.clone());
                result += format!("{}\n\n", namespace_alias_definition).as_str();
                last_group_kind = "namespace_alias_definition".to_string();
            },
            "namespace_definition" => {
                if last_group_kind != "namespace_definition" && last_group_kind != "comment" { result += "\n"; }
                let namespace_definition = handle_namespace_definition(child, src.clone());
                result += format!("{}\n\n", namespace_definition).as_str();
                last_group_kind = "namespace_definition".to_string();
            },
            "alias_declaration" => {
                if last_group_kind != "alias_declaration" { result += "\n"; }
                let alias_declaration = handle_alias_declaration(child, src.clone());
                result += format!("{}\n\n", alias_declaration).as_str();
                last_group_kind = "alias_declaration".to_string();
            },
            "static_assert_declaration" => {
                if last_group_kind != "static_assert_declaration" { result += "\n"; }
                let static_assert_declaration = handle_static_assert_declaration(child, src.clone());
                result += format!("{}\n\n", static_assert_declaration).as_str();
                last_group_kind = "static_assert_declaration".to_string();
            },
            "namespace" => {
                if last_group_kind != "namespace" { result += "\n"; }
                let namespace = handle_namespace(child, src.clone());
                result += format!("{}\n\n", namespace).as_str();
                last_group_kind = "namespace".to_string();
            },
            "identifier" => {
                if last_group_kind != "identifier" { result += "\n"; }
                let identifier = handle_identifier(child, src.clone());
                result += format!("{}\n\n", identifier).as_str();
                last_group_kind = "identifier".to_string();
            },
            "function_declarator" => {
                if last_group_kind != "function_declarator" { result += "\n"; }
                let function_declarator = handle_function_declarator(child, src.clone());
                result += format!("{}\n\n", function_declarator).as_str();
                last_group_kind = "function_declarator".to_string();
            },
            ";" => (), // handled in functions above
            "{" => (), // handled in functions above
            _ => println!("Unknown cpp grammar name 1: {}\n", &child.grammar_name()),
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
    result = result.trim_start().to_string();
    return result;
}

fn try_split_leading_statement_macro_cpp(children: &[Node], idx: usize, src: &str) -> Option<(String, usize)> {
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

    // If Tree-sitter stopped `node` at `if (cond)` / `for (...)` / `while (...)` before its `{ ... }` body
    // (or before a trailing `else`), absorb those continuation siblings into the re-parsed slice.
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
    let tree = cpp_format::parse_existing_cpp_file(wrapped.clone());
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
        // Distinguish `MACRO_GUARD\nType var = ...;` or `MACRO_GUARD\nCALL(...);` from `MY_TYPE\nvar = 0;`
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
            if let Some((split_out, new_end_idx)) = try_split_leading_statement_macro_cpp(&children, idx, &src) {
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
            "alias_declaration" => {
                let mut alias_declaration = handle_alias_declaration(node, src.clone());
                alias_declaration = utils::add_all_leading_tabs(alias_declaration);
                result += format!("{}\n", alias_declaration).as_str();
            },
            "using_declaration" => {
                let mut using_declaration = handle_using_declaration(node, src.clone());
                using_declaration = utils::add_all_leading_tabs(using_declaration);
                result += format!("{}\n", using_declaration).as_str();
            },
            "static_assert_declaration" => {
                let mut static_assert_declaration = handle_static_assert_declaration(node, src.clone());
                static_assert_declaration = utils::add_all_leading_tabs(static_assert_declaration);
                result += format!("{}\n", static_assert_declaration).as_str();
            },
            "union_specifier" => {
                let union_specifier = handle_union_specifier(node, src.clone());
                result += format!("\t{}\n", union_specifier).as_str();
            },
            "enum_specifier" => {
                let enum_specifier = handle_enum_specifier(node, src.clone());
                result += format!("\t{}\n", enum_specifier).as_str();
            },
            "class_specifier" => {
                let class_specifier = handle_class_specifier(node, src.clone());
                result += format!("\t{}\n", class_specifier).as_str();
            },
            "throw_statement" => {
                let throw_statement = handle_throw_statement(node, src.clone());
                result += format!("\t{}\n", throw_statement).as_str();
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
    let in_for_loop = root.parent().map(|p| p.grammar_name() == "for_statement" || p.grammar_name() == "for_range_loop").unwrap_or(false);
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
        let is_declarator = matches!(
            node.grammar_name(),
            "init_declarator"
                | "pointer_declarator"
                | "reference_declarator"
                | "array_declarator"
                | "function_declarator"
                | "parenthesized_declarator"
                | "structured_binding_declarator"
                | "operator_cast"
        ) || node.kind() == "identifier";

        let piece = match node.grammar_name() {
            "compound_statement" => Some(handle_compound_statement(node.clone(), src.clone())),
            "init_declarator" => Some(handle_init_declarator(node, src.clone())),
            "primitive_type" => Some(handle_primitive_type(node, src.clone())),
            "identifier" => Some(handle_identifier(node, src.clone())),
            "function_declarator" => Some(handle_function_declarator(node, src.clone())),
            "parenthesized_declarator" => Some(handle_parenthesized_declarator(node, src.clone())),
            "reference_declarator" => Some(handle_reference_declarator(node, src.clone())),
            "structured_binding_declarator" => Some(handle_structured_binding_declarator(node, src.clone())),
            "type_qualifier" => Some(handle_type_qualifier(node, src.clone())),
            "ERROR" => Some(handle_error(node, src.clone())),
            "pointer_declarator" => Some(handle_pointer_declarator(node, src.clone())),
            "array_declarator" => Some(handle_array_declarator(node, src.clone())),
            "storage_class_specifier" => Some(handle_storage_class_specifier(node, src.clone())),
            "explicit_function_specifier" => Some(handle_explicit_function_specifier(node, src.clone())),
            "struct_specifier" => Some(handle_struct_specifier(node, src.clone())),
            "union_specifier" => Some(handle_union_specifier(node, src.clone())),
            "enum_specifier" => Some(handle_enum_specifier(node, src.clone())),
            "class_specifier" => Some(handle_class_specifier(node, src.clone())),
            "sized_type_specifier" => Some(handle_sized_type_specifier(node, src.clone())),
            "template_type" => Some(handle_template_type(node, src.clone())),
            "qualified_identifier" => Some(handle_qualified_identifier(node, src.clone())),
            "operator_cast" => Some(handle_operator_cast(node, src.clone())),
            "placeholder_type_specifier" => Some(handle_placeholder_type_specifier(node, src.clone()).trim().to_string()),
            "decltype" => Some(handle_decltype(node, src.clone())),
            "default_method_clause" => Some(handle_default_method_clause(node, src.clone())),
            "delete_method_clause" => Some(handle_delete_method_clause(node, src.clone())),
            "attribute_specifier" | "attribute_declaration" => Some(handle_attribute(node, src.clone())),
            "virtual" => Some("virtual".to_string()),
            "inline" => Some("inline".to_string()),
            "constexpr" => Some("constexpr".to_string()),
            "friend" => Some("friend".to_string()),
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
                println!("You shouldn't be here (cpp declaration): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or(""));
                None
            },
        };

        if let Some(p) = piece {
            if !seen_declarator && !is_declarator {
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
            if line.contains("&") {
                line = utils::remove_reference_spaces(line);
            }
            lines.push(line);
        }
        return lines.join("\n");
    }

    result = parts.join(" ");
    if result.contains(",") { result = utils::remove_whitespace_before_commas(&result); }
    result = utils::remove_unnecessary_spaces(&result);
    if result.contains("= \t[") {
        let idx = result.find("= \t[").unwrap();
        result.remove(idx+2);
    }
    if result.contains("*") { result = utils::remove_pointer_spaces(result); }
    if result.contains("&") { result = utils::remove_reference_spaces(result); }
    if result.contains("::") && !result.contains("operator") { result = utils::remove_object_constructor_space(result); }
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
                let error = handle_error(node, src.clone());
                result += format!("{} ", error).as_str();
            },
            "function_declarator" => {
                let function_declarator = handle_function_declarator(node, src.clone());
                result += format!("{}\n", function_declarator).as_str();
            },
            "default_method_clause" => {
                let default_method_clause = handle_default_method_clause(node, src.clone());
                result = result.trim_end().to_string();
                result += format!("{}\n", default_method_clause).as_str();
            },
            "delete_method_clause" => {
                let delete_method_clause = handle_delete_method_clause(node, src.clone());
                result = result.trim_end().to_string();
                result += format!("{}\n", delete_method_clause).as_str();
            },
            "explicit_function_specifier" => {
                let explicit_function_specifier = handle_explicit_function_specifier(node, src.clone());
                result += format!("{} ", explicit_function_specifier).as_str();
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
            "class_specifier" => {
                let class_specifier = handle_class_specifier(node.clone(), src.clone());
                result += format!("{} ", class_specifier.trim_end()).as_str();
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
                result = result.trim_end().to_string();
                result += format!("{}\n", reference_declarator).as_str();
            },
            "optional_parameter_declaration" => {
                let optional_parameter_declaration = handle_optional_parameter_declaration(node, src.clone());
                result += format!("{} ", optional_parameter_declaration).as_str();
            },
            "operator_cast" => {
                let operator_cast = handle_operator_cast(node, src.clone());
                result += format!("{} ", operator_cast).as_str();
            },
            "placeholder_type_specifier" => {
                let placeholder_type_specifier = handle_placeholder_type_specifier(node, src.clone());
                result += format!("{} ", placeholder_type_specifier.trim()).as_str();
            },
            "field_initializer_list" => {
                let field_initializer_list = handle_field_initializer_list(node, src.clone());
                result += format!("{} ", field_initializer_list).as_str();
                result = result.trim_end().to_string();
                result += "\n";
            },
            "comment" => {
                let comment = handle_comment(node, src.clone());
                utils::append_comment_to_result(&mut result, &comment, node.start_byte(), &src, "", "\n", true);
            },
            "attribute_specifier" | "attribute_declaration" => {
                let attribute = handle_attribute(node, src.clone());
                result += format!("{} ", attribute).as_str();
            },
            "virtual" => result += "virtual ",
            "inline" => result += "inline ",
            "constexpr" => result += "constexpr ",
            "friend" => result += "friend ",
            _ => println!("You shouldn't be here (cpp function_definition): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    if let Some(brace_idx) = result.find("\n{") {
        let sig = utils::remove_reference_spaces(result[..brace_idx].to_string());
        result = format!("{}{}", sig, &result[brace_idx..]);
    } else {
        result = utils::remove_reference_spaces(result);
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
            "qualified_identifier" => {
                result = handle_qualified_identifier(node, src.clone());
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
            "new_expression" => {
                result = handle_new_expression(node, src.clone());
            },
            "delete_expression" => {
                result = handle_delete_expression(node, src.clone());
            },
            "throw_statement" => {
                result = handle_throw_statement(node, src.clone());
            },
            "comma_expression" => {
                result = handle_comma_expression(node, src.clone());
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
            "number_literal" => result = handle_number_literal(node, src.clone()),
            "string_literal" => result = handle_string_literal(node, src.clone()),
            "char_literal" => result = handle_char_literal(node, src.clone()),
            "true" => result = "true".to_string(),
            "false" => result = "false".to_string(),
            "null" => result = handle_null(node, src.clone()),
            "this" => result = "this".to_string(),
            ";" => {
                if !node.is_missing() {
                    result += ";";
                }
            },
            _ => println!("You shouldn't be here (cpp expression_statement): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            "this" => parts.push("this".to_string()),
            "true" => parts.push("true".to_string()),
            "false" => parts.push("false".to_string()),
            "null" => parts.push(handle_null(node, src.clone())),
            "assignment_expression" => {
                let assignment_expression = handle_inner_assignment_expression(node, src.clone());
                parts.push(assignment_expression);
            },
            "call_expression" => {
                let call_expression = handle_call_expression(node, src.clone());
                parts.push(call_expression);
            },
            "cast_expression" => {
                let cast_expression = handle_cast_expression(node, src.clone());
                parts.push(cast_expression);
            },
            "pointer_expression" => {
                let pointer_expression = handle_pointer_expression(node, src.clone());
                parts.push(pointer_expression);
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
            "raw_string_literal" => {
                let raw_string_literal = handle_raw_string_literal(node, src.clone());
                parts.push(raw_string_literal);
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
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(node, src.clone());
                parts.push(parenthesized_expression);
            },
            "sizeof_expression" => {
                let sizeof_expression = handle_sizeof_expression(node, src.clone());
                parts.push(sizeof_expression);
            },
            "new_expression" => {
                let new_expression = handle_new_expression(node, src.clone());
                parts.push(new_expression);
            },
            "lambda_expression" => {
                let lambda_expression = handle_lambda_expression(node, src.clone());
                parts.push(lambda_expression);
            },
            "compound_literal_expression" => {
                let compound_literal_expression = handle_compound_literal_expression(node, src.clone());
                parts.push(compound_literal_expression);
            },
            "initializer_list" => {
                let initializer_list = handle_initializer_list(node, src.clone());
                if parts.last().map(|s| s == "=" || s == "return" || s == "?" || s == ":").unwrap_or(false) || parts.is_empty() {
                    parts.push(initializer_list);
                } else if let Some(last) = parts.last_mut() {
                    *last = format!("{}{}", last, initializer_list);
                }
            },
            "conditional_expression" => {
                let conditional_expression = handle_conditional_expression(node, src.clone());
                parts.push(conditional_expression);
            },
            "qualified_identifier" => {
                let qualified_identifier = handle_qualified_identifier(node, src.clone());
                parts.push(qualified_identifier);
            },
            _ => println!("You shouldn't be here (cpp assignment_expression): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap()),
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
            "qualified_identifier" => {
                let qualified_identifier = handle_qualified_identifier(node, src.clone());
                parts.push(qualified_identifier);
            },
            "update_expression" => {
                let update_expression = handle_update_expression(node, src.clone());
                parts.push(update_expression);
            },
            "new_expression" => {
                let new_expression = handle_new_expression(node, src.clone());
                parts.push(new_expression);
            },
            "lambda_expression" => {
                let lambda_expression = handle_lambda_expression(node, src.clone());
                parts.push(lambda_expression);
            },
            "compound_literal_expression" => {
                let compound_literal_expression = handle_compound_literal_expression(node, src.clone());
                parts.push(compound_literal_expression);
            },
            "initializer_list" => {
                let initializer_list = handle_initializer_list(node, src.clone());
                if parts.last().map(|s| s == "=" || s == "return" || s == "?" || s == ":").unwrap_or(false) || parts.is_empty() {
                    parts.push(initializer_list);
                } else if let Some(last) = parts.last_mut() {
                    *last = format!("{}{}", last, initializer_list);
                }
            },
            "concatenated_string" => {
                let concatenated_string = handle_concatenated_string(node, src.clone());
                parts.push(concatenated_string);
            },
            "raw_string_literal" => {
                let raw_string_literal = handle_raw_string_literal(node, src.clone());
                parts.push(raw_string_literal);
            },
            "ERROR" => {
                let error = handle_error(node, src.clone());
                parts.push(error);
            },
            "this" => parts.push("this".to_string()),
            "true" => parts.push("true".to_string()),
            "false" => parts.push("false".to_string()),
            "null" => parts.push(handle_null(node, src.clone())),
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
            _ => println!("You shouldn't be here (cpp inner_assignment_expression): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            if let Some((split_out, new_end_idx)) = try_split_leading_statement_macro_cpp(&children, idx, &src) {
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
            "throw_statement" => {
                let throw_statement = handle_throw_statement(node, src.clone());
                parts.push(format!("\t{}", throw_statement));
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
            "try_statement" => {
                let mut try_statement = handle_try_statement(node, src.clone());
                try_statement = utils::add_all_leading_tabs(try_statement);
                parts.push(try_statement);
            },
            "alias_declaration" => {
                let mut alias_declaration = handle_alias_declaration(node, src.clone());
                alias_declaration = utils::add_all_leading_tabs(alias_declaration);
                parts.push(alias_declaration);
            },
            "using_declaration" => {
                let mut using_declaration = handle_using_declaration(node, src.clone());
                using_declaration = utils::add_all_leading_tabs(using_declaration);
                parts.push(using_declaration);
            },
            "static_assert_declaration" => {
                let mut static_assert_declaration = handle_static_assert_declaration(node, src.clone());
                static_assert_declaration = utils::add_all_leading_tabs(static_assert_declaration);
                parts.push(static_assert_declaration);
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
            "class_specifier" => {
                let class_specifier = handle_class_specifier(node, src.clone());
                parts.push(format!("\t{}", class_specifier));
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
            "preproc_if" => {
                let mut preproc_if = handle_preproc_if(node, src.clone());
                preproc_if = utils::add_all_leading_tabs(preproc_if);
                parts.push(preproc_if);
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
            "primitive_type" => {
                let primitive_type = handle_primitive_type(node, src.clone());
                result += primitive_type.as_str();
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
            "template_function" => {
                let template_function = handle_template_function(node, src.clone());
                result += template_function.as_str();
            },
            "template_type" => {
                let template_type = handle_template_type(node, src.clone());
                result += template_type.as_str();
            },
            ";" => result += ";",
            _ => println!("You shouldn't be here (cpp call_expression): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_if_statement(root: Node, src: String) -> String {
    let mut parts = Vec::<String>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "if" => parts.push("if".to_string()),
            "constexpr" => parts.push("constexpr".to_string()),
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
            "for_range_loop" => {
                let for_range_loop = handle_for_range_loop(node, src.clone());
                parts.push(for_range_loop);
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
            "throw_statement" => {
                let throw_statement = handle_throw_statement(node, src.clone());
                parts.push(throw_statement);
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
            ";" => parts.push(";".to_string()),
            _ => println!("You shouldn't be here (cpp if_statement): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            "for_range_loop" => {
                let for_range_loop = handle_for_range_loop(node, src.clone());
                pieces.push(for_range_loop);
            },
            "while_statement" => {
                let while_statement = handle_while_statement(node, src.clone());
                pieces.push(while_statement);
            },
            "do_statement" => {
                let do_statement = handle_do_statement(node, src.clone());
                pieces.push(do_statement);
            },
            "continue_statement" => pieces.push("continue;".to_string()),
            "break_statement" => pieces.push("break;".to_string()),
            "goto_statement" => {
                let goto_statement = handle_goto_statement(node, src.clone());
                pieces.push(goto_statement);
            },
            "throw_statement" => {
                let throw_statement = handle_throw_statement(node, src.clone());
                pieces.push(throw_statement);
            },
            "comment" => {
                let comment = handle_comment(node, src.clone());
                utils::push_expr_comment(&mut pieces, &comment, node.start_byte(), node.end_byte(), &src);
            },
            ";" => pieces.push(";".to_string()),
            _ => println!("You shouldn't be here (cpp else_clause): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            _ => println!("You shouldn't be here (cpp switch_statement): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
                utils::append_comment_to_result(&mut result, &comment, node.start_byte(), &src, "\t", "\n", true);
            },
            "continue_statement" => {
                result += "\tcontinue;";
            },
            "while_statement" => {
                let while_statement = handle_while_statement(node, src.clone());
                result += format!("\t{}\n", while_statement).as_str();
            },
            "for_statement" => {
                let for_statement = handle_for_statement(node, src.clone());
                result += format!("\t{}\n", for_statement).as_str();
            },
            "for_range_loop" => {
                let for_range_loop = handle_for_range_loop(node, src.clone());
                result += format!("\t{}\n", for_range_loop).as_str();
            },
            "do_statement" => {
                let do_statement = handle_do_statement(node, src.clone());
                result += format!("\t{}\n", do_statement).as_str();
            },
            "labeled_statement" => {
                let labeled_statement = handle_labeled_statement(node, src.clone());
                result += format!("\t{}\n", labeled_statement).as_str();
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
            ":" => {
                result = result.trim_end().to_string();
                result += ":\n"
            },
            ";" => result += ";",
            _ => println!("You shouldn't be here (cpp case_statement): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            _ => println!("You shouldn't be here (cpp goto_statement): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            "if_statement" => {
                let if_statement = handle_if_statement(node, src.clone());
                parts.push(if_statement);
            },
            "for_statement" => {
                let for_statement = handle_for_statement(node, src.clone());
                parts.push(for_statement);
            },
            "for_range_loop" => {
                let for_range_loop = handle_for_range_loop(node, src.clone());
                parts.push(for_range_loop);
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
            _ => println!("You shouldn't be here (cpp while_statement): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let result = parts.join(" ");
    return result;
}

fn handle_null(root: Node, src: String) -> String {
    return root.utf8_text(src.as_bytes()).unwrap_or("nullptr").to_string();
}

fn handle_argument_list(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "," => result += ", ",
            "(" => result += "(",
            ")" => result += ")",
            "null" => result += handle_null(node, src.clone()).as_str(),
            "true" => result += "true",
            "false" => result += "false",
            "this" => result += "this",
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
            "parameter_pack_expansion" => {
                let parameter_pack_expansion = handle_parameter_pack_expansion(node, src.clone());
                result += parameter_pack_expansion.as_str();
            },
            "lambda_expression" => {
                let mut lambda_expression = handle_lambda_expression(node, src.clone());
                lambda_expression = utils::add_all_leading_tabs(lambda_expression).trim_start().to_string();
                result += lambda_expression.as_str();
            },
            "compound_literal_expression" => {
                let compound_literal_expression = handle_compound_literal_expression(node, src.clone());
                result += compound_literal_expression.as_str();
            },
            "offsetof_expression" => {
                let offsetof_expression = handle_offsetof_expression(node, src.clone());
                result += offsetof_expression.as_str();
            },
            "template_function" => {
                let template_function = handle_template_function(node, src.clone());
                result += template_function.as_str();
            },
            "type_descriptor" => {
                let type_descriptor = handle_type_descriptor(node, src.clone());
                result += type_descriptor.as_str();
            },
            "ERROR" => {
                let error = handle_error(node, src.clone());
                result += error.as_str();
            },
            _ => println!("You shouldn't be here (cpp argument_list): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            "compound_literal_expression" => {
                let compound_literal_expression = handle_compound_literal_expression(node, src.clone());
                parts.push(compound_literal_expression);
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
            "qualified_identifier" => {
                let qualified_identifier = handle_qualified_identifier(node, src.clone());
                parts.push(qualified_identifier);
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
                if parts.last().map(|s| s == "=" || s == "return" || s == "?" || s == ":").unwrap_or(false) || parts.is_empty() {
                    parts.push(initializer_list);
                } else if let Some(last) = parts.last_mut() {
                    *last = format!("{}{}", last, initializer_list);
                }
            },
            "sizeof_expression" => {
                let sizeof_expression = handle_sizeof_expression(node, src.clone());
                parts.push(sizeof_expression);
            },
            "update_expression" => {
                let update_expression = handle_update_expression(node, src.clone());
                parts.push(update_expression);
            },
            "assignment_expression" => {
                let assignment_expression = handle_assignment_expression(node, src.clone());
                parts.push(assignment_expression);
            },
            "new_expression" => {
                let new_expression = handle_new_expression(node, src.clone());
                parts.push(new_expression);
            },
            "lambda_expression" => {
                let lambda_expression = handle_lambda_expression(node, src.clone());
                parts.push(lambda_expression);
            },
            "comma_expression" => {
                let comma_expression = handle_comma_expression(node, src.clone());
                parts.push(comma_expression);
            },
            "concatenated_string" => {
                let concatenated_string = handle_concatenated_string(node, src.clone());
                parts.push(concatenated_string);
            },
            "raw_string_literal" => {
                let raw_string_literal = handle_raw_string_literal(node, src.clone());
                parts.push(raw_string_literal);
            },
            "return" => {
                parts.push("return".to_string());
            },
            "null" => {
                parts.push(handle_null(node, src.clone()));
            },
            "this" => {
                parts.push("this".to_string());
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
                println!("You shouldn't be here (cpp return_statement): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or(""));
            },
        }
    }
    result = parts.join(" ");
    result = utils::remove_unnecessary_spaces(&result);
    return result;
}

fn is_comment_container_cpp(grammar_name: &str) -> bool {
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

fn is_valid_ref_sibling_cpp(node: Node, src: &str) -> bool {
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
            | "access_specifier"
            | "labeled_statement"
    ) || g.starts_with("preproc_") || g.starts_with('#')
    {
        return false;
    }
    utils::trailing_comment_gap(node.start_byte(), src).is_none()
}

fn find_comment_ref_byte_cpp(root: Node, src: &str) -> (Option<usize>, bool) {
    let Some(direct_parent) = root.parent() else {
        return (None, true);
    };
    if direct_parent.grammar_name() == "translation_unit" {
        return (None, true);
    }

    if !is_comment_container_cpp(direct_parent.grammar_name()) {
        let mut cur = direct_parent;
        while let Some(p) = cur.parent() {
            if is_comment_container_cpp(p.grammar_name()) {
                if p.grammar_name() == "translation_unit" {
                    return (Some(cur.start_byte()), false);
                }
                if is_valid_ref_sibling_cpp(cur, src) {
                    return (Some(cur.start_byte()), false);
                }
                break;
            }
            cur = p;
        }
    }

    let mut sib_opt = root.next_sibling();
    while let Some(sib) = sib_opt {
        if is_valid_ref_sibling_cpp(sib, src) {
            return (Some(sib.start_byte()), false);
        }
        sib_opt = sib.next_sibling();
    }

    sib_opt = root.prev_sibling();
    while let Some(sib) = sib_opt {
        if is_valid_ref_sibling_cpp(sib, src) {
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
            _ => println!("You shouldn't be here (cpp comment): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap()),
        }
    }
    let result: String;
    if parts.len() == 0 {
        result = root.utf8_text(src.as_bytes()).unwrap().to_string();
    }
    else {
        result = parts.join(" ");
    }
    let (ref_byte, is_top_level) = find_comment_ref_byte_cpp(root, &src);
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
            result += node.utf8_text(src.as_bytes()).unwrap();
        }
        else {
            let mut temp = String::new();
            for subnode in node.children(&mut node.walk()) {
                let text = subnode.utf8_text(src.as_bytes()).unwrap();
                match subnode.grammar_name() {
                    "," => temp += ", ",
                    "~" => temp += "~",
                    "(" => temp += "(",
                    ")" => temp += ")",
                    "()" => temp += "()",
                    "::" => temp += "::",
                    "<<" => temp += "operator<<",
                    ">>" => temp += "operator>>",
                    "^" => temp += "operator^",
                    "&" => temp += "operator&",
                    "<" => temp += "operator<",
                    "<=" => temp += "operator<=",
                    ">" => temp += "operator>",
                    ">=" => temp += "operator>=",
                    "+" => temp += "operator+",
                    "-" => temp += "operator-",
                    "*" => temp += "operator*",
                    "/" => temp += "operator/",
                    "%" => temp += "operator%",
                    "&&" => temp += "operator&&",
                    "|" => temp += "operator|",
                    "||" => temp += "operator||",
                    "!" => temp += "operator!",
                    "[]" => temp += "operator[]",
                    "[" => temp += "[",
                    "]" => temp += "]",
                    "=" => temp += "operator=",
                    "==" => temp += "operator==",
                    "!=" => temp += "operator!=",
                    "|=" => temp += "operator|=",
                    "&=" => temp += "operator&=",
                    "^=" => temp += "operator^=",
                    "+=" => temp += "operator+=",
                    "-=" => temp += "operator-=",
                    "*=" => temp += "operator*=",
                    "/=" => temp += "operator/=",
                    "%=" => temp += "operator%=",
                    "<<=" => temp += "operator<<=",
                    ">>=" => temp += "operator>>=",
                    "<=>" => temp += "operator<=>",
                    "->" => temp += "operator->",
                    "operator" => temp += "operator",
                    "const" => temp += " const",
                    "override" => temp += " override",
                    "final" => temp += " final",
                    "noexcept" => temp += " noexcept",
                    "pointer_declarator" => {
                        let pointer_declarator = handle_pointer_declarator(subnode, src.clone());
                        temp += pointer_declarator.as_str();
                    },
                    "reference_declarator" => {
                        let reference_declarator = handle_reference_declarator(subnode, src.clone());
                        temp += reference_declarator.as_str();
                    },
                    "parenthesized_declarator" => {
                        let parenthesized_declarator = handle_parenthesized_declarator(subnode, src.clone());
                        temp += parenthesized_declarator.as_str();
                    },
                    "trailing_return_type" => {
                        let trailing_return_type = handle_trailing_return_type(subnode, src.clone());
                        temp += format!(" {}", trailing_return_type).as_str();
                    },
                    "type_descriptor" => {
                        let type_descriptor = handle_type_descriptor(subnode, src.clone());
                        temp += type_descriptor.as_str();
                    },
                    "argument_list" => {
                        let argument_list = handle_argument_list(subnode, src.clone());
                        temp += argument_list.as_str();
                    },
                    "__attribute__" => {
                        let attribute = handle_attribute(subnode, src.clone());
                        temp += attribute.as_str();
                    },
                    "pointer_type_declarator" => {
                        let pointer_type_declarator = handle_pointer_type_declarator(subnode, src.clone());
                        temp += pointer_type_declarator.as_str();
                    },
                    "parameter_declaration" => {
                        let mut parameter_declaration = handle_parameter_declaration(subnode, src.clone());
                        parameter_declaration = utils::remove_pointer_spaces(parameter_declaration);
                        temp += parameter_declaration.as_str();
                    },
                    "variadic_parameter_declaration" => {
                        let mut variadic_parameter_declaration = handle_variadic_parameter_declaration(subnode, src.clone());
                        variadic_parameter_declaration = utils::remove_pointer_spaces(variadic_parameter_declaration);
                        temp += variadic_parameter_declaration.as_str();
                    },
                    "optional_parameter_declaration" => {
                        let mut optional_parameter_declaration = handle_optional_parameter_declaration(subnode, src.clone());
                        optional_parameter_declaration = utils::remove_pointer_spaces(optional_parameter_declaration);
                        temp += optional_parameter_declaration.as_str();
                    },
                    "identifier" => {
                        let identifier = handle_identifier(subnode, src.clone());
                        temp += identifier.as_str();
                    },
                    "qualified_identifier" => {
                        let qualified_identifier = handle_qualified_identifier(subnode, src.clone());
                        temp += qualified_identifier.as_str();
                    },
                    "template_function" => {
                        let template_function = handle_template_function(subnode, src.clone());
                        temp += template_function.as_str();
                    },
                    "operator_name" => {
                        let operator_name = handle_operator_name(subnode, src.clone());
                        temp += operator_name.as_str();
                    },
                    "variadic_parameter" => {
                        let variadic_parameter = handle_variadic_parameter(subnode, src.clone());
                        temp += variadic_parameter.as_str();
                    },
                    "comment" => {
                        let comment = handle_comment(subnode, src.clone());
                        temp += format!("{} ", comment).as_str();
                    },
                    "destructor_name" => {
                        let destructor_name = handle_destructor_name(subnode, src.clone());
                        temp += destructor_name.as_str();
                    },
                    "ERROR" => {
                        let error = handle_error(subnode, src.clone());
                        temp += error.as_str();
                    },
                    "..." => temp += " ...",
                    _ => println!("You shouldn't be here (cpp function_declarator): {}: {}\n", subnode.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            "qualified_identifier" => {
                let qualified_identifier = handle_qualified_identifier(node, src.clone());
                result += format!(" {}", qualified_identifier).as_str();
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
            "reference_declarator" => {
                let reference_declarator = handle_reference_declarator(node, src.clone());
                result += reference_declarator.as_str();
            },
            "ERROR" => {
                let error = handle_error(node, src.clone());
                result += format!(" {}", error).as_str();
            },
            _ => println!("You shouldn't be here (cpp pointer_declarator): {}: {}\n", node.kind(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            _ => println!("You shouldn't be here (cpp identifier): {}: {}\n", root.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            "offsetof_expression" => {
                let offsetof_expression = handle_offsetof_expression(node, src.clone());
                parts.push(offsetof_expression);
            },
            "conditional_expression" => {
                let conditional_expression = handle_conditional_expression(node, src.clone());
                parts.push(conditional_expression);
            },
            "compound_literal_expression" => {
                let compound_literal_expression = handle_compound_literal_expression(node, src.clone());
                parts.push(compound_literal_expression);
            },
            "new_expression" => {
                let new_expression = handle_new_expression(node, src.clone());
                parts.push(new_expression);
            },
            "comment" => {
                let comment = handle_comment(node, src.clone());
                utils::push_expr_comment(&mut parts, &comment, node.start_byte(), node.end_byte(), &src);
            },
            "null" => parts.push(handle_null(node, src.clone())),
            "true" => parts.push("true".to_string()),
            "false" => parts.push("false".to_string()),
            "this" => parts.push("this".to_string()),
            "+" => parts.push("+".to_string()),
            "-" => parts.push("-".to_string()),
            "*" => parts.push("*".to_string()),
            "/" => parts.push("/".to_string()),
            "%" => parts.push("%".to_string()),
            ">" => parts.push(">".to_string()),
            "<" => parts.push("<".to_string()),
            ">=" => parts.push(">=".to_string()),
            "<=" => parts.push("<=".to_string()),
            "<=>" => parts.push("<=>".to_string()),
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
            _ => println!("You shouldn't be here (cpp binary_expression): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            "unary_expression" => {
                let unary_expression = handle_unary_expression(node, src.clone());
                parts.push(unary_expression);
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
            "cast_expression" => {
                let cast_expression = handle_cast_expression(node, src.clone());
                parts.push(cast_expression);
            },
            "conditional_expression" => {
                let conditional_expression = handle_conditional_expression(node, src.clone());
                parts.push(conditional_expression);
            },
            "offsetof_expression" => {
                let offsetof_expression = handle_offsetof_expression(node, src.clone());
                parts.push(offsetof_expression);
            },
            "compound_literal_expression" => {
                let compound_literal_expression = handle_compound_literal_expression(node, src.clone());
                parts.push(compound_literal_expression);
            },
            "new_expression" => {
                let new_expression = handle_new_expression(node, src.clone());
                parts.push(new_expression);
            },
            "comment" => {
                let comment = handle_comment(node, src.clone());
                utils::push_expr_comment(&mut parts, &comment, node.start_byte(), node.end_byte(), &src);
            },
            "null" => parts.push(handle_null(node, src.clone())),
            "this" => parts.push("this".to_string()),
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
            "<=>" => parts.push("<=>".to_string()),
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
            _ => println!("You shouldn't be here (cpp inner_binary_expression): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            "qualified_identifier" => {
                let qualified_identifier = handle_qualified_identifier(node, src.clone());
                result += qualified_identifier.as_str();
            },
            "pointer_declarator" => {
                let pointer_declarator = handle_pointer_declarator(node, src.clone());
                result += pointer_declarator.as_str();
            },
            "reference_declarator" => {
                let reference_declarator = handle_reference_declarator(node, src.clone());
                result += reference_declarator.as_str();
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
            "pointer_expression" => {
                let pointer_expression = handle_pointer_expression(node, src.clone());
                result += pointer_expression.as_str();
            },
            "(" => result += "(",
            ")" => result += ")",
            _ => println!("You shouldn't be here (cpp parenthesized_declarator): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            "update_expression" => {
                let update_expression = handle_update_expression(node, src.clone());
                result += update_expression.as_str();
            },
            "binary_expression" => {
                let binary_expression = handle_binary_expression(node, src.clone());
                result += binary_expression.as_str();
            },
            "unary_expression" => {
                let unary_expression = handle_unary_expression(node, src.clone());
                result += unary_expression.as_str();
            },
            "field_expression" => {
                let field_expression = handle_field_expression(node, src.clone());
                result += field_expression.as_str();
            },
            "subscript_expression" => {
                let subscript_expression = handle_subscript_expression(node, src.clone());
                result += subscript_expression.as_str();
            },
            "sizeof_expression" => {
                let sizeof_expression = handle_sizeof_expression(node, src.clone());
                result += sizeof_expression.as_str();
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(node, src.clone());
                result += parenthesized_expression.as_str();
            },
            "comma_expression" => {
                let comma_expression = handle_comma_expression(node, src.clone());
                result += comma_expression.as_str();
            },
            "compound_literal_expression" => {
                let compound_literal_expression = handle_compound_literal_expression(node, src.clone());
                result += compound_literal_expression.as_str();
            },
            "lambda_expression" => {
                let lambda_expression = handle_lambda_expression(node, src.clone());
                result += lambda_expression.as_str();
            },
            "cast_expression" => {
                let cast_expression = handle_cast_expression(node, src.clone());
                result += cast_expression.as_str();
            },
            "assignment_expression" => {
                let assignment_expression = handle_assignment_expression(node, src.clone());
                result += assignment_expression.as_str();
            },
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                result += identifier.as_str();
            },
            "qualified_identifier" => {
                let qualified_identifier = handle_qualified_identifier(node, src.clone());
                result += qualified_identifier.as_str();
            },
            "pointer_expression" => {
                let pointer_expression = handle_pointer_expression(node, src.clone());
                result += pointer_expression.as_str();
            },
            "conditional_expression" => {
                let conditional_expression = handle_conditional_expression(node, src.clone());
                result += conditional_expression.as_str();
            },
            "this" => result += "this",
            "true" => result += "true",
            "false" => result += "false",
            "null" => result += handle_null(node, src.clone()).as_str(),
            "ERROR" => {
                let error = handle_error(node, src.clone());
                result += error.as_str();
            },
            "(" => result += "(",
            ")" => result += ")",
            _ => println!("You shouldn't be here (cpp parenthesized_expression): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            "this" => result += "this",
            "&" => result += "&",
            ";" => result += ";",
            _ => println!("You shouldn't be here (cpp pointer_expression): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            "qualified_identifier" => {
                let qualified_identifier = handle_qualified_identifier(node, src.clone());
                parts.push(qualified_identifier);
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
            "array_declarator" => {
                let array_declarator = handle_array_declarator(node, src.clone());
                parts.push(array_declarator);
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
            "template_type" => {
                let template_type = handle_template_type(node, src.clone());
                parts.push(template_type);
            },
            "struct_specifier" => {
                let struct_specifier = handle_struct_specifier(node, src.clone());
                parts.push(struct_specifier);
            },
            "union_specifier" => {
                let union_specifier = handle_union_specifier(node, src.clone());
                parts.push(union_specifier);
            },
            "enum_specifier" => {
                let enum_specifier = handle_enum_specifier(node, src.clone());
                parts.push(enum_specifier);
            },
            "class_specifier" => {
                let class_specifier = handle_class_specifier(node, src.clone());
                parts.push(class_specifier);
            },
            "placeholder_type_specifier" => {
                let placeholder_type_specifier = handle_placeholder_type_specifier(node, src.clone());
                parts.push(placeholder_type_specifier.trim().to_string());
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
            "abstract_reference_declarator" => {
                let abstract_reference_declarator = handle_abstract_reference_declarator(node, src.clone());
                parts.push(abstract_reference_declarator);
            },
            "reference_declarator" => {
                let reference_declarator = handle_reference_declarator(node, src.clone());
                parts.push(reference_declarator);
            },
            "ERROR" => {
                let error = handle_error(node, src.clone());
                parts.push(error);
            },
            _ => println!("You shouldn't be here (cpp parameter_declaration): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let mut result = parts.join(" ");
    result = utils::remove_unnecessary_spaces(&result);
    result = utils::remove_reference_spaces(result);
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
            "qualified_identifier" => {
                let qualified_identifier = handle_qualified_identifier(node, src.clone());
                result += qualified_identifier.as_str();
            },
            "unary_expression" => {
                let unary_expression = handle_unary_expression(node, src.clone());
                result += unary_expression.as_str();
            },
            "conditional_expression" => {
                let conditional_expression = handle_conditional_expression(node, src.clone());
                result += conditional_expression.as_str();
            },
            "this" => result += "this",
            _ => println!("You shouldn't be here (cpp subscript_expression): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            "qualified_identifier" => {
                let qualified_identifier = handle_qualified_identifier(node, src.clone());
                result += qualified_identifier.as_str();
            },
            "template_method" => {
                let template_method = handle_template_method(node, src.clone());
                result += template_method.as_str();
            },
            "subscript_expression" => {
                let subscript_expression = handle_subscript_expression(node, src.clone());
                result += subscript_expression.as_str();
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
            "call_expression" => {
                let call_expression = handle_call_expression(node, src.clone());
                result += call_expression.as_str();
            },
            "field_expression" => {
                let field_expression = handle_inner_field_expression(node, src.clone());
                result += field_expression.as_str();
            },
            "destructor_name" => {
                let destructor_name = handle_destructor_name(node, src.clone());
                result += destructor_name.as_str();
            },
            "ERROR" => {
                let error = handle_error(node, src.clone());
                result += error.as_str();
            },
            "this" => result += "this",
            "->" => result += "->",
            "." => result += ".",
            _ => println!("You shouldn't be here (cpp field_expression): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            "bitfield_clause" => {
                parts.push(handle_bitfield_clause(node, src.clone()));
            },
            "parenthesized_expression" => {
                parts.push(handle_parenthesized_expression(node, src.clone()));
            },
            "primitive_type" => {
                parts.push(handle_primitive_type(node, src.clone()));
            },
            "#ifdef" => parts.push("#ifdef".to_string()),
            "#endif" => parts.push("#endif".to_string()),
            "ERROR" => {
                parts.push(node.utf8_text(src.as_bytes()).unwrap_or("").to_string());
            },
            "operator" => {
                parts.push("operator".to_string());
            },
            "switch" => {
                parts.push("switch".to_string());
            },
            "{" => parts.push("{".to_string()),
            "}" => parts.push("}".to_string()),
            ";" => parts.push(";".to_string()),
            "::" => parts.push("::".to_string()),
            ":" => parts.push(":".to_string()),
            "=" => parts.push("=".to_string()),
            "(" => parts.push("(".to_string()),
            ")" => parts.push(")".to_string()),
            "-" => parts.push("-".to_string()),
            ">" => parts.push(">".to_string()),
            "->" => parts.push("->".to_string()),
            "." => parts.push(".".to_string()),
            "?" => parts.push("?".to_string()),
            "," => parts.push(",".to_string()),
            "false" => parts.push("false".to_string()),
            "true" => parts.push("true".to_string()),
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
            "null" => result += handle_null(node, src.clone()).as_str(),
            "this" => result += "this",
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
            _ => println!("You shouldn't be here (cpp cast_expression): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            "qualified_identifier" => {
                let qualified_identifier = handle_qualified_identifier(node, src.clone());
                parts.push(qualified_identifier);
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
            "function_declarator" => {
                let function_declarator = handle_function_declarator(node, src.clone());
                parts.push(function_declarator);
            },
            "parenthesized_declarator" => {
                let parenthesized_declarator = handle_parenthesized_declarator(node, src.clone());
                parts.push(parenthesized_declarator);
            },
            "structured_binding_declarator" => {
                let structured_binding_declarator = handle_structured_binding_declarator(node, src.clone());
                parts.push(structured_binding_declarator);
            },
            "pointer_expression" => {
                let pointer_expression = handle_pointer_expression(node, src.clone());
                parts.push(pointer_expression);
            },
            "reference_declarator" => {
                let reference_declarator = handle_reference_declarator(node, src.clone());
                parts.push(reference_declarator);
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
                if parts.last().map(|s| s == "=" || s == "return" || s == "?" || s == ":").unwrap_or(false) || parts.is_empty() {
                    parts.push(initializer_list);
                } else if let Some(last) = parts.last_mut() {
                    *last = format!("{}{}", last, initializer_list);
                }
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
                if parts.last().map(|s| s == "=" || s == "return" || s == "?" || s == ":").unwrap_or(false) || parts.is_empty() {
                    parts.push(parenthesized_expression);
                } else if let Some(last) = parts.last_mut() {
                    *last = format!("{}{}", last, parenthesized_expression);
                }
            },
            "array_declarator" => {
                let array_declarator = handle_array_declarator(node, src.clone());
                parts.push(array_declarator);
            },
            "string_literal" => {
                let string_literal = handle_string_literal(node, src.clone());
                parts.push(string_literal);
            },
            "concatenated_string" => {
                let concatenated_string = handle_concatenated_string(node, src.clone());
                parts.push(concatenated_string);
            },
            "raw_string_literal" => {
                let raw_string_literal = handle_raw_string_literal(node, src.clone());
                parts.push(raw_string_literal);
            },
            "argument_list" => {
                let argument_list = handle_argument_list(node, src.clone());
                if parts.last().map(|s| s == "=" || s == "return" || s == "?" || s == ":").unwrap_or(false) || parts.is_empty() {
                    parts.push(argument_list);
                } else if let Some(last) = parts.last_mut() {
                    *last = format!("{}{}", last, argument_list);
                }
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
            "ERROR" => {
                let error = handle_error(node, src.clone());
                parts.push(error);
            },
            "=" => parts.push("=".to_string()),
            "this" => parts.push("this".to_string()),
            "false" => parts.push("false".to_string()),
            "true" => parts.push("true".to_string()),
            "null" => parts.push(handle_null(node, src.clone())),
            _ => println!("You shouldn't be here (cpp init_declarator): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            "qualified_identifier" => {
                let qualified_identifier = handle_qualified_identifier(node, src.clone());
                parts.push(qualified_identifier);
            },
            "primitive_type" => {
                let primitive_type = handle_primitive_type(node, src.clone());
                parts.push(primitive_type);
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
            "class_specifier" => {
                let class_specifier = handle_class_specifier(node, src.clone());
                parts.push(class_specifier.trim_end().to_string());
            },
            "sized_type_specifier" => {
                let sized_type_specifier = handle_sized_type_specifier(node, src.clone());
                parts.push(sized_type_specifier);
            },
            "template_type" => {
                let template_type = handle_template_type(node, src.clone());
                parts.push(template_type);
            },
            "type_qualifier" => {
                let type_qualifier = handle_type_qualifier(node, src.clone());
                parts.push(type_qualifier);
            },
            "abstract_reference_declarator" => {
                let abstract_reference_declarator = handle_abstract_reference_declarator(node, src.clone());
                parts.push(abstract_reference_declarator);
            },
            "decltype" => {
                let decltype = handle_decltype(node, src.clone());
                parts.push(decltype);
            },
            _ => println!("You shouldn't be here (cpp type_descriptor): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let mut result = parts.join(" ");
    result = utils::remove_pointer_spaces(result);
    result = utils::remove_reference_spaces(result);
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
                utils::append_expr_comment(&mut result, &comment, node.start_byte(), node.end_byte(), &src);
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
            "pointer_expression" => {
                let pointer_expression = handle_pointer_expression(node, src.clone());
                result += pointer_expression.as_str();
            },
            "conditional_expression" => {
                let conditional_expression = handle_conditional_expression(node, src.clone());
                result += conditional_expression.as_str();
            },
            "binary_expression" => {
                let binary_expression = handle_binary_expression(node, src.clone());
                result += binary_expression.as_str();
            },
            "initializer_pair" => {
                let initializer_pair = handle_initializer_pair(node, src.clone());
                result += initializer_pair.as_str();
            },
            "cast_expression" => {
                let cast_expression = handle_cast_expression(node, src.clone());
                result += cast_expression.as_str();
            },
            "unary_expression" => {
                let unary_expression = handle_unary_expression(node, src.clone());
                result += unary_expression.as_str();
            },
            "field_expression" => {
                let field_expression = handle_field_expression(node, src.clone());
                result += field_expression.as_str();
            },
            "subscript_expression" => {
                let subscript_expression = handle_subscript_expression(node, src.clone());
                result += subscript_expression.as_str();
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
            "concatenated_string" => {
                let concatenated_string = handle_concatenated_string(node, src.clone());
                result += concatenated_string.as_str();
            },
            "raw_string_literal" => {
                let raw_string_literal = handle_raw_string_literal(node, src.clone());
                result += raw_string_literal.as_str();
            },
            "null" => result += handle_null(node, src.clone()).as_str(),
            "true" => result += "true",
            "false" => result += "false",
            "this" => result += "this",
            "{" => result += "{",
            "}" => result += "}",
            "," => result += ", ",
            _ => println!("You shouldn't be here (cpp initializer_list): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_sizeof_expression(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "sizeof" => result += "sizeof",
            "..." => result += "...",
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
            "qualified_identifier" => {
                let qualified_identifier = handle_qualified_identifier(node, src.clone());
                result += format!(" {}", qualified_identifier).as_str();
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
            _ => println!("You shouldn't be here (cpp sizeof_expression): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    result = result.replace("( ", "(");
    return result;
}

fn handle_type_qualifier(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "const" => result = "const".to_string(),
            "constexpr" => result = "constexpr".to_string(),
            "consteval" => result = "consteval".to_string(),
            "constinit" => result = "constinit".to_string(),
            "volatile" => result = "volatile".to_string(),
            "mutable" => result = "mutable".to_string(),
            "restrict" => result = "restrict".to_string(),
            "__restrict" => result = "__restrict".to_string(),
            "__restrict__" => result = "__restrict__".to_string(),
            _ => println!("You shouldn't be here (cpp type_qualifier): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            "abstract_reference_declarator" => {
                let inner = handle_abstract_reference_declarator(node, src.clone());
                result += inner.as_str();
            },
            "type_qualifier" => {
                let type_qualifier = handle_type_qualifier(node, src.clone());
                result += format!(" {}", type_qualifier).as_str();
            },
            "*" => result += "*",
            _ => println!("You shouldn't be here (cpp abstract_pointer_declarator): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            "initializer_list" => {
                let initializer_list = handle_initializer_list(node, src.clone());
                if parts.last().map(|s| s == "=" || s == "return" || s == "?" || s == ":").unwrap_or(false) || parts.is_empty() {
                    parts.push(initializer_list);
                } else if let Some(last) = parts.last_mut() {
                    *last = format!("{}{}", last, initializer_list);
                }
            },
            "compound_literal_expression" => {
                let compound_literal_expression = handle_compound_literal_expression(node, src.clone());
                parts.push(compound_literal_expression);
            },
            "new_expression" => {
                let new_expression = handle_new_expression(node, src.clone());
                parts.push(new_expression);
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
            "raw_string_literal" => {
                let raw_string_literal = handle_raw_string_literal(node, src.clone());
                parts.push(raw_string_literal);
            },
            "null" => parts.push(handle_null(node, src.clone())),
            "this" => parts.push("this".to_string()),
            "true" => parts.push("true".to_string()),
            "false" => parts.push("false".to_string()),
            ":" => parts.push(":".to_string()),
            "?" => parts.push("?".to_string()),
            _ => println!("You shouldn't be here (cpp conditional_expression): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            "compound_literal_expression" => {
                let compound_literal_expression = handle_compound_literal_expression(node, src.clone());
                result += compound_literal_expression.as_str();
            },
            "assignment_expression" => {
                let assignment_expression = handle_assignment_expression(node, src.clone());
                result += assignment_expression.as_str();
            },
            "preproc_defined" => {
                let preproc_defined = handle_preproc_defined(node, src.clone());
                result += preproc_defined.as_str();
            },
            "qualified_identifier" => {
                let qualified_identifier = handle_qualified_identifier(node, src.clone());
                result += qualified_identifier.as_str();
            },
            "subscript_expression" => {
                let subscript_expression = handle_subscript_expression(node, src.clone());
                result += subscript_expression.as_str();
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
            "ERROR" => {
                let error = handle_error(node, src.clone());
                result += error.as_str();
            },
            "true" => result += "true",
            "false" => result += "false",
            "this" => result += "this",
            "!" => result += "!",
            "~" => result += "~",
            "-" => result += "-",
            "+" => result += "+",
            _ => println!("You shouldn't be here (cpp unary_expression): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            "qualified_identifier" => {
                let qualified_identifier = handle_qualified_identifier(node, src.clone());
                parts.push(qualified_identifier);
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
            "parenthesized_declarator" => {
                let parenthesized_declarator = handle_parenthesized_declarator(node, src.clone());
                parts.push(parenthesized_declarator);
            },
            "array_declarator" => {
                let array_declarator = handle_array_declarator(node, src.clone());
                parts.push(array_declarator);
            },
            "ERROR" => {
                let error = handle_error(node, src.clone());
                parts.push(error);
            },
            "[" => parts.push("[".to_string()),
            "]" => parts.push("]".to_string()),
            _ => println!("You shouldn't be here (cpp array_declarator): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            _ => println!("You shouldn't be here (cpp string_literal): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            _ => println!("You shouldn't be here (cpp concatenated_string): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let result = parts.join(" ");
    return result;
}

fn handle_labeled_statement(root: Node, src: String) -> String {
    let same_line = root.start_position().row == root.end_position().row;
    let sep = if same_line { " " } else { "\t" };
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
            "for_range_loop" => {
                let for_range_loop = handle_for_range_loop(node, src.clone());
                result += format!("{}{}", sep, for_range_loop).as_str();
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
                if same_line {
                    result += format!(" {}", labeled_statement.trim_start()).as_str();
                } else {
                    result += format!("\n{}", labeled_statement).as_str();
                }
            },
            "comment" => {
                let comment = handle_comment(node, src.clone());
                utils::append_comment_to_result(&mut result, &comment, node.start_byte(), &src, "\t", "\n", true);
            },
            "break_statement" => result += format!("{}break;", sep).as_str(),
            "continue_statement" => result += format!("{}continue;", sep).as_str(),
            ":" => {
                if same_line {
                    result += ":";
                } else {
                    result += ":\n";
                }
            },
            ";" => result += ";",
            _ => println!("You shouldn't be here (cpp labeled_statement): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    result = utils::add_all_leading_tabs(result);
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
            "for_range_loop" => {
                let for_range_loop = handle_for_range_loop(node, src.clone());
                vec.push(for_range_loop);
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
                if !reached_header_end { temp += update_expression.as_str(); }
                else { vec.push(update_expression); }
            },
            "assignment_expression" => {
                let assignment_expression = handle_assignment_expression(node, src.clone());
                if !reached_header_end { temp += assignment_expression.as_str(); }
                else { vec.push(assignment_expression); }
            },
            "binary_expression" => {
                let binary_expression = handle_binary_expression(node, src.clone());
                if !reached_header_end { temp += binary_expression.as_str(); }
                else { vec.push(binary_expression); }
            },
            "call_expression" => {
                let call_expression = handle_call_expression(node, src.clone());
                if !reached_header_end { temp += call_expression.as_str(); }
                else { vec.push(call_expression); }
            },
            "unary_expression" => {
                let unary_expression = handle_unary_expression(node, src.clone());
                if !reached_header_end { temp += unary_expression.as_str(); }
                else { vec.push(unary_expression); }
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(node, src.clone());
                if !reached_header_end { temp += parenthesized_expression.as_str(); }
                else { vec.push(parenthesized_expression); }
            },
            "field_expression" => {
                let field_expression = handle_field_expression(node, src.clone());
                if !reached_header_end { temp += field_expression.as_str(); }
                else { vec.push(field_expression); }
            },
            "subscript_expression" => {
                let subscript_expression = handle_subscript_expression(node, src.clone());
                if !reached_header_end { temp += subscript_expression.as_str(); }
                else { vec.push(subscript_expression); }
            },
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                if !reached_header_end { temp += identifier.as_str(); }
                else { vec.push(identifier); }
            },
            "qualified_identifier" => {
                let qualified_identifier = handle_qualified_identifier(node, src.clone());
                if !reached_header_end { temp += qualified_identifier.as_str(); }
                else { vec.push(qualified_identifier); }
            },
            "number_literal" => {
                let number_literal = handle_number_literal(node, src.clone());
                if !reached_header_end { temp += number_literal.as_str(); }
                else { vec.push(number_literal); }
            },
            "pointer_expression" => {
                let pointer_expression = handle_pointer_expression(node, src.clone());
                if !reached_header_end { temp += pointer_expression.as_str(); }
                else { vec.push(pointer_expression); }
            },
            "comma_expression" => {
                let comma_expression = handle_comma_expression(node, src.clone());
                if !reached_header_end { temp += comma_expression.as_str(); }
                else { vec.push(comma_expression); }
            },
            "declaration" => {
                let declaration = handle_declaration(node, src.clone());
                if !reached_header_end { temp += format!("{} ", declaration).as_str(); }
                else { vec.push(declaration); }
            },
            "expression_statement" => {
                let expression_statement = handle_expression_statement(node, src.clone());
                vec.push(expression_statement);
            },
            "true" => temp += "true",
            "false" => temp += "false",
            ";" => {
                if !reached_header_end { temp += "; "; }
                else { vec.push(";".to_string()); }
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
            _ => println!("You shouldn't be here (cpp for_statement): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            "qualified_identifier" => {
                let qualified_identifier = handle_qualified_identifier(node, src.clone());
                result += format!(" {}", qualified_identifier).as_str();
            },
            "template_type" => {
                let template_type = handle_template_type(node, src.clone());
                result += format!(" {}", template_type).as_str();
            },
            "virtual_specifier" => {
                let virtual_specifier = handle_virtual_specifier(node, src.clone());
                result += format!(" {}", virtual_specifier).as_str();
            },
            "base_class_clause" => {
                let base_class_clause = handle_base_class_clause(node, src.clone());
                result += format!(" {}", base_class_clause).as_str();
            },
            "attribute_specifier" | "attribute_declaration" => {
                let attribute = handle_attribute(node, src.clone());
                result += format!(" {}", attribute).as_str();
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
            _ => println!("You shouldn't be here (cpp struct_specifier): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            "qualified_identifier" => {
                let qualified_identifier = handle_qualified_identifier(node, src.clone());
                result += format!(" {}", qualified_identifier).as_str();
            },
            "template_type" => {
                let template_type = handle_template_type(node, src.clone());
                result += format!(" {}", template_type).as_str();
            },
            "typedef" => {
                result += "typedef";
            },
            "primitive_type" => {
                let primitive_type = handle_primitive_type(node, src.clone());
                result += format!(" {}", primitive_type).as_str();
            },
            "type_qualifier" => {
                let type_qualifier = handle_type_qualifier(node, src.clone());
                result += format!(" {}", type_qualifier).as_str();
            },
            "sized_type_specifier" => {
                let sized_type_specifier = handle_sized_type_specifier(node, src.clone());
                result += format!(" {}", sized_type_specifier).as_str();
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
                result += format!(" {}", struct_specifier).as_str();
            },
            "union_specifier" => {
                let union_specifier = handle_union_specifier(node, src.clone());
                result += format!(" {}", union_specifier).as_str();
            },
            "class_specifier" => {
                let class_specifier = handle_class_specifier(node, src.clone());
                result += format!(" {}", class_specifier).as_str();
            },
            "pointer_declarator" => {
                let pointer_declarator = handle_pointer_declarator(node, src.clone());
                result = result.trim_end().to_string();
                result += pointer_declarator.as_str();
            },
            "pointer_type_declarator" => {
                let pointer_type_declarator = handle_pointer_type_declarator(node, src.clone());
                result = result.trim_end().to_string();
                result += pointer_type_declarator.as_str();
            },
            "enum_specifier" => {
                let enum_specifier = handle_enum_specifier(node, src.clone());
                result += format!(" {}", enum_specifier.trim_end()).as_str();
            },
            ";" => result += ";",
            "," => result += ", ",
            _ => println!("You shouldn't be here (cpp type_definition): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
                let missing_semi = node_has_missing_semicolon_cpp(node);
                let field_declaration = handle_field_declaration(node, src.clone());
                if pending_incomplete_decl {
                    utils::merge_incomplete_decl_into_parts(&mut parts, &field_declaration);
                } else {
                    parts.push(utils::add_all_leading_tabs(field_declaration));
                }
                pending_incomplete_decl = missing_semi;
            },
            "declaration" => {
                let missing_semi = node_has_missing_semicolon_cpp(node);
                let declaration = handle_declaration(node, src.clone());
                if pending_incomplete_decl {
                    utils::merge_incomplete_decl_into_parts(&mut parts, &declaration);
                } else {
                    parts.push(utils::add_all_leading_tabs(declaration));
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
            "template_declaration" => {
                pending_incomplete_decl = false;
                let mut template_declaration = handle_template_declaration(node, src.clone());
                template_declaration = utils::add_all_leading_tabs(template_declaration);
                parts.push(template_declaration);
            },
            "friend_declaration" => {
                pending_incomplete_decl = false;
                let friend_declaration = handle_friend_declaration(node, src.clone());
                parts.push(format!("\t{}", friend_declaration));
            },
            "alias_declaration" => {
                pending_incomplete_decl = false;
                let alias_declaration = handle_alias_declaration(node, src.clone());
                parts.push(format!("\t{}", alias_declaration));
            },
            "using_declaration" => {
                pending_incomplete_decl = false;
                let using_declaration = handle_using_declaration(node, src.clone());
                parts.push(format!("\t{}", using_declaration));
            },
            "static_assert_declaration" => {
                pending_incomplete_decl = false;
                let static_assert_declaration = handle_static_assert_declaration(node, src.clone());
                parts.push(format!("\t{}", static_assert_declaration));
            },
            "struct_specifier" => {
                pending_incomplete_decl = false;
                let mut struct_specifier = handle_struct_specifier(node, src.clone());
                struct_specifier = format!("{};", struct_specifier.trim_end());
                struct_specifier = utils::add_all_leading_tabs(struct_specifier);
                parts.push(struct_specifier);
            },
            "union_specifier" => {
                pending_incomplete_decl = false;
                let mut union_specifier = handle_union_specifier(node, src.clone());
                union_specifier = format!("{};", union_specifier.trim_end());
                union_specifier = utils::add_all_leading_tabs(union_specifier);
                parts.push(union_specifier);
            },
            "enum_specifier" => {
                pending_incomplete_decl = false;
                let mut enum_specifier = handle_enum_specifier(node, src.clone());
                enum_specifier = format!("{};", enum_specifier.trim_end());
                enum_specifier = utils::add_all_leading_tabs(enum_specifier);
                parts.push(enum_specifier);
            },
            "class_specifier" => {
                pending_incomplete_decl = false;
                let mut class_specifier = handle_class_specifier(node, src.clone());
                class_specifier = format!("{};", class_specifier.trim_end());
                class_specifier = utils::add_all_leading_tabs(class_specifier);
                parts.push(class_specifier);
            },
            "comment" => {
                let comment = handle_comment(node, src.clone());
                utils::push_comment_to_parts(&mut parts, &comment, node.start_byte(), &src, true);
            },
            "access_specifier" => {
                pending_incomplete_decl = false;
                let access_specifier = handle_access_specifier(node, src.clone());
                if parts.len() > 1 {
                    parts.push(format!("\n{}", access_specifier));
                } else {
                    parts.push(access_specifier);
                }
            },
            "type_definition" => {
                pending_incomplete_decl = false;
                let type_definition = handle_type_definition(node, src.clone());
                parts.push(format!("\t{}", type_definition));
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
            "preproc_function_def" => {
                pending_incomplete_decl = false;
                let preproc_function_def = handle_preproc_function_def(node, src.clone());
                parts.push(preproc_function_def);
            },
            "preproc_call" => {
                pending_incomplete_decl = false;
                let preproc_call = handle_preproc_call(node, src.clone());
                parts.push(preproc_call);
            },
            "function_definition" => {
                pending_incomplete_decl = false;
                let mut function_definition = handle_function_definition(node, src.clone());
                function_definition = utils::add_all_leading_tabs(function_definition);
                parts.push(format!("{}\n", function_definition));
            },
            "{" => {
                pending_incomplete_decl = false;
                parts.push("{".to_string());
            },
            "}" => {
                pending_incomplete_decl = false;
                if let Some(last) = parts.last_mut() {
                    if last.ends_with('\n') {
                        last.pop();
                    }
                }
                parts.push("}".to_string());
            },
            ":" => (),
            ";" => {
                if pending_incomplete_decl && !node.is_missing() {
                    if let Some(last) = parts.last_mut() {
                        last.push(';');
                    }
                    pending_incomplete_decl = false;
                }
            },
            _ => println!("You shouldn't be here (cpp field_declaration_list): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap()),
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
            "qualified_identifier" => {
                let qualified_identifier = handle_qualified_identifier(node, src.clone());
                result += format!("{} ", qualified_identifier).as_str();
            },
            "primitive_type" => {
                let primitive_type = handle_primitive_type(node, src.clone());
                result += format!("{} ", primitive_type).as_str();
            },
            "sized_type_specifier" => {
                let sized_type_specifier = handle_sized_type_specifier(node, src.clone());
                result += format!("{} ", sized_type_specifier).as_str();
            },
            "template_type" => {
                let template_type = handle_template_type(node, src.clone());
                result += format!("{} ", template_type).as_str();
            },
            "type_qualifier" => {
                let type_qualifier = handle_type_qualifier(node, src.clone());
                result += format!("{} ", type_qualifier).as_str();
            },
            "placeholder_type_specifier" => {
                let placeholder_type_specifier = handle_placeholder_type_specifier(node, src.clone());
                result += format!("{} ", placeholder_type_specifier.trim()).as_str();
            },
            "decltype" => {
                let decltype = handle_decltype(node, src.clone());
                result += format!("{} ", decltype).as_str();
            },
            "explicit_function_specifier" => {
                let explicit_function_specifier = handle_explicit_function_specifier(node, src.clone());
                result += format!("{} ", explicit_function_specifier).as_str();
            },
            "array_declarator" => {
                let array_declarator = handle_array_declarator(node, src.clone());
                result += format!("{} ", array_declarator).as_str();
            },
            "reference_declarator" => {
                let reference_declarator = handle_reference_declarator(node, src.clone());
                result += format!("{} ", reference_declarator).as_str();
            },
            "pointer_declarator" => {
                let pointer_declarator = handle_pointer_declarator(node, src.clone());
                result += format!("{} ", pointer_declarator).as_str();
            },
            "parenthesized_declarator" => {
                let parenthesized_declarator = handle_parenthesized_declarator(node, src.clone());
                result += format!("{} ", parenthesized_declarator).as_str();
            },
            "compound_literal_expression" => {
                let compound_literal_expression = handle_compound_literal_expression(node, src.clone());
                result += format!("{} ", compound_literal_expression).as_str();
            },
            "initializer_list" => {
                let initializer_list = handle_initializer_list(node, src.clone());
                result = result.trim_end().to_string();
                if result.ends_with('=') {
                    result += format!(" {} ", initializer_list).as_str();
                } else {
                    result += format!("{} ", initializer_list).as_str();
                }
            },
            "function_declarator" => {
                let function_declarator = handle_function_declarator(node, src.clone());
                result += format!("{} ", function_declarator).as_str();
            },
            "optional_parameter_declaration" => {
                let optional_parameter_declaration = handle_optional_parameter_declaration(node, src.clone());
                result += format!("{} ", optional_parameter_declaration).as_str();
            },
            "default_method_clause" => {
                let default_method_clause = handle_default_method_clause(node, src.clone());
                result = result.trim_end().to_string();
                result += default_method_clause.as_str();
            },
            "delete_method_clause" => {
                let delete_method_clause = handle_delete_method_clause(node, src.clone());
                result = result.trim_end().to_string();
                result += delete_method_clause.as_str();
            },
            "bitfield_clause" => {
                let bitfield_clause = handle_bitfield_clause(node, src.clone());
                result += format!("{} ", bitfield_clause).as_str();
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
            "class_specifier" => {
                let class_specifier = handle_class_specifier(node, src.clone());
                result += format!("{} ", class_specifier).as_str();
            },
            "enum_specifier" => {
                let enum_specifier = handle_enum_specifier(node, src.clone());
                result += format!("{} ", enum_specifier).as_str();
            },
            "number_literal" => {
                let number_literal = handle_number_literal(node, src.clone());
                result += format!("{} ", number_literal).as_str();
            },
            "char_literal" => {
                let char_literal = handle_char_literal(node, src.clone());
                result += format!("{} ", char_literal).as_str();
            },
            "string_literal" => {
                let string_literal = handle_string_literal(node, src.clone());
                result += format!("{} ", string_literal).as_str();
            },
            "pointer_expression" => {
                let pointer_expression = handle_pointer_expression(node, src.clone());
                result += format!("{} ", pointer_expression).as_str();
            },
            "call_expression" => {
                let call_expression = handle_call_expression(node, src.clone());
                result += format!("{} ", call_expression).as_str();
            },
            "cast_expression" => {
                let cast_expression = handle_cast_expression(node, src.clone());
                result += format!("{} ", cast_expression).as_str();
            },
            "binary_expression" => {
                let binary_expression = handle_binary_expression(node, src.clone());
                result += format!("{} ", binary_expression).as_str();
            },
            "unary_expression" => {
                let unary_expression = handle_unary_expression(node, src.clone());
                result += format!("{} ", unary_expression).as_str();
            },
            "conditional_expression" => {
                let conditional_expression = handle_conditional_expression(node, src.clone());
                result += format!("{} ", conditional_expression).as_str();
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(node, src.clone());
                result += format!("{} ", parenthesized_expression).as_str();
            },
            "sizeof_expression" => {
                let sizeof_expression = handle_sizeof_expression(node, src.clone());
                result += format!("{} ", sizeof_expression).as_str();
            },
            "subscript_expression" => {
                let subscript_expression = handle_subscript_expression(node, src.clone());
                result += format!("{} ", subscript_expression).as_str();
            },
            "field_expression" => {
                let field_expression = handle_field_expression(node, src.clone());
                result += format!("{} ", field_expression).as_str();
            },
            "new_expression" => {
                let new_expression = handle_new_expression(node, src.clone());
                result += format!("{} ", new_expression).as_str();
            },
            "lambda_expression" => {
                let lambda_expression = handle_lambda_expression(node, src.clone());
                result += format!("{} ", lambda_expression).as_str();
            },
            "ERROR" => {
                let error = handle_error(node, src.clone());
                result += format!("{} ", error).as_str();
            },
            "virtual" => result += "virtual ",
            "inline" => result += "inline ",
            "constexpr" => result += "constexpr ",
            "true" => result += "true",
            "false" => result += "false",
            "null" => result += handle_null(node, src.clone()).as_str(),
            "this" => result += "this",
            ";" => {
                if !node.is_missing() {
                    result += ";";
                }
            },
            "=" => result += " = ",
            "," => result += ", ",
            _ => println!("You shouldn't be here (cpp field_declaration): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    result = utils::remove_unnecessary_spaces(&result);
    if result.contains("*") { result = utils::remove_pointer_spaces(result); }
    if result.contains("&") { result = utils::remove_reference_spaces(result); }
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
            _ => println!("You shouldn't be here (cpp preproc_def): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            "abstract_parenthesized_declarator" => {
                let abstract_parenthesized_declarator = handle_abstract_parenthesized_declarator(node, src.clone());
                result += abstract_parenthesized_declarator.as_str();
            },
            "abstract_array_declarator" => {
                let abstract_array_declarator = handle_abstract_array_declarator(node, src.clone());
                result += abstract_array_declarator.as_str();
            },
            "abstract_pointer_declarator" => {
                let abstract_pointer_declarator = handle_abstract_pointer_declarator(node, src.clone());
                result += abstract_pointer_declarator.as_str();
            },
            "[" => result += "[",
            "]" => result += "]",
            _ => println!("You shouldn't be here (cpp abstract_array_declarator): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            _ => println!("You shouldn't be here (cpp sized_type_specifier): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            "qualified_identifier" => {
                let qualified_identifier = handle_qualified_identifier(node, src.clone());
                result += format!(" {}", qualified_identifier).as_str();
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
            "class" => result += " class",
            "struct" => result += " struct",
            _ => println!("You shouldn't be here (cpp enum_specifier): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            "qualified_identifier" => {
                let qualified_identifier = handle_qualified_identifier(node, src.clone());
                result += format!(" {}", qualified_identifier).as_str();
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
            _ => println!("You shouldn't be here (cpp union_specifier): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            "preproc_call" => {
                if !temp.is_empty() {
                    parts.push(temp);
                    temp = String::new();
                }
                let preproc_call = handle_preproc_call(node, src.clone());
                parts.push(preproc_call);
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
            _ => println!("You shouldn't be here (cpp enumerator_list): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            "qualified_identifier" => parts.push(handle_qualified_identifier(node, src.clone())),
            _ => parts.push(node.utf8_text(src.as_bytes()).unwrap_or("").to_string()),
        }
    }
    return parts.join(" ");
}

fn handle_storage_class_specifier(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "static" => result += "static ",
            "inline" => result += "inline ",
            "extern" => result += "extern ",
            "register" => result += "register ",
            "thread_local" => result += "thread_local ",
            "mutable" => result += "mutable ",
            "constexpr" => result += "constexpr ",
            _ => println!("You shouldn't be here (cpp storage_class_specifier): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            "preproc_call" => {
                let preproc_call = handle_preproc_call(node, src.clone());
                parts.push(preproc_call);
            },
            "field_declaration" => {
                let field_declaration = handle_field_declaration(node, src.clone());
                parts.push(format!("\t{}", field_declaration));
            },
            "declaration" => {
                let declaration = handle_declaration(node, src.clone());
                parts.push(declaration);
            },
            "type_definition" => {
                let type_definition = handle_type_definition(node, src.clone());
                parts.push(type_definition);
            },
            "alias_declaration" => {
                let alias_declaration = handle_alias_declaration(node, src.clone());
                parts.push(alias_declaration);
            },
            "using_declaration" => {
                let using_declaration = handle_using_declaration(node, src.clone());
                parts.push(using_declaration);
            },
            "template_declaration" => {
                let template_declaration = handle_template_declaration(node, src.clone());
                parts.push(template_declaration);
            },
            "template_instantiation" => {
                let template_instantiation = handle_template_instantiation(node, src.clone());
                parts.push(template_instantiation);
            },
            "linkage_specification" => {
                let linkage_specification = handle_linkage_specification(node, src.clone());
                parts.push(linkage_specification);
            },
            "namespace_definition" => {
                let namespace_definition = handle_namespace_definition(node, src.clone());
                parts.push(namespace_definition);
            },
            "namespace_alias_definition" => {
                let namespace_alias_definition = handle_namespace_alias_definition(node, src.clone());
                parts.push(namespace_alias_definition);
            },
            "static_assert_declaration" => {
                let static_assert_declaration = handle_static_assert_declaration(node, src.clone());
                parts.push(static_assert_declaration);
            },
            "class_specifier" => {
                let class_specifier = handle_class_specifier(node, src.clone());
                parts.push(format!("{};", class_specifier.trim_end()));
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
            "for_range_loop" => {
                let for_range_loop = handle_for_range_loop(node, src.clone());
                parts.push(for_range_loop);
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
            "access_specifier" => {
                let access_specifier = handle_access_specifier(node, src.clone());
                if parts.len() > 1 {
                    parts.push(format!("\n{}", access_specifier));
                } else {
                    parts.push(access_specifier);
                }
            },
            "expression_statement" => {
                let expression_statement = handle_expression_statement(node, src.clone());
                parts.push(expression_statement);
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
            "preproc_function_def" => {
                let preproc_function_def = handle_preproc_function_def(node, src.clone());
                parts.push(preproc_function_def);
            },
            "function_definition" => {
                let function_definition = handle_function_definition(node, src.clone());
                parts.push(function_definition);
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
            "return_statement" => {
                let return_statement = handle_return_statement(node, src.clone());
                parts.push(return_statement);
            },
            "enumerator" => {
                let enumerator = handle_enumerator(node, src.clone());
                parts.push(format!("\t{},", enumerator));
            },
            "ERROR" => {
                let error = handle_error(node, src.clone());
                parts.push(error);
            },
            "," => (),
            ";" => (),
            "}" => parts.push("}".to_string()),
            _ => println!("You shouldn't be here (cpp preproc_ifdef): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            "return_statement" => {
                let return_statement = handle_return_statement(node, src.clone());
                parts.push(return_statement);
            },
            "preproc_if" => {
                let preproc_if = handle_preproc_if(node, src.clone());
                parts.push(preproc_if);
            },
            "preproc_ifdef" => {
                let preproc_ifdef = handle_preproc_ifdef(node, src.clone());
                parts.push(preproc_ifdef);
            },
            "declaration" => {
                let declaration = handle_declaration(node, src.clone());
                parts.push(declaration);
            },
            "field_declaration" => {
                let field_declaration = handle_field_declaration(node, src.clone());
                parts.push(format!("\t{}", field_declaration));
            },
            "type_definition" => {
                let type_definition = handle_type_definition(node, src.clone());
                parts.push(type_definition);
            },
            "alias_declaration" => {
                let alias_declaration = handle_alias_declaration(node, src.clone());
                parts.push(alias_declaration);
            },
            "using_declaration" => {
                let using_declaration = handle_using_declaration(node, src.clone());
                parts.push(using_declaration);
            },
            "template_declaration" => {
                let template_declaration = handle_template_declaration(node, src.clone());
                parts.push(template_declaration);
            },
            "template_instantiation" => {
                let template_instantiation = handle_template_instantiation(node, src.clone());
                parts.push(template_instantiation);
            },
            "linkage_specification" => {
                let linkage_specification = handle_linkage_specification(node, src.clone());
                parts.push(linkage_specification);
            },
            "namespace_definition" => {
                let namespace_definition = handle_namespace_definition(node, src.clone());
                parts.push(namespace_definition);
            },
            "static_assert_declaration" => {
                let static_assert_declaration = handle_static_assert_declaration(node, src.clone());
                parts.push(static_assert_declaration);
            },
            "class_specifier" => {
                let class_specifier = handle_class_specifier(node, src.clone());
                parts.push(format!("{};", class_specifier.trim_end()));
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
            "for_range_loop" => {
                let for_range_loop = handle_for_range_loop(node, src.clone());
                parts.push(for_range_loop);
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
            "access_specifier" => {
                let access_specifier = handle_access_specifier(node, src.clone());
                if parts.len() > 1 {
                    parts.push(format!("\n{}", access_specifier));
                } else {
                    parts.push(access_specifier);
                }
            },
            "function_definition" => {
                let function_definition = handle_function_definition(node, src.clone());
                parts.push(function_definition);
            },
            "preproc_else" => {
                let preproc_else = handle_preproc_else(node, src.clone());
                parts.push(preproc_else);
            },
            "preproc_call" => {
                let preproc_call = handle_preproc_call(node, src.clone());
                parts.push(preproc_call);
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
                if !temp.is_empty() {
                    temp += preproc_defined.as_str();
                    parts.push(temp);
                    temp = String::new();
                } else {
                    parts.push(preproc_defined);
                }
            },
            "preproc_function_def" => {
                let preproc_function_def = handle_preproc_function_def(node, src.clone());
                if !temp.is_empty() {
                    temp += preproc_function_def.as_str();
                    parts.push(temp);
                    temp = String::new();
                } else {
                    parts.push(preproc_function_def);
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
            "enumerator" => {
                let enumerator = handle_enumerator(node, src.clone());
                parts.push(format!("\t{},", enumerator));
            },
            "ERROR" => {
                let error = handle_error(node, src.clone());
                parts.push(error);
            },
            "," => (),
            "\n" => (),
            ";" => (),
            _ => println!("You shouldn't be here (cpp preproc_if): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            _ => println!("You shouldn't be here (cpp preproc_function_def): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            "field_declaration" => {
                let field_declaration = handle_field_declaration(node, src.clone());
                parts.push(format!("\t{}", field_declaration));
            },
            "type_definition" => {
                let type_definition = handle_type_definition(node, src.clone());
                parts.push(type_definition);
            },
            "alias_declaration" => {
                let alias_declaration = handle_alias_declaration(node, src.clone());
                parts.push(alias_declaration);
            },
            "using_declaration" => {
                let using_declaration = handle_using_declaration(node, src.clone());
                parts.push(using_declaration);
            },
            "expression_statement" => {
                let expression_statement = handle_expression_statement(node, src.clone());
                parts.push(expression_statement);
            },
            "template_declaration" => {
                let template_declaration = handle_template_declaration(node, src.clone());
                parts.push(template_declaration);
            },
            "template_instantiation" => {
                let template_instantiation = handle_template_instantiation(node, src.clone());
                parts.push(template_instantiation);
            },
            "linkage_specification" => {
                let linkage_specification = handle_linkage_specification(node, src.clone());
                parts.push(linkage_specification);
            },
            "namespace_definition" => {
                let namespace_definition = handle_namespace_definition(node, src.clone());
                parts.push(namespace_definition);
            },
            "static_assert_declaration" => {
                let static_assert_declaration = handle_static_assert_declaration(node, src.clone());
                parts.push(static_assert_declaration);
            },
            "class_specifier" => {
                let class_specifier = handle_class_specifier(node, src.clone());
                parts.push(format!("{};", class_specifier.trim_end()));
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
            "for_range_loop" => {
                let for_range_loop = handle_for_range_loop(node, src.clone());
                parts.push(for_range_loop);
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
            "access_specifier" => {
                let access_specifier = handle_access_specifier(node, src.clone());
                if parts.len() > 1 {
                    parts.push(format!("\n{}", access_specifier));
                } else {
                    parts.push(access_specifier);
                }
            },
            "preproc_if" => {
                let preproc_if = handle_preproc_if(node, src.clone());
                parts.push(preproc_if);
            },
            "preproc_call" => {
                let preproc_call = handle_preproc_call(node, src.clone());
                parts.push(preproc_call);
            },
            "binary_expression" => {
                let binary_expression = handle_binary_expression(node, src.clone());
                parts.push(binary_expression);
            },
            "preproc_def" => {
                let preproc_def = handle_preproc_def(node, src.clone());
                parts.push(preproc_def);
            },
            "preproc_ifdef" => {
                let preproc_ifdef = handle_preproc_ifdef(node, src.clone());
                parts.push(preproc_ifdef);
            },
            "preproc_elif" => {
                let preproc_elif = handle_preproc_elif(node, src.clone());
                parts.push(preproc_elif);
            },
            "preproc_include" => {
                let preproc_include = handle_preproc_include(node, src.clone());
                parts.push(preproc_include);
            },
            "comment" => {
                let comment = handle_comment(node, src.clone());
                utils::push_comment_to_parts(&mut parts, &comment, node.start_byte(), &src, false);
            },
            "function_definition" => {
                let function_definition = handle_function_definition(node, src.clone());
                parts.push(function_definition);
            },
            "preproc_defined" => {
                let preproc_defined = handle_preproc_defined(node, src.clone());
                parts.push(preproc_defined);
            },
            "preproc_function_def" => {
                let preproc_function_def = handle_preproc_function_def(node, src.clone());
                parts.push(preproc_function_def);
            },
            "enumerator" => {
                let enumerator = handle_enumerator(node, src.clone());
                parts.push(format!("\t{},", enumerator));
            },
            "ERROR" => {
                let error = handle_error(node, src.clone());
                parts.push(error);
            },
            "," => (),
            ";" => (),
            _ => println!("You shouldn't be here (cpp preproc_else): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            _ => println!("You shouldn't be here (cpp variadic_parameter): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            _ => println!("You shouldn't be here (cpp preproc_params): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            _ => println!("You shouldn't be here (cpp preproc_call): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            _ => println!("You shouldn't be here (cpp do_statement): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_inner_field_expression(root: Node, src: String) -> String {
    return handle_field_expression(root, src);
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
            "template_type" => {
                let template_type = handle_template_type(node, src.clone());
                temp += format!("{} ", template_type).as_str();
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
            "structured_binding_declarator" => {
                let structured_binding_declarator = handle_structured_binding_declarator(node, src.clone());
                temp += format!("{} ", structured_binding_declarator).as_str();
            },
            "sized_type_specifier" => {
                let sized_type_specifier = handle_sized_type_specifier(node, src.clone());
                temp += format!("{} ", sized_type_specifier).as_str();
            },
            "pointer_expression" => {
                let pointer_expression = handle_pointer_expression(node, src.clone());
                temp += format!("{} ", pointer_expression).as_str();
            },
            "subscript_expression" => {
                let subscript_expression = handle_subscript_expression(node, src.clone());
                temp += format!("{} ", subscript_expression).as_str();
            },
            "unary_expression" => {
                let unary_expression = handle_unary_expression(node, src.clone());
                temp += format!("{} ", unary_expression).as_str();
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(node, src.clone());
                temp += format!("{} ", parenthesized_expression).as_str();
            },
            "return_statement" => {
                let return_statement = handle_return_statement(node, src.clone());
                vec.push(return_statement);
            },
            "break_statement" => vec.push("break;".to_string()),
            "continue_statement" => vec.push("continue;".to_string()),
            "while_statement" => {
                let while_statement = handle_while_statement(node, src.clone());
                vec.push(while_statement);
            },
            "for_statement" => {
                let for_statement = handle_for_statement(node, src.clone());
                vec.push(for_statement);
            },
            "for_range_loop" => {
                let for_range_loop = handle_for_range_loop(node, src.clone());
                vec.push(for_range_loop);
            },
            "placeholder_type_specifier" => temp += "auto ",
            ":" => temp += ": ",
            ";" => temp += "; ",
            "(" => temp += "(",
            ")" => {
                reached_compound = true;
                temp = temp.trim_end().to_string();
                temp += ")";
                temp = utils::remove_pointer_spaces(temp);
                temp = utils::remove_reference_spaces(temp);
                vec.push(temp);
                temp = "".to_string();
            },
            "for" => temp += "for ",
            _ => println!("You shouldn't be here (cpp for_range_loop): {} : {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap()),
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
            "operator_name" => {
                let operator_name = handle_operator_name(node, src.clone());
                result += operator_name.as_str();
            },
            "destructor_name" => {
                let destructor_name = handle_destructor_name(node, src.clone());
                result += destructor_name.as_str();
            },
            "dependent_name" => {
                result += node.utf8_text(src.as_bytes()).unwrap_or("");
            },
            "ERROR" => {
                let error = handle_error(node, src.clone());
                result += error.as_str();
            },
            "template" => result += "template ",
            "::" => result += "::",
            _ => println!("You shouldn't be here (cpp qualified_identifier): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            "qualified_identifier" => {
                let qualified_identifier = handle_qualified_identifier(node, src.clone());
                result += qualified_identifier.as_str();
            },
            "template_argument_list" => {
                let template_argument_list = handle_template_argument_list(node, src.clone());
                result += template_argument_list.as_str();
            },
            _ => println!("You shouldn't be here (cpp template_type): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            "operator_name" => {
                let operator_name = handle_operator_name(node, src.clone());
                result += operator_name.as_str();
            },
            "template_argument_list" => {
                let template_argument_list = handle_template_argument_list(node, src.clone());
                result += template_argument_list.as_str();
            },
            _ => println!("You shouldn't be here (cpp template_function): {} : {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            "template_function" => {
                let template_function = handle_template_function(node, src.clone());
                result += template_function.as_str();
            },
            "qualified_identifier" => {
                let qualified_identifier = handle_qualified_identifier(node, src.clone());
                result += qualified_identifier.as_str();
            },
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
            "unary_expression" => {
                let unary_expression = handle_unary_expression(node, src.clone());
                result += unary_expression.as_str();
            },
            "field_expression" => {
                let field_expression = handle_field_expression(node, src.clone());
                result += field_expression.as_str();
            },
            "pointer_expression" => {
                let pointer_expression = handle_pointer_expression(node, src.clone());
                result += pointer_expression.as_str();
            },
            "binary_expression" => {
                let binary_expression = handle_binary_expression(node, src.clone());
                result += binary_expression.as_str();
            },
            "parameter_pack_expansion" => {
                let parameter_pack_expansion = handle_parameter_pack_expansion(node, src.clone());
                result += parameter_pack_expansion.as_str();
            },
            "number_literal" => {
                let number_literal = handle_number_literal(node, src.clone());
                result += number_literal.as_str();
            },
            "true" => result += "true",
            "false" => result += "false",
            "null" => result += handle_null(node, src.clone()).as_str(),
            "<" => result += "<",
            ">" => result += ">",
            "," => result += ", ",
            _ => println!("You shouldn't be here (cpp template_argument_list): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            "qualified_identifier" => {
                let qualified_identifier = handle_qualified_identifier(node, src.clone());
                result += qualified_identifier.as_str();
            },
            "field_expression" => {
                let field_expression = handle_field_expression(node, src.clone());
                result += field_expression.as_str();
            },
            "pointer_expression" => {
                let pointer_expression = handle_pointer_expression(node, src.clone());
                result += pointer_expression.as_str();
            },
            "subscript_expression" => {
                let subscript_expression = handle_subscript_expression(node, src.clone());
                result += subscript_expression.as_str();
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(node, src.clone());
                result += parenthesized_expression.as_str();
            },
            "cast_expression" => {
                let cast_expression = handle_cast_expression(node, src.clone());
                result += cast_expression.as_str();
            },
            "assignment_expression" => {
                let assignment_expression = handle_assignment_expression(node, src.clone());
                result += assignment_expression.as_str();
            },
            "conditional_expression" => {
                let conditional_expression = handle_conditional_expression(node, src.clone());
                result += conditional_expression.as_str();
            },
            "update_expression" => {
                let update_expression = handle_update_expression(node, src.clone());
                result += update_expression.as_str();
            },
            "declaration" => {
                let declaration = handle_declaration(node, src.clone());
                result += declaration.as_str();
            },
            "init_declarator" => {
                let init_declarator = handle_init_declarator(node, src.clone());
                result += init_declarator.as_str();
            },
            "number_literal" => {
                let number_literal = handle_number_literal(node, src.clone());
                result += number_literal.as_str();
            },
            "char_literal" => {
                let char_literal = handle_char_literal(node, src.clone());
                result += char_literal.as_str();
            },
            "(" => result += "(",
            ")" => result += ")",
            ";" => result += "; ",
            "true" => result += "true",
            "false" => result += "false",
            "null" => result += handle_null(node, src.clone()).as_str(),
            "this" => result += "this",
            _ => println!("You shouldn't be here (cpp condition_clause): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

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
            _ => println!("You shouldn't be here (cpp raw_string_literal): {}: {}", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            "qualified_identifier" => {
                let qualified_identifier = handle_qualified_identifier(node, src.clone());
                result += qualified_identifier.as_str();
            },
            "field_expression" => {
                let field_expression = handle_field_expression(node, src.clone());
                result += field_expression.as_str();
            },
            "update_expression" => {
                let update_expression = handle_update_expression(node, src.clone());
                result += update_expression.as_str();
            },
            "cast_expression" => {
                let cast_expression = handle_cast_expression(node, src.clone());
                result += cast_expression.as_str();
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(node, src.clone());
                result += parenthesized_expression.as_str();
            },
            "pointer_expression" => {
                let pointer_expression = handle_pointer_expression(node, src.clone());
                result += pointer_expression.as_str();
            },
            "unary_expression" => {
                let unary_expression = handle_unary_expression(node, src.clone());
                result += unary_expression.as_str();
            },
            "conditional_expression" => {
                let conditional_expression = handle_conditional_expression(node, src.clone());
                result += conditional_expression.as_str();
            },
            "binary_expression" => {
                let binary_expression = handle_binary_expression(node, src.clone());
                result += binary_expression.as_str();
            },
            "string_literal" => {
                let string_literal = handle_string_literal(node, src.clone());
                result += string_literal.as_str();
            },
            "char_literal" => {
                let char_literal = handle_char_literal(node, src.clone());
                result += char_literal.as_str();
            },
            "subscript_expression" => {
                let subscript_expression = handle_subscript_expression(node, src.clone());
                result += subscript_expression.as_str();
            },
            "call_expression" => {
                let call_expression = handle_call_expression(node, src.clone());
                result += call_expression.as_str();
            },
            "," => result += ", ",
            "[" => result += "[",
            "]" => result += "]",
            _ => println!("You shouldn't be here (cpp subscript_argument_list): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            "..." => result += "...",
            ":" => result += "\t: ",
            "," => result += ", ",
            _ => println!("You shouldn't be here (cpp field_initializer_list): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            "qualified_identifier" => {
                let qualified_identifier = handle_qualified_identifier(node, src.clone());
                result += qualified_identifier.as_str();
            },
            "template_type" => {
                let template_type = handle_template_type(node, src.clone());
                result += template_type.as_str();
            },
            "argument_list" => {
                let argument_list = handle_argument_list(node, src.clone());
                result += argument_list.as_str();
            },
            "initializer_list" => {
                let initializer_list = handle_initializer_list(node, src.clone());
                result += initializer_list.as_str();
            },
            _ => println!("You shouldn't be here (cpp field_initializer): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            "qualified_identifier" => {
                let qualified_identifier = handle_qualified_identifier(node, src.clone());
                result += qualified_identifier.as_str();
            },
            "function_declarator" => {
                let function_declarator = handle_function_declarator(node, src.clone());
                result += function_declarator.as_str();
            },
            "pointer_declarator" => {
                let pointer_declarator = handle_pointer_declarator(node, src.clone());
                result += pointer_declarator.as_str();
            },
            "array_declarator" => {
                let array_declarator = handle_array_declarator(node, src.clone());
                result += array_declarator.as_str();
            },
            "parenthesized_declarator" => {
                let parenthesized_declarator = handle_parenthesized_declarator(node, src.clone());
                result += parenthesized_declarator.as_str();
            },
            "variadic_declarator" => {
                let variadic_declarator = handle_variadic_declarator(node, src.clone());
                result += variadic_declarator.as_str();
            },
            "structured_binding_declarator" => {
                let structured_binding_declarator = handle_structured_binding_declarator(node, src.clone());
                result += structured_binding_declarator.as_str();
            },
            "ERROR" => {
                let error = handle_error(node, src.clone());
                result += error.as_str();
            },
            "&" => result += "& ",
            "&&" => result += "&& ",
            _ => println!("You shouldn't be here (cpp reference_declarator): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_abstract_reference_declarator(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "&" => result += "&",
            "&&" => result += "&&",
            _ => println!("You shouldn't be here (cpp abstract_reference_declarator): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
                result += format!(" {}", identifier).as_str();
            },
            "qualified_identifier" => {
                let qualified_identifier = handle_qualified_identifier(node, src.clone());
                result += format!(" {}", qualified_identifier).as_str();
            },
            "template_type" => {
                let template_type = handle_template_type(node, src.clone());
                result += format!(" {}", template_type).as_str();
            },
            "virtual_specifier" => {
                let virtual_specifier = handle_virtual_specifier(node, src.clone());
                result += format!(" {}", virtual_specifier).as_str();
            },
            "base_class_clause" => {
                let base_class_clause = handle_base_class_clause(node, src.clone());
                result += format!(" {}", base_class_clause).as_str();
            },
            "attribute_specifier" | "attribute_declaration" => {
                let attribute = handle_attribute(node, src.clone());
                result += format!(" {}", attribute).as_str();
            },
            "field_declaration_list" => {
                let field_declaration_list = handle_field_declaration_list(node, src.clone());
                result = result.trim_end().to_string();
                result += format!("\n{}", field_declaration_list).as_str();
            },
            "class" => result += "class",
            _ => println!("You shouldn't be here (cpp class_specifier): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
                result += format!("{} ", template_parameter_list).as_str();
            },
            "function_definition" => {
                let function_definition = handle_function_definition(node, src.clone());
                result += function_definition.as_str();
            },
            "declaration" => {
                let declaration = handle_declaration(node, src.clone());
                result += declaration.as_str();
            },
            "field_declaration" => {
                let field_declaration = handle_field_declaration(node, src.clone());
                result += field_declaration.as_str();
            },
            "struct_specifier" => {
                let struct_specifier = handle_struct_specifier(node, src.clone());
                result += format!("{};", struct_specifier.trim_end()).as_str();
            },
            "class_specifier" => {
                let class_specifier = handle_class_specifier(node, src.clone());
                result += format!("{};", class_specifier.trim_end()).as_str();
            },
            "alias_declaration" => {
                let alias_declaration = handle_alias_declaration(node, src.clone());
                result += alias_declaration.as_str();
            },
            "type_definition" => {
                let type_definition = handle_type_definition(node, src.clone());
                result += type_definition.as_str();
            },
            "template_declaration" => {
                let template_declaration = handle_template_declaration(node, src.clone());
                result += template_declaration.as_str();
            },
            "friend_declaration" => {
                let friend_declaration = handle_friend_declaration(node, src.clone());
                result += friend_declaration.as_str();
            },
            "comment" => {
                let comment = handle_comment(node, src.clone());
                utils::append_comment_to_result(&mut result, &comment, node.start_byte(), &src, "", "\n", true);
            },
            "template" => result += "template ",
            ";" => (),
            _ => println!("You shouldn't be here (cpp template_declaration): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            "parameter_declaration" => {
                let parameter_declaration = handle_parameter_declaration(node, src.clone());
                result += parameter_declaration.as_str();
            },
            "variadic_parameter_declaration" => {
                let variadic_parameter_declaration = handle_variadic_parameter_declaration(node, src.clone());
                result += variadic_parameter_declaration.as_str();
            },
            "variadic_type_parameter_declaration" => {
                let variadic_type_parameter_declaration = handle_variadic_type_parameter_declaration(node, src.clone());
                result += variadic_type_parameter_declaration.as_str();
            },
            "optional_type_parameter_declaration" => {
                let optional_type_parameter_declaration = handle_optional_type_parameter_declaration(node, src.clone());
                result += optional_type_parameter_declaration.as_str();
            },
            "optional_parameter_declaration" => {
                let optional_parameter_declaration = handle_optional_parameter_declaration(node, src.clone());
                result += optional_parameter_declaration.as_str();
            },
            "<" => result += "<",
            ">" => result += ">",
            "," => result += ", ",
            _ => println!("You shouldn't be here (cpp template_parameter_list): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            },
            "typename" => result += "typename ",
            "class" => result += "class ",
            _ => println!("You shouldn't be here (type_parameter_declaration): {} : {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result.trim_end().to_string();
}

fn handle_access_specifier(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "public" => result += "public:",
            "private" => result += "private:",
            "protected" => result += "protected:",
            _ => println!("You shouldn't be here (cpp access_specifier): {} : {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            _ => println!("You shouldn't be here (cpp try_statement): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            _ => println!("You shouldn't be here (cpp catch_clause): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            _ => println!("You shouldn't be here (cpp lambda_expression): {} : {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            _ => println!("You shouldn't be here (cpp lambda_capture_specifier): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            },
            "type_qualifier" => {
                let type_qualifier = handle_type_qualifier(node, src.clone());
                parts.push(type_qualifier);
            },
            "trailing_return_type" => {
                let trailing_return_type = handle_trailing_return_type(node, src.clone());
                parts.push(trailing_return_type);
            },
            "abstract_parenthesized_declarator" => {
                let abstract_parenthesized_declarator = handle_abstract_parenthesized_declarator(node, src.clone());
                parts.push(abstract_parenthesized_declarator);
            },
            "noexcept" => parts.push("noexcept".to_string()),
            "mutable" => parts.push("mutable".to_string()),
            "constexpr" => parts.push("constexpr".to_string()),
            _ => println!("You shouldn't be here (cpp abstract_function_declarator): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            _ => println!("You shouldn't be here (cpp namespace_alias_definition): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let mut result = parts.join(" ");
    result = utils::remove_unnecessary_spaces(&result);
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
            "nested_namespace_specifier" => {
                let nested = handle_nested_namespace_specifier(node, src.clone());
                result += nested.as_str();
            },
            "::" => result += "::",
            _ => println!("You shouldn't be here (cpp nested_namespace_specifier): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
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
            "preproc_def" => parts.push(handle_preproc_def(node, src.clone())),
            "preproc_include" => parts.push(handle_preproc_include(node, src.clone())),
            "preproc_if" => parts.push(handle_preproc_if(node, src.clone())),
            "preproc_ifdef" => parts.push(handle_preproc_ifdef(node, src.clone())),
            "preproc_elif" => parts.push(handle_preproc_elif(node, src.clone())),
            "preproc_function_def" => parts.push(handle_preproc_function_def(node, src.clone())),
            "preproc_else" => parts.push(handle_preproc_else(node, src.clone())),
            "preproc_call" => parts.push(handle_preproc_call(node, src.clone())),
            "expression_statement" => parts.push(handle_expression_statement(node, src.clone())),
            "declaration" => parts.push(handle_declaration(node, src.clone())),
            "field_declaration" => parts.push(format!("\t{}", handle_field_declaration(node, src.clone()))),
            "function_definition" => parts.push(handle_function_definition(node, src.clone())),
            "type_definition" => parts.push(handle_type_definition(node, src.clone())),
            "return_statement" => parts.push(handle_return_statement(node, src.clone())),
            "if_statement" => parts.push(handle_if_statement(node, src.clone())),
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
            _ => println!("You shouldn't be here (cpp preproc_elif): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return parts.join("\n");
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
            _ => println!("You shouldn't be here (cpp preproc_defined): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    result = result.replace("( ", "(");
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
            _ => println!("You shouldn't be here (cpp alias_declaration): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let mut result = parts.join(" ");
    result = utils::remove_unnecessary_spaces(&result);
    return result;
}

fn handle_new_expression(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "::" => result += "::",
            "new" => result += "new ",
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                result += identifier.as_str();
            },
            "qualified_identifier" => {
                let qualified_identifier = handle_qualified_identifier(node, src.clone());
                result += qualified_identifier.as_str();
            },
            "template_type" => {
                let template_type = handle_template_type(node, src.clone());
                result += template_type.as_str();
            },
            "primitive_type" => {
                let primitive_type = handle_primitive_type(node, src.clone());
                result += primitive_type.as_str();
            },
            "placement_argument_list" | "argument_list" => {
                let argument_list = handle_argument_list(node, src.clone());
                result += argument_list.as_str();
            },
            "new_declarator" => {
                result += node.utf8_text(src.as_bytes()).unwrap_or("");
            },
            "initializer_list" => {
                let initializer_list = handle_initializer_list(node, src.clone());
                result += initializer_list.as_str();
            },
            _ => println!("You shouldn't be here (cpp new_expression): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_namespace_definition(root: Node, src: String) -> String {
    let mut parts = Vec::<String>::new();
    let mut temp = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "inline" => temp += "inline ",
            "namespace" => temp += "namespace ",
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                temp += identifier.as_str();
            },
            "nested_namespace_specifier" => {
                let nested = handle_nested_namespace_specifier(node, src.clone());
                temp += nested.as_str();
            },
            "::" => temp += "::",
            "declaration_list" => {
                if !temp.is_empty() {
                    parts.push(temp.trim_end().to_string());
                    temp = String::new();
                }
                let declaration_list = handle_declaration_list(node, src.clone());
                parts.push(declaration_list);
            },
            _ => println!("You shouldn't be here (cpp namespace_definition): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let result = parts.join("\n");
    return result;
}

fn handle_declaration_list(root: Node, src: String) -> String {
    let mut parts = Vec::<String>::new();
    let mut pending_incomplete_decl = false;
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
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
            "comment" => {
                let comment = handle_comment(node, src.clone());
                utils::push_comment_to_parts(&mut parts, &comment, node.start_byte(), &src, false);
            },
            "declaration" => {
                let missing_semi = node_has_missing_semicolon_cpp(node);
                let declaration = handle_declaration(node, src.clone());
                if pending_incomplete_decl {
                    utils::merge_incomplete_decl_into_parts(&mut parts, &declaration);
                } else {
                    parts.push(declaration);
                }
                pending_incomplete_decl = missing_semi;
            },
            "function_definition" => {
                pending_incomplete_decl = false;
                let function_definition = handle_function_definition(node, src.clone());
                parts.push(format!("{}\n", function_definition));
            },
            "struct_specifier" => {
                let struct_specifier = handle_struct_specifier(node, src.clone());
                parts.push(format!("{};\n", struct_specifier.trim_end()));
            },
            "union_specifier" => {
                let union_specifier = handle_union_specifier(node, src.clone());
                parts.push(format!("{};\n", union_specifier.trim_end()));
            },
            "enum_specifier" => {
                let enum_specifier = handle_enum_specifier(node, src.clone());
                parts.push(format!("{};\n", enum_specifier.trim_end()));
            },
            "class_specifier" => {
                let class_specifier = handle_class_specifier(node, src.clone());
                parts.push(format!("{};\n", class_specifier.trim_end()));
            },
            "template_declaration" => {
                let template_declaration = handle_template_declaration(node, src.clone());
                parts.push(format!("{}\n", template_declaration));
            },
            "template_instantiation" => {
                let template_instantiation = handle_template_instantiation(node, src.clone());
                parts.push(template_instantiation);
            },
            "type_definition" => {
                let type_definition = handle_type_definition(node, src.clone());
                parts.push(type_definition);
            },
            "alias_declaration" => {
                let alias_declaration = handle_alias_declaration(node, src.clone());
                parts.push(alias_declaration);
            },
            "using_declaration" => {
                let using_declaration = handle_using_declaration(node, src.clone());
                parts.push(using_declaration);
            },
            "namespace_definition" => {
                let namespace_definition = handle_namespace_definition(node, src.clone());
                parts.push(namespace_definition);
            },
            "namespace_alias_definition" => {
                let namespace_alias_definition = handle_namespace_alias_definition(node, src.clone());
                parts.push(namespace_alias_definition);
            },
            "linkage_specification" => {
                let linkage_specification = handle_linkage_specification(node, src.clone());
                parts.push(linkage_specification);
            },
            "static_assert_declaration" => {
                let static_assert_declaration = handle_static_assert_declaration(node, src.clone());
                parts.push(static_assert_declaration);
            },
            "expression_statement" => {
                let expression_statement = handle_expression_statement(node, src.clone());
                parts.push(expression_statement);
            },
            "preproc_include" => parts.push(handle_preproc_include(node, src.clone())),
            "preproc_def" => parts.push(handle_preproc_def(node, src.clone())),
            "preproc_function_def" => parts.push(handle_preproc_function_def(node, src.clone())),
            "preproc_call" => parts.push(handle_preproc_call(node, src.clone())),
            "preproc_ifdef" => {
                let preproc_ifdef = handle_preproc_ifdef(node, src.clone());
                parts.push(preproc_ifdef);
            },
            "preproc_if" => {
                let preproc_if = handle_preproc_if(node, src.clone());
                parts.push(preproc_if);
            },
            "preproc_elif" => {
                let preproc_elif = handle_preproc_elif(node, src.clone());
                parts.push(preproc_elif);
            },
            _ => println!("You shouldn't be here (cpp declaration_list): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            },
            _ => println!("You shouldn't be here (cpp structured_binding_declarator): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_initializer_pair(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "field_designator" => result += handle_field_designator(node, src.clone()).as_str(),
            "subscript_designator" => result += handle_subscript_designator(node, src.clone()).as_str(),
            "identifier" => result += handle_identifier(node, src.clone()).as_str(),
            "qualified_identifier" => result += handle_qualified_identifier(node, src.clone()).as_str(),
            "number_literal" => result += handle_number_literal(node, src.clone()).as_str(),
            "char_literal" => result += handle_char_literal(node, src.clone()).as_str(),
            "string_literal" => result += handle_string_literal(node, src.clone()).as_str(),
            "concatenated_string" => result += handle_concatenated_string(node, src.clone()).as_str(),
            "binary_expression" => result += handle_binary_expression(node, src.clone()).as_str(),
            "unary_expression" => result += handle_unary_expression(node, src.clone()).as_str(),
            "cast_expression" => result += handle_cast_expression(node, src.clone()).as_str(),
            "call_expression" => result += handle_call_expression(node, src.clone()).as_str(),
            "field_expression" => result += handle_field_expression(node, src.clone()).as_str(),
            "subscript_expression" => result += handle_subscript_expression(node, src.clone()).as_str(),
            "pointer_expression" => result += handle_pointer_expression(node, src.clone()).as_str(),
            "conditional_expression" => result += handle_conditional_expression(node, src.clone()).as_str(),
            "parenthesized_expression" => result += handle_parenthesized_expression(node, src.clone()).as_str(),
            "sizeof_expression" => result += handle_sizeof_expression(node, src.clone()).as_str(),
            "initializer_list" => result += handle_initializer_list(node, src.clone()).as_str(),
            "compound_literal_expression" => result += handle_compound_literal_expression(node, src.clone()).as_str(),
            "null" => result += handle_null(node, src.clone()).as_str(),
            "true" => result += "true",
            "false" => result += "false",
            "=" => result += " = ",
            ":" => result += ": ",
            _ => println!("You shouldn't be here (cpp initializer_pair): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            "ERROR" => {
                let error = handle_error(node, src.clone());
                result += error.as_str();
            },
            "," => result += ", ",
            _ => println!("You shouldn't be here (cpp comma_expression): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_static_assert_declaration(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "static_assert" => result += "static_assert",
            "binary_expression" => {
                let binary_expression = handle_binary_expression(node, src.clone());
                result += binary_expression.as_str();
            },
            "unary_expression" => {
                let unary_expression = handle_unary_expression(node, src.clone());
                result += unary_expression.as_str();
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(node, src.clone());
                result += parenthesized_expression.as_str();
            },
            "call_expression" => {
                let call_expression = handle_call_expression(node, src.clone());
                result += call_expression.as_str();
            },
            "template_function" => {
                let template_function = handle_template_function(node, src.clone());
                result += template_function.as_str();
            },
            "qualified_identifier" => {
                let qualified_identifier = handle_qualified_identifier(node, src.clone());
                result += qualified_identifier.as_str();
            },
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                result += identifier.as_str();
            },
            "string_literal" => {
                let string_literal = handle_string_literal(node, src.clone());
                result += string_literal.as_str();
            },
            "concatenated_string" => {
                let concatenated_string = handle_concatenated_string(node, src.clone());
                result += concatenated_string.as_str();
            },
            "true" => result += "true",
            "false" => result += "false",
            "(" => result += "(",
            ")" => result += ")",
            "," => result += ", ",
            ";" => result += ";",
            _ => println!("You shouldn't be here (cpp static_assert_declaration): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_static_assert(root: Node, src: String) -> String {
    let _ = (root, src);
    return "static_assert".to_string();
}

fn handle_placeholder_type_specifier(root: Node, src: String) -> String {
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
            "template_type" => {
                let template_type = handle_template_type(node, src.clone());
                result += format!("{} ", template_type).as_str();
            },
            "decltype" => {
                let decltype = handle_decltype(node, src.clone());
                result += decltype.as_str();
            },
            "auto" => result += "auto ",
            _ => println!("You shouldn't be here (cpp placeholder_type_specifier): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_variadic_parameter_declaration(root: Node, src: String) -> String {
    let mut parts = Vec::<String>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => parts.push(handle_identifier(node, src.clone())),
            "qualified_identifier" => parts.push(handle_qualified_identifier(node, src.clone())),
            "template_type" => parts.push(handle_template_type(node, src.clone())),
            "type_qualifier" => parts.push(handle_type_qualifier(node, src.clone())),
            "primitive_type" => parts.push(handle_primitive_type(node, src.clone())),
            "reference_declarator" => parts.push(handle_reference_declarator(node, src.clone())),
            "pointer_declarator" => parts.push(handle_pointer_declarator(node, src.clone())),
            "variadic_declarator" => parts.push(handle_variadic_declarator(node, src.clone())),
            _ => println!("You shouldn't be here: (cpp handle_variadic_parameter_declaration): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return utils::remove_reference_spaces(parts.join(" "));
}

fn handle_variadic_type_parameter_declaration(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                result += identifier.as_str();
            },
            "reference_declarator" => {
                let reference_declarator = handle_reference_declarator(node, src.clone());
                result += reference_declarator.as_str();
            },
            "typename" => result += "typename",
            "class" => result += "class",
            "..." => result += "... ",
            _ => println!("You shouldn't be here: (cpp handle_variadic_type_parameter_declaration): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result.trim_end().to_string();
}

fn handle_abstract_parenthesized_declarator(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "abstract_reference_declarator" => {
                let abstract_reference_declarator = handle_abstract_reference_declarator(node, src.clone());
                result += abstract_reference_declarator.as_str();
            },
            "abstract_pointer_declarator" => {
                let abstract_pointer_declarator = handle_abstract_pointer_declarator(node, src.clone());
                result += abstract_pointer_declarator.as_str();
            },
            "abstract_function_declarator" => {
                let abstract_function_declarator = handle_abstract_function_declarator(node, src.clone());
                result += abstract_function_declarator.as_str();
            },
            "(" => result += "(",
            ")" => result += ")",
            "," => result += ",",
            ";" => result += ";",
            _ => println!("You shouldn't be here (cpp abstract_parenthesized_declarator): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_variadic_declarator(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                result += identifier.as_str();
            },
            "..." => result += "... ",
            _ => println!("You shouldn't be here (cpp variadic_declarator): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_base_class_clause(root: Node, src: String) -> String {
    let mut parts = Vec::<String>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "access_specifier" => {
                parts.push(node.utf8_text(src.as_bytes()).unwrap_or("").trim().to_string());
            },
            "public" | "private" | "protected" | "virtual" => {
                parts.push(node.grammar_name().to_string());
            },
            "identifier" => {
                parts.push(handle_identifier(node, src.clone()));
            },
            "template_type" => {
                parts.push(handle_template_type(node, src.clone()));
            },
            "qualified_identifier" => {
                parts.push(handle_qualified_identifier(node, src.clone()));
            },
            "decltype" => {
                parts.push(handle_decltype(node, src.clone()));
            },
            "..." => parts.push("...".to_string()),
            ":" => parts.push(":".to_string()),
            "," => parts.push(",".to_string()),
            _ => println!("You shouldn't be here (cpp base_class_clause): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let mut result = parts.join(" ");
    if result.contains(",") {
        result = utils::remove_whitespace_before_commas(&result);
    }
    return result;
}

fn handle_parameter_pack_expansion(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "type_descriptor" => {
                let type_descriptor = handle_type_descriptor(node, src.clone());
                result += type_descriptor.as_str();
            },
            "call_expression" => {
                let call_expression = handle_call_expression(node, src.clone());
                result += call_expression.as_str();
            },
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                result += identifier.as_str();
            },
            "qualified_identifier" => {
                let qualified_identifier = handle_qualified_identifier(node, src.clone());
                result += qualified_identifier.as_str();
            },
            "binary_expression" => {
                let binary_expression = handle_binary_expression(node, src.clone());
                result += binary_expression.as_str();
            },
            "..." => result += "...",
            _ => println!("You shouldn't be here (cpp parameter_pack_expansion): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_optional_type_parameter_declaration(root: Node, src: String) -> String {
    let mut parts = Vec::<String>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => parts.push(handle_identifier(node, src.clone())),
            "qualified_identifier" => parts.push(handle_qualified_identifier(node, src.clone())),
            "template_type" => parts.push(handle_template_type(node, src.clone())),
            "type_descriptor" => parts.push(handle_type_descriptor(node, src.clone())),
            "primitive_type" => parts.push(handle_primitive_type(node, src.clone())),
            "typename" => parts.push("typename".to_string()),
            "class" => parts.push("class".to_string()),
            "=" => parts.push("=".to_string()),
            _ => println!("You shouldn't be here (cpp optional_type_parameter_declaration): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return parts.join(" ");
}

fn handle_decltype(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "sizeof_expression" => {
                let sizeof_expression = handle_sizeof_expression(node, src.clone());
                result += sizeof_expression.as_str();
            },
            "identifier" => result += handle_identifier(node, src.clone()).as_str(),
            "qualified_identifier" => result += handle_qualified_identifier(node, src.clone()).as_str(),
            "call_expression" => result += handle_call_expression(node, src.clone()).as_str(),
            "field_expression" => result += handle_field_expression(node, src.clone()).as_str(),
            "binary_expression" => result += handle_binary_expression(node, src.clone()).as_str(),
            "unary_expression" => result += handle_unary_expression(node, src.clone()).as_str(),
            "decltype" => {
                if node.child_count() == 0 {
                    result += "decltype";
                } else {
                    result += handle_decltype(node, src.clone()).as_str();
                }
            },
            "auto" => result += "auto",
            "(" => result += "(",
            ")" => result += ")",
            _ => println!("You shouldn't be here (cpp decltype): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_destructor_name(root: Node, src: String) -> String {
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
            "destructor_name" => {
                let destructor_name = handle_destructor_name(node, src.clone());
                result += destructor_name.as_str();
            },
            "::" => result += "::",
            "~" => result += "~",
            _ => println!("You shouldn't be here (cpp destructor_name): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_friend_declaration(root: Node, src: String) -> String {
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
            "template_type" => {
                let template_type = handle_template_type(node, src.clone());
                result += template_type.as_str();
            },
            "declaration" => {
                let declaration = handle_declaration(node, src.clone());
                result += declaration.as_str();
            },
            "function_definition" => {
                let function_definition = handle_function_definition(node, src.clone());
                result += function_definition.as_str();
            },
            "friend" => result += "friend ",
            "class" => result += "class ",
            "struct" => result += "struct ",
            "union" => result += "union ",
            "typename" => result += "typename ",
            ";" => result += ";",
            _ => println!("You shouldn't be here (cpp friend_declaration): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_pointer_type_declarator(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                result += format!(" {}", identifier).as_str();
            },
            "qualified_identifier" => {
                let qualified_identifier = handle_qualified_identifier(node, src.clone());
                result += format!(" {}", qualified_identifier).as_str();
            },
            "pointer_type_declarator" => {
                let inner = handle_pointer_type_declarator(node, src.clone());
                result += inner.as_str();
            },
            "function_declarator" => {
                let function_declarator = handle_function_declarator(node, src.clone());
                result += format!(" {}", function_declarator).as_str();
            },
            "parenthesized_declarator" => {
                let parenthesized_declarator = handle_parenthesized_declarator(node, src.clone());
                result += format!(" {}", parenthesized_declarator).as_str();
            },
            "type_qualifier" => {
                let type_qualifier = handle_type_qualifier(node, src.clone());
                result += format!(" {}", type_qualifier).as_str();
            },
            "ERROR" => {
                let error = handle_error(node, src.clone());
                result += format!(" {}", error).as_str();
            },
            "*" => result += "*",
            _ => println!("You shouldn't be here (cpp pointer_type_declarator): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_optional_parameter_declaration(root: Node, src: String) -> String {
    let mut parts = Vec::<String>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => parts.push(handle_identifier(node, src.clone())),
            "qualified_identifier" => parts.push(handle_qualified_identifier(node, src.clone())),
            "primitive_type" => parts.push(handle_primitive_type(node, src.clone())),
            "sized_type_specifier" => parts.push(handle_sized_type_specifier(node, src.clone())),
            "struct_specifier" => parts.push(handle_struct_specifier(node, src.clone())),
            "enum_specifier" => parts.push(handle_enum_specifier(node, src.clone())),
            "number_literal" => parts.push(handle_number_literal(node, src.clone())),
            "char_literal" => parts.push(handle_char_literal(node, src.clone())),
            "type_qualifier" => parts.push(handle_type_qualifier(node, src.clone())),
            "template_type" => parts.push(handle_template_type(node, src.clone())),
            "reference_declarator" => parts.push(handle_reference_declarator(node, src.clone())),
            "abstract_reference_declarator" => parts.push(handle_abstract_reference_declarator(node, src.clone())),
            "abstract_pointer_declarator" => parts.push(handle_abstract_pointer_declarator(node, src.clone())),
            "array_declarator" => parts.push(handle_array_declarator(node, src.clone())),
            "compound_literal_expression" => parts.push(handle_compound_literal_expression(node, src.clone())),
            "initializer_list" => parts.push(handle_initializer_list(node, src.clone())),
            "string_literal" => parts.push(handle_string_literal(node, src.clone())),
            "pointer_declarator" => parts.push(handle_pointer_declarator(node, src.clone())),
            "call_expression" => parts.push(handle_call_expression(node, src.clone())),
            "cast_expression" => parts.push(handle_cast_expression(node, src.clone())),
            "binary_expression" => parts.push(handle_binary_expression(node, src.clone())),
            "unary_expression" => parts.push(handle_unary_expression(node, src.clone())),
            "field_expression" => parts.push(handle_field_expression(node, src.clone())),
            "parenthesized_expression" => parts.push(handle_parenthesized_expression(node, src.clone())),
            "null" => parts.push(handle_null(node, src.clone())),
            "=" => parts.push("=".to_string()),
            "true" => parts.push("true".to_string()),
            "false" => parts.push("false".to_string()),
            _ => println!("You shouldn't be here (cpp optional_parameter_declaration): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let mut result = parts.join(" ");
    result = utils::remove_unnecessary_spaces(&result);
    result = utils::remove_reference_spaces(result);
    return result;
}

fn handle_compound_literal_expression(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "(" => result += "(",
            ")" => result += ")",
            "type_descriptor" => result += handle_type_descriptor(node, src.clone()).as_str(),
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
            "initializer_list" => {
                let initializer_list = handle_initializer_list(node, src.clone());
                result += initializer_list.as_str();
            },
            "template_type" => {
                let template_type = handle_template_type(node, src.clone());
                result += template_type.as_str();
            },
            _ => println!("You shouldn't be here (cpp compound_literal_expression): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_namespace(root: Node, src: String) -> String {
    let _ = (root, src);
    return "namespace".to_string();
}

fn handle_default_method_clause(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "=" => result += " = ",
            ";" => result += ";",
            "default" => result += "default",
            _ => println!("You shouldn't be here (cpp default_method_clause): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_delete_method_clause(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "=" => result += " = ",
            ";" => result += ";",
            "delete" => result += "delete",
            _ => println!("You shouldn't be here (cpp delete_method_clause): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_explicit_function_specifier(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "explicit" => result += "explicit",
            "(" => result += "(",
            ")" => result += ")",
            "binary_expression" => result += handle_binary_expression(node, src.clone()).as_str(),
            "identifier" => result += handle_identifier(node, src.clone()).as_str(),
            "true" => result += "true",
            "false" => result += "false",
            _ => println!("You shouldn't be here (cpp explicit_function_specifier): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    if result.is_empty() {
        result = "explicit".to_string();
    }
    return result;
}

fn handle_virtual_specifier(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "final" => result += "final",
            "override" => result += "override",
            _ => println!("You shouldn't be here (cpp virtual_specifier): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    if result.is_empty() {
        result = root.utf8_text(src.as_bytes()).unwrap_or("").to_string();
    }
    return result;
}

fn handle_trailing_return_type(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "->" => result += "-> ",
            "type_descriptor" => result += handle_type_descriptor(node, src.clone()).as_str(),
            _ => println!("You shouldn't be here (cpp trailing_return_type): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_operator_name(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "operator" => result += "operator",
            "new" => result += " new",
            "delete" => result += " delete",
            "co_await" => result += " co_await",
            _ => result += node.utf8_text(src.as_bytes()).unwrap_or(""),
        }
    }
    if result.is_empty() {
        result = root.utf8_text(src.as_bytes()).unwrap_or("").to_string();
    }
    return result;
}

fn handle_template_method(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "template" => result += "template ",
            "identifier" => result += handle_identifier(node, src.clone()).as_str(),
            "operator_name" => result += handle_operator_name(node, src.clone()).as_str(),
            "template_argument_list" => result += handle_template_argument_list(node, src.clone()).as_str(),
            _ => println!("You shouldn't be here (cpp template_method): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_template_instantiation(root: Node, src: String) -> String {
    let mut parts = Vec::<String>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "extern" => parts.push("extern".to_string()),
            "template" => parts.push("template".to_string()),
            "class" => parts.push("class".to_string()),
            "struct" => parts.push("struct".to_string()),
            "class_specifier" => parts.push(handle_class_specifier(node, src.clone())),
            "struct_specifier" => parts.push(handle_struct_specifier(node, src.clone())),
            "declaration" => parts.push(handle_declaration(node, src.clone())),
            "qualified_identifier" => parts.push(handle_qualified_identifier(node, src.clone())),
            "template_type" => parts.push(handle_template_type(node, src.clone())),
            "identifier" => parts.push(handle_identifier(node, src.clone())),
            "type_qualifier" => parts.push(handle_type_qualifier(node, src.clone())),
            "primitive_type" => parts.push(handle_primitive_type(node, src.clone())),
            "function_declarator" => parts.push(handle_function_declarator(node, src.clone())),
            "pointer_declarator" => parts.push(handle_pointer_declarator(node, src.clone())),
            "reference_declarator" => parts.push(handle_reference_declarator(node, src.clone())),
            ";" => parts.push(";".to_string()),
            _ => println!("You shouldn't be here (cpp template_instantiation): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let result = parts.join(" ");
    return utils::remove_unnecessary_spaces(&result);
}

fn handle_linkage_specification(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "extern" => result += "extern ",
            "string_literal" => result += format!("{} ", handle_string_literal(node, src.clone())).as_str(),
            "declaration_list" => {
                let declaration_list = handle_declaration_list(node, src.clone());
                result += declaration_list.as_str();
            },
            "declaration" => result += handle_declaration(node, src.clone()).as_str(),
            "function_definition" => result += handle_function_definition(node, src.clone()).as_str(),
            "type_definition" => result += handle_type_definition(node, src.clone()).as_str(),
            _ => println!("You shouldn't be here (cpp linkage_specification): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result.trim_end().to_string();
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
            _ => println!("You shouldn't be here (cpp offsetof_expression): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_delete_expression(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "::" => result += "::",
            "delete" => result += "delete",
            "[" => result += "[",
            "]" => result += "]",
            "identifier" => result += format!(" {}", handle_identifier(node, src.clone())).as_str(),
            "qualified_identifier" => result += format!(" {}", handle_qualified_identifier(node, src.clone())).as_str(),
            "field_expression" => result += format!(" {}", handle_field_expression(node, src.clone())).as_str(),
            "subscript_expression" => result += format!(" {}", handle_subscript_expression(node, src.clone())).as_str(),
            "pointer_expression" => result += format!(" {}", handle_pointer_expression(node, src.clone())).as_str(),
            "call_expression" => result += format!(" {}", handle_call_expression(node, src.clone())).as_str(),
            "parenthesized_expression" => result += format!(" {}", handle_parenthesized_expression(node, src.clone())).as_str(),
            "this" => result += " this",
            _ => println!("You shouldn't be here (cpp delete_expression): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_throw_statement(root: Node, src: String) -> String {
    let mut parts = Vec::<String>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "throw" => parts.push("throw".to_string()),
            "call_expression" => parts.push(handle_call_expression(node, src.clone())),
            "identifier" => parts.push(handle_identifier(node, src.clone())),
            "qualified_identifier" => parts.push(handle_qualified_identifier(node, src.clone())),
            "string_literal" => parts.push(handle_string_literal(node, src.clone())),
            "number_literal" => parts.push(handle_number_literal(node, src.clone())),
            "new_expression" => parts.push(handle_new_expression(node, src.clone())),
            "binary_expression" => parts.push(handle_binary_expression(node, src.clone())),
            ";" => parts.push(";".to_string()),
            _ => println!("You shouldn't be here (cpp throw_statement): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let result = parts.join(" ");
    return utils::remove_unnecessary_spaces(&result);
}

fn handle_field_designator(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "." => result += ".",
            "identifier" => result += handle_identifier(node, src.clone()).as_str(),
            _ => println!("You shouldn't be here (cpp field_designator): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
            _ => println!("You shouldn't be here (cpp subscript_designator): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_bitfield_clause(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "call_expression" => {
                let call_expression = handle_call_expression(node, src.clone());
                result += call_expression.as_str();
            },
            "number_literal" => {
                let number_literal = handle_number_literal(node, src.clone());
                result += number_literal.as_str();
            },
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                result += identifier.as_str();
            },
            "qualified_identifier" => {
                let qualified_identifier = handle_qualified_identifier(node, src.clone());
                result += qualified_identifier.as_str();
            },
            "binary_expression" => {
                let binary_expression = handle_binary_expression(node, src.clone());
                result += binary_expression.as_str();
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(node, src.clone());
                result += parenthesized_expression.as_str();
            },
            "=" => result += " = ",
            ";" => result += ";",
            "::" => result += "::",
            ":" => result += ": ",
            "default" => result += "default",
            _ => println!("You shouldn't be here (cpp bitfield_clause): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_operator_cast(root: Node, src: String) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = handle_identifier(node, src.clone());
                result += format!(" {}", identifier).as_str();
            },
            "qualified_identifier" => {
                let qualified_identifier = handle_qualified_identifier(node, src.clone());
                result += format!(" {}", qualified_identifier).as_str();
            },
            "primitive_type" => {
                let primitive_type = handle_primitive_type(node, src.clone());
                result += format!(" {}", primitive_type).as_str();
            },
            "template_type" => {
                let template_type = handle_template_type(node, src.clone());
                result += format!(" {}", template_type).as_str();
            },
            "type_qualifier" => {
                let type_qualifier = handle_type_qualifier(node, src.clone());
                result += format!(" {}", type_qualifier).as_str();
            },
            "abstract_pointer_declarator" => {
                let abstract_pointer_declarator = handle_abstract_pointer_declarator(node, src.clone());
                result += abstract_pointer_declarator.as_str();
            },
            "abstract_reference_declarator" => {
                let abstract_reference_declarator = handle_abstract_reference_declarator(node, src.clone());
                result += abstract_reference_declarator.as_str();
            },
            "abstract_function_declarator" => {
                let abstract_function_declarator = handle_abstract_function_declarator(node, src.clone());
                if abstract_function_declarator.starts_with('(') {
                    result += abstract_function_declarator.as_str();
                } else {
                    result += format!(" {}", abstract_function_declarator).as_str();
                }
            },
            "operator" => result += "operator",
            _ => println!("You shouldn't be here (cpp operator_cast): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_attribute(root: Node, src: String) -> String {
    return root.utf8_text(src.as_bytes()).unwrap_or("").to_string();
}

