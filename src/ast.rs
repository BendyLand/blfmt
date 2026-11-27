use std::borrow::Cow;
use tree_sitter::{Node, Tree};
use crate::utils;

pub trait AstLang {
    const IS_CPP: bool;
    const LANG: &'static str;

    fn parse_wrapped_block(&self, wrapped: &str) -> Tree;
    fn handle_null<'a>(&self, node: Node, src: &'a str) -> &'a str;
    fn dispatch_node<'a>(&self, node: Node, src: &'a str) -> Option<Cow<'a, str>>;
    fn handle_function_declarator_subnode(
        &self,
        _subnode: Node,
        _src: &str,
        _temp: &mut String,
    ) -> bool {
        false
    }
}

#[inline]
fn lang_node<L: AstLang>(ctx: &L, node: Node, src: &str) -> String {
    ctx.dispatch_node(node, src)
        .map(|c| c.into_owned())
        .unwrap_or_default()
}

pub fn node_has_missing_semicolon(root: Node) -> bool {
    for child in root.children(&mut root.walk()) {
        if child.grammar_name() == ";" && child.is_missing() {
            return true;
        }
    }
    false
}

pub fn is_comment_container(grammar_name: &str) -> bool {
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

pub fn extract_comment<'a>(root: Node, src: &'a str) -> &'a str {
    root.utf8_text(src.as_bytes()).unwrap_or("")
}

pub fn handle_attribute<'a>(root: Node, src: &'a str) -> &'a str {
    root.utf8_text(src.as_bytes()).unwrap_or("")
}

pub fn handle_char_literal<'a>(root: Node, src: &'a str) -> &'a str {
    root.utf8_text(src.as_bytes()).unwrap_or("")
}

pub fn handle_number_literal<'a>(root: Node, src: &'a str) -> &'a str {
    root.utf8_text(src.as_bytes()).unwrap_or("")
}

pub fn handle_primitive_type<'a>(root: Node, src: &'a str) -> &'a str {
    root.utf8_text(src.as_bytes()).unwrap_or("")
}

pub fn handle_raw_string_literal<'a>(root: Node, src: &'a str) -> &'a str {
    root.utf8_text(src.as_bytes()).unwrap_or("")
}

pub fn handle_type_identifier<'a>(root: Node, src: &'a str) -> &'a str {
    root.utf8_text(src.as_bytes()).unwrap_or("")
}

pub fn handle_field_identifier<'a>(root: Node, src: &'a str) -> &'a str {
    root.utf8_text(src.as_bytes()).unwrap_or("")
}

pub fn handle_gnu_asm_expression<'a>(root: Node, src: &'a str) -> &'a str {
    root.utf8_text(src.as_bytes()).unwrap_or("")
}

pub fn handle_update_expression(root: Node, src: &str) -> String {
    utils::remove_all_spaces(&root.utf8_text(src.as_bytes()).unwrap_or(""))
}

pub fn handle_preproc_include(root: Node, src: &str) -> String {
    let mut vec: Vec<&str> = Vec::new();
    for node in root.children(&mut root.walk()) {
        vec.push(node.utf8_text(src.as_bytes()).unwrap_or(""));
    }
    vec.join(" ")
}

pub fn handle_preproc_def(root: Node, src: &str) -> String {
    utils::consume_define_node(root, src).0
}

pub fn handle_preproc_function_def(root: Node, src: &str) -> String {
    utils::consume_define_node(root, src).0
}

pub fn is_valid_ref_sibling<L: AstLang>(_ctx: &L, node: Node, src: &str) -> bool {
    let g = node.grammar_name();
    if matches!(
        g,
        "comment"
            | "{"
            | "}"
            | ";"
            | ","
            | "("
            | ")"
            | "["
            | "]"
            | ":"
            | "else"
            | "case"
            | "default"
            | "labeled_statement"
    ) || (L::IS_CPP && g == "access_specifier")
        || g.starts_with("preproc_")
        || g.starts_with('#')
    {
        return false;
    }
    utils::trailing_comment_gap(node.start_byte(), src).is_none()
}

pub fn find_comment_ref_byte<L: AstLang>(ctx: &L, root: Node, src: &str) -> (Option<usize>, bool) {
    let Some(direct_parent) = root.parent() else {
        return (None, true);
    };
    if direct_parent.grammar_name() == "translation_unit" {
        return (None, true);
    }

    if !is_comment_container(direct_parent.grammar_name()) {
        let mut cur = direct_parent;
        while let Some(p) = cur.parent() {
            if is_comment_container(p.grammar_name()) {
                if p.grammar_name() == "translation_unit" {
                    return (Some(cur.start_byte()), false);
                }
                if is_valid_ref_sibling(ctx, cur, src) {
                    return (Some(cur.start_byte()), false);
                }
                break;
            }
            cur = p;
        }
        return (None, false);
    }

    let mut sib_opt = root.next_sibling();
    while let Some(sib) = sib_opt {
        if is_valid_ref_sibling(ctx, sib, src) {
            return (Some(sib.start_byte()), false);
        }
        sib_opt = sib.next_sibling();
    }
    sib_opt = root.prev_sibling();
    while let Some(sib) = sib_opt {
        if is_valid_ref_sibling(ctx, sib, src) {
            return (Some(sib.start_byte()), false);
        }
        sib_opt = sib.prev_sibling();
    }
    (None, false)
}

pub fn try_split_leading_statement_macro<L: AstLang>(ctx: &L, children: &[Node], idx: usize, src: &str) -> Option<(String, usize)> {
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
    let tree = ctx.parse_wrapped_block(&wrapped);
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

    let formatted_compound = handle_compound_statement(ctx, compound, &wrapped);
    let inner = formatted_compound
        .strip_prefix("{\n")
        .unwrap_or(&formatted_compound)
        .strip_suffix('}')
        .unwrap_or(&formatted_compound);
    let first_formatted = first_line.trim();
    Some((format!("\t{}\n{}", first_formatted, inner), end_idx))
}

pub fn handle_compound_statement<L: AstLang>(ctx: &L, root: Node, src: &str) -> String {
    let mut result = String::new();
    let children: Vec<Node> = root.children(&mut root.walk()).collect();
    let mut idx = 0usize;
    while idx < children.len() {
        let node = children[idx];
        if matches!(
            node.grammar_name(),
            "function_definition" | "declaration" | "expression_statement" | "ERROR"
        ) {
            if let Some((split_out, new_end_idx)) = try_split_leading_statement_macro(ctx, &children, idx, src) {
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
                let mut declaration = handle_declaration(ctx, node, src);
                declaration = utils::add_all_leading_tabs(&declaration);
                result += format!("{}\n", declaration).as_str();
            },
            "function_definition" => {
                let function_definition = handle_function_definition(ctx, node, src);
                result += format!("\t{}\n", function_definition).as_str();
            },
            "expression_statement" => {
                let expression_statement = handle_expression_statement(ctx, node, src);
                result += format!("\t{}\n", expression_statement).as_str();
            },
            "compound_statement" => {
                let compound_statement = handle_inner_compound_statement(ctx, node, src);
                result += format!("\t{}\n", compound_statement).as_str();
            },
            "if_statement" => {
                let mut if_statement = handle_if_statement(ctx, node, src);
                if_statement = utils::add_all_leading_tabs(&if_statement);
                result += format!("{}\n", if_statement).as_str();
            },
            "switch_statement" => {
                let mut switch_statement = handle_switch_statement(ctx, node, src);
                switch_statement = utils::remove_blank_lines(switch_statement.split("\n").collect::<Vec<&str>>());
                switch_statement = utils::add_all_leading_tabs(&switch_statement);
                result += format!("{}\n", switch_statement).as_str();
            },
            "case_statement" => {
                let case_statement = handle_case_statement(ctx, node, src);
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
                let goto_statement = handle_goto_statement(ctx, node, src);
                result += format!("\t{}\n", goto_statement).as_str();
            },
            "primitive_type" => {
                let primitive_type = handle_primitive_type(node, src);
                result += format!("\t{}\n", primitive_type).as_str();
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(ctx, node, src);
                result += format!("\t{}\n", parenthesized_expression).as_str();
            },
            "identifier" => {
                let identifier = handle_identifier(ctx, node, src);
                result += format!("\t{}\n", identifier).as_str();
            },
            "return_statement" => {
                let return_statement = handle_return_statement(ctx, node, src);
                result += format!("\t{}\n", return_statement).as_str();
            },
            "comment" => {
                let comment = handle_comment(ctx, node, src);
                utils::append_comment_to_result(&mut result, &comment, node.start_byte(), src, "\t", "\n", true);
            },
            "labeled_statement" => {
                let labeled_statement = handle_labeled_statement(ctx, node, src);
                result += format!("\t{}\n", labeled_statement).as_str();
            },
            "while_statement" => {
                let mut while_statement = handle_while_statement(ctx, node, src);
                while_statement = utils::add_all_leading_tabs(&while_statement);
                result += format!("{}\n", while_statement).as_str();
            },
            "for_statement" => {
                let mut for_statement = handle_for_statement(ctx, node, src);
                for_statement = utils::add_all_leading_tabs(&for_statement);
                result += format!("{}\n", for_statement).as_str();
            },
            "for_range_loop" => {
                let mut for_range_loop = lang_node(ctx, node, src);
                for_range_loop = utils::add_all_leading_tabs(&for_range_loop);
                result += format!("{}\n", for_range_loop).as_str();
            },
            "preproc_def" => {
                let (preproc_def, end_b) = utils::consume_define_node(node, src);
                while idx < children.len() && children[idx].start_byte() < end_b {
                    idx += 1;
                }
                result += format!("{}\n", preproc_def).as_str();
            },
            "type_definition" => {
                let type_definition = handle_type_definition(ctx, node, src);
                result += format!("\t{}\n", type_definition).as_str();
            },
            "struct_specifier" => {
                let struct_specifier = handle_struct_specifier(ctx, node, src);
                result += format!("\t{}\n", struct_specifier).as_str();
            },
            "init_declarator" => {
                let init_declarator = handle_init_declarator(ctx, node, src);
                result += format!("\t{}\n", init_declarator).as_str();
            },
            "ERROR" => {
                let error = handle_error(ctx, node, src);
                result += format!("\t{}\n", error).as_str();
            },
            "parameter_list" => {
                let parameter_list = handle_parameter_list(ctx, node, src);
                result += format!("\t{}\n", parameter_list).as_str();
            },
            "preproc_ifdef" => {
                let preproc_ifdef = handle_preproc_ifdef(ctx, node, src);
                result += format!("\t{}\n", preproc_ifdef).as_str();
            },
            "preproc_if" => {
                let mut preproc_if = handle_preproc_if(ctx, node, src);
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
                let preproc_call = handle_preproc_call(ctx, node, src);
                result += format!("\t{}\n", preproc_call).as_str();
            },
            "do_statement" => {
                let mut do_statement = handle_do_statement(ctx, node, src);
                do_statement = utils::add_all_leading_tabs(&do_statement);
                result += format!("{}\n", do_statement).as_str();
            },
            "try_statement" => {
                let mut try_statement = lang_node(ctx, node, src);
                try_statement = utils::add_all_leading_tabs(&try_statement);
                result += format!("{}\n", try_statement).as_str();
            },
            "alias_declaration" => {
                let mut alias_declaration = lang_node(ctx, node, src);
                alias_declaration = utils::add_all_leading_tabs(&alias_declaration);
                result += format!("{}\n", alias_declaration).as_str();
            },
            "using_declaration" => {
                let mut using_declaration = lang_node(ctx, node, src);
                using_declaration = utils::add_all_leading_tabs(&using_declaration);
                result += format!("{}\n", using_declaration).as_str();
            },
            "static_assert_declaration" => {
                let mut static_assert_declaration = lang_node(ctx, node, src);
                static_assert_declaration = utils::add_all_leading_tabs(&static_assert_declaration);
                result += format!("{}\n", static_assert_declaration).as_str();
            },
            "union_specifier" => {
                let union_specifier = handle_union_specifier(ctx, node, src);
                result += format!("\t{}\n", union_specifier).as_str();
            },
            "enum_specifier" => {
                let enum_specifier = handle_enum_specifier(ctx, node, src);
                result += format!("\t{}\n", enum_specifier).as_str();
            },
            "class_specifier" => {
                let class_specifier = lang_node(ctx, node, src);
                result += format!("\t{}\n", class_specifier).as_str();
            },
            "throw_statement" => {
                let throw_statement = lang_node(ctx, node, src);
                result += format!("\t{}\n", throw_statement).as_str();
            },
            "attributed_statement" => {
                let mut attributed_statement = handle_attributed_statement(ctx, node, src);
                attributed_statement = utils::add_all_leading_tabs(&attributed_statement);
                result += format!("{}\n", attributed_statement).as_str();
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

pub fn handle_declaration<L: AstLang>(ctx: &L, root: Node, src: &str) -> String {
    let mut result: String;
    let mut parts = Vec::<Cow<'_, str>>::new();
    let in_for_loop = root.parent().map(|p| p.grammar_name() == "for_statement" || (L::IS_CPP && p.grammar_name() == "for_range_loop")).unwrap_or(false);
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
        let is_specifier = if L::IS_CPP {
            !(matches!(
                node.grammar_name(),
                "init_declarator"
                    | "pointer_declarator"
                    | "reference_declarator"
                    | "array_declarator"
                    | "function_declarator"
                    | "parenthesized_declarator"
                    | "structured_binding_declarator"
                    | "operator_cast"
            ) || node.kind() == "identifier")
        } else {
            matches!(
                node.grammar_name(),
                "primitive_type"
                    | "type_qualifier"
                    | "storage_class_specifier"
                    | "struct_specifier"
                    | "union_specifier"
                    | "enum_specifier"
                    | "sized_type_specifier"
                    | "macro_type_specifier"
            ) || node.kind() == "type_identifier"
        };

        let piece = match node.grammar_name() {
            "compound_statement" => Some(Cow::from(handle_compound_statement(ctx, node, src))),
            "init_declarator" => Some(Cow::from(handle_init_declarator(ctx, node, src))),
            "primitive_type" => Some(Cow::from(handle_primitive_type(node, src).to_string())),
            "identifier" => Some(Cow::from(handle_identifier(ctx, node, src).to_string())),
            "function_declarator" => Some(Cow::from(handle_function_declarator(ctx, node, src))),
            "parenthesized_declarator" => Some(Cow::from(handle_parenthesized_declarator(ctx, node, src))),
            "reference_declarator" => Some(Cow::from(lang_node(ctx, node, src))),
            "structured_binding_declarator" => Some(Cow::from(lang_node(ctx, node, src))),
            "type_qualifier" => Some(Cow::from(handle_type_qualifier(ctx, node, src).to_string())),
            "ERROR" => Some(Cow::from(handle_error(ctx, node, src))),
            "pointer_declarator" => Some(Cow::from(handle_pointer_declarator(ctx, node, src))),
            "array_declarator" => Some(Cow::from(handle_array_declarator(ctx, node, src))),
            "storage_class_specifier" => Some(Cow::from(handle_storage_class_specifier(ctx, node, src))),
            "explicit_function_specifier" => Some(Cow::from(lang_node(ctx, node, src))),
            "struct_specifier" => Some(Cow::from(handle_struct_specifier(ctx, node, src))),
            "union_specifier" => Some(Cow::from(handle_union_specifier(ctx, node, src))),
            "enum_specifier" => Some(Cow::from(handle_enum_specifier(ctx, node, src))),
            "class_specifier" => Some(Cow::from(lang_node(ctx, node, src))),
            "sized_type_specifier" => Some(Cow::from(handle_sized_type_specifier(ctx, node, src))),
            "macro_type_specifier" if !L::IS_CPP => Some(Cow::from(lang_node(ctx, node, src))),
            "template_type" => Some(Cow::from(lang_node(ctx, node, src))),
            "qualified_identifier" => Some(Cow::from(lang_node(ctx, node, src))),
            "operator_cast" => Some(Cow::from(lang_node(ctx, node, src))),
            "dependent_type" => Some(Cow::from(lang_node(ctx, node, src))),
            "placeholder_type_specifier" => Some(Cow::from(lang_node(ctx, node, src).trim().to_string())),
            "decltype" => Some(Cow::from(lang_node(ctx, node, src))),
            "default_method_clause" => Some(Cow::from(lang_node(ctx, node, src))),
            "delete_method_clause" => Some(Cow::from(lang_node(ctx, node, src))),
            "pure_virtual_clause" => Some(Cow::from(lang_node(ctx, node, src))),
            "attribute_specifier" | "attribute_declaration" => Some(Cow::from(handle_attribute(node, src).to_string())),
            "call_expression" => Some(Cow::from(handle_call_expression(ctx, node, src))),
            "binary_expression" => Some(Cow::from(handle_binary_expression(ctx, node, src))),
            "unary_expression" => Some(Cow::from(handle_unary_expression(ctx, node, src))),
            "cast_expression" => Some(Cow::from(handle_cast_expression(ctx, node, src))),
            "field_expression" => Some(Cow::from(handle_field_expression(ctx, node, src))),
            "subscript_expression" => Some(Cow::from(handle_subscript_expression(ctx, node, src))),
            "pointer_expression" => Some(Cow::from(handle_pointer_expression(ctx, node, src))),
            "parenthesized_expression" => Some(Cow::from(handle_parenthesized_expression(ctx, node, src))),
            "conditional_expression" => Some(Cow::from(handle_conditional_expression(ctx, node, src))),
            "assignment_expression" => Some(Cow::from(handle_assignment_expression(ctx, node, src))),
            "update_expression" => Some(Cow::from(handle_update_expression(node, src))),
            "new_expression" => Some(Cow::from(lang_node(ctx, node, src))),
            "sizeof_expression" => Some(Cow::from(handle_sizeof_expression(ctx, node, src))),
            "initializer_list" => Some(Cow::from(handle_initializer_list(ctx, node, src))),
            "compound_literal_expression" => Some(Cow::from(handle_compound_literal_expression(ctx, node, src))),
            "lambda_expression" => Some(Cow::from(lang_node(ctx, node, src))),
            "template_function" => Some(Cow::from(lang_node(ctx, node, src))),
            "number_literal" => Some(Cow::Borrowed(handle_number_literal(node, src))),
            "char_literal" => Some(Cow::Borrowed(handle_char_literal(node, src))),
            "string_literal" => Some(Cow::from(handle_string_literal(ctx, node, src))),
            "concatenated_string" => Some(Cow::from(handle_concatenated_string(ctx, node, src))),
            "raw_string_literal" => Some(Cow::Borrowed(handle_raw_string_literal(node, src))),
            "null" => Some(Cow::Borrowed(ctx.handle_null(node, src))),
            "true" => Some(Cow::Borrowed("true")),
            "false" => Some(Cow::Borrowed("false")),
            "this" => Some(Cow::Borrowed("this")),
            "=" => Some(Cow::Borrowed("=")),
            "virtual" => Some(Cow::Borrowed("virtual")),
            "inline" => Some(Cow::Borrowed("inline")),
            "constexpr" => Some(Cow::Borrowed("constexpr")),
            "friend" => Some(Cow::Borrowed("friend")),
            "comment" => {
                let comment = handle_comment(ctx, node, src);
                utils::push_expr_comment(&mut parts, &comment, node.start_byte(), node.end_byte(), src);
                if seen_declarator {
                    utils::push_expr_comment(&mut current_decl, &comment, node.start_byte(), node.end_byte(), src);
                }
                None
            },
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
                println!("You shouldn't be here (cpp declaration): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or(""));
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
            if L::IS_CPP && line.contains("&") {
                line = utils::remove_reference_spaces(&line);
            }
            lines.push(line);
        }
        return lines.join("\n");
    }

    result = parts.join(" ");
    if result.contains(",") { result = utils::remove_whitespace_before_commas(&result); }
    result = utils::remove_unnecessary_spaces(&result);
    if L::IS_CPP && result.contains("= \t[") {
        let idx = result.find("= \t[").unwrap();
        result.remove(idx + 2);
    }
    if result.contains("*") { result = utils::remove_pointer_spaces(&result); }
    if L::IS_CPP {
        if result.contains("&") { result = utils::remove_reference_spaces(&result); }
        if result.contains("::") && !result.contains("operator") {
            result = utils::remove_object_constructor_space(&result);
        }
    }
    return result;
}

pub fn handle_function_definition<L: AstLang>(ctx: &L, root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => {
                let compound_statement = handle_compound_statement(ctx, node, src);
                result = result.trim_end().to_string();
                if L::IS_CPP {
                    result = utils::remove_reference_spaces(&result);
                }
                result += format!("\n{}\n", compound_statement).as_str();
            },
            "primitive_type" => {
                let primitive_type = handle_primitive_type(node, src);
                result += format!("{} ", primitive_type).as_str();
            },
            "type_qualifier" => {
                let type_qualifier = handle_type_qualifier(ctx, node, src);
                result += format!("{} ", type_qualifier).as_str();
            },
            "identifier" => {
                let identifier = handle_identifier(ctx, node, src);
                result += format!("{} ", identifier).as_str();
            },
            "qualified_identifier" => {
                let qualified_identifier = lang_node(ctx, node, src);
                result += format!("{} ", qualified_identifier).as_str();
            },
            "ERROR" => {
                let error = handle_error(ctx, node, src);
                result += format!("{} ", error).as_str();
            },
            "function_declarator" => {
                let function_declarator = handle_function_declarator(ctx, node, src);
                result += format!("{}\n", function_declarator).as_str();
            },
            "default_method_clause" => {
                let default_method_clause = lang_node(ctx, node, src);
                result = result.trim_end().to_string();
                result += format!("{}\n", default_method_clause).as_str();
            },
            "delete_method_clause" => {
                let delete_method_clause = lang_node(ctx, node, src);
                result = result.trim_end().to_string();
                result += format!("{}\n", delete_method_clause).as_str();
            },
            "pure_virtual_clause" => {
                let pure_virtual_clause = lang_node(ctx, node, src);
                result = result.trim_end().to_string();
                result += format!("{}\n", pure_virtual_clause).as_str();
            },
            "explicit_function_specifier" => {
                let explicit_function_specifier = lang_node(ctx, node, src);
                result += format!("{} ", explicit_function_specifier).as_str();
            },
            "pointer_declarator" => {
                let pointer_declarator = handle_pointer_declarator(ctx, node, src);
                result = result.trim_end().to_string();
                result += format!("{}\n", pointer_declarator).as_str();
            },
            "parenthesized_declarator" => {
                let parenthesized_declarator = handle_parenthesized_declarator(ctx, node, src);
                result += format!("{}\n", parenthesized_declarator).as_str();
            },
            "struct_specifier" => {
                let struct_specifier = handle_struct_specifier(ctx, node, src);
                result += format!("{} ", struct_specifier.trim_end()).as_str();
            },
            "union_specifier" => {
                let union_specifier = handle_union_specifier(ctx, node, src);
                result += format!("{} ", union_specifier.trim_end()).as_str();
            },
            "enum_specifier" => {
                let enum_specifier = handle_enum_specifier(ctx, node, src);
                result += format!("{} ", enum_specifier.trim_end()).as_str();
            },
            "class_specifier" => {
                let class_specifier = lang_node(ctx, node, src);
                result += format!("{} ", class_specifier.trim_end()).as_str();
            },
            "storage_class_specifier" => {
                let storage_class_specifier = handle_storage_class_specifier(ctx, node, src);
                result += format!("{} ", storage_class_specifier).as_str();
            },
            "sized_type_specifier" => {
                let sized_type_specifier = handle_sized_type_specifier(ctx, node, src);
                result += format!("{} ", sized_type_specifier).as_str();
            },
            "template_type" => {
                let template_type = lang_node(ctx, node, src);
                result += format!("{} ", template_type).as_str();
            },
            "reference_declarator" => {
                let reference_declarator = lang_node(ctx, node, src);
                result = result.trim_end().to_string();
                result += format!("{}\n", reference_declarator).as_str();
            },
            "optional_parameter_declaration" => {
                let optional_parameter_declaration = lang_node(ctx, node, src);
                result += format!("{} ", optional_parameter_declaration).as_str();
            },
            "operator_cast" => {
                let operator_cast = lang_node(ctx, node, src);
                result += format!("{} ", operator_cast).as_str();
            },
            "placeholder_type_specifier" => {
                let placeholder_type_specifier = lang_node(ctx, node, src);
                result += format!("{} ", placeholder_type_specifier.trim()).as_str();
            },
            "field_initializer_list" => {
                let field_initializer_list = lang_node(ctx, node, src);
                result += format!("{} ", field_initializer_list).as_str();
                result = result.trim_end().to_string();
                result += "\n";
            },
            "comment" => {
                let comment = handle_comment(ctx, node, src);
                utils::append_comment_to_result(&mut result, &comment, node.start_byte(), src, "", "\n", true);
            },
            "attribute_specifier" | "attribute_declaration" => {
                let attribute = handle_attribute(node, src);
                result += format!("{} ", attribute).as_str();
            },
            "dependent_type" => {
                let dependent_type = lang_node(ctx, node, src);
                result += format!("{} ", dependent_type).as_str();
            },
            "virtual" => result += "virtual ",
            "inline" => result += "inline ",
            "constexpr" => result += "constexpr ",
            "friend" => result += "friend ",
            _ => println!("You shouldn't be here (cpp function_definition): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    if L::IS_CPP {
        if let Some(brace_idx) = result.find("\n{") {
            let sig = utils::remove_reference_spaces(&result[..brace_idx]);
            result = format!("{}{}", sig, &result[brace_idx..]);
        } else {
            result = utils::remove_reference_spaces(&result);
        }
    }
    result = utils::ensure_space_after_char(&result, '=');
    result = result.trim_end().to_string();
    return result;
}

pub fn handle_expression_statement<L: AstLang>(ctx: &L, root: Node, src: &str) -> String {
    let mut result = "".to_string();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "assignment_expression" => {
                result = handle_assignment_expression(ctx, node, src);
            },
            "update_expression" => {
                result = handle_update_expression(node, src);
            },
            "call_expression" => {
                result = handle_call_expression(ctx, node, src);
            },
            "identifier" => {
                result = handle_identifier(ctx, node, src).to_string();
            },
            "qualified_identifier" => {
                result = lang_node(ctx, node, src);
            },
            "binary_expression" => {
                result = handle_binary_expression(ctx, node, src);
            },
            "unary_expression" => {
                result = handle_unary_expression(ctx, node, src);
            },
            "pointer_expression" => {
                result = handle_pointer_expression(ctx, node, src);
            },
            "cast_expression" => {
                result = handle_cast_expression(ctx, node, src);
            },
            "conditional_expression" => {
                result = handle_conditional_expression(ctx, node, src);
            },
            "parenthesized_expression" => {
                result = handle_parenthesized_expression(ctx, node, src);
            },
            "field_expression" => {
                result = handle_field_expression(ctx, node, src);
            },
            "subscript_expression" => {
                result = handle_subscript_expression(ctx, node, src);
            },
            "compound_literal_expression" => {
                result = handle_compound_literal_expression(ctx, node, src);
            },
            "new_expression" => {
                result = lang_node(ctx, node, src);
            },
            "delete_expression" => {
                result = lang_node(ctx, node, src);
            },
            "throw_statement" => {
                result = lang_node(ctx, node, src);
            },
            "comma_expression" => {
                result = handle_comma_expression(ctx, node, src);
            },
            "ERROR" => {
                if L::IS_CPP {
                    result = handle_error(ctx, node, src);
                } else {
                    println!(
                        "Warning: encountered ERROR node, likely due to preprocessor macro interrupt.\n\t--> {:?}\n",
                        node.utf8_text(src.as_bytes()).unwrap_or("[unreadable]")
                    );
                }
            },
            "gnu_asm_expression" => {
                result = handle_gnu_asm_expression(node, src).to_string();
            },
            "lambda_expression" => {
                result = lang_node(ctx, node, src);
                let head = result.split("\n").collect::<Vec<&str>>()[0].to_string();
                let mut temp = result.split("\n").collect::<Vec<&str>>()[1..].join("\n");
                temp = utils::add_all_leading_tabs(&temp);
                result = format!("{}\n{}", head, temp);
            },
            "number_literal" => {
                if L::IS_CPP { result += handle_number_literal(node, src); }
                else { result = handle_number_literal(node, src).to_string(); }
            },
            "string_literal" => {
                if L::IS_CPP { result += handle_string_literal(ctx, node, src).as_str(); }
                else { result = handle_string_literal(ctx, node, src); }
            },
            "concatenated_string" => result += handle_concatenated_string(ctx, node, src).as_str(),
            "raw_string_literal" => result += handle_raw_string_literal(node, src),
            "char_literal" => {
                if L::IS_CPP { result += handle_char_literal(node, src); }
                else { result = handle_char_literal(node, src).to_string(); }
            },
            "true" => result += "true",
            "false" => result += "false",
            "null" => result += ctx.handle_null(node, src),
            "this" => result += "this",
            "comment" => {
                let comment = handle_comment(ctx, node, src);
                utils::append_expr_comment(&mut result, &comment, node.start_byte(), node.end_byte(), src);
            },
            ";" => {
                if !node.is_missing() {
                    result = result.trim_end().to_string();
                    result += ";";
                }
            },
            _ => println!("You shouldn't be here (cpp expression_statement): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    if result.ends_with("\n\t") {
        result.truncate(result.len() - 2);
    }
    result = utils::remove_unnecessary_spaces(&result);
    return result;
}

pub fn handle_assignment_expression<L: AstLang>(ctx: &L, root: Node, src: &str) -> String {
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
            "this" => parts.push(Cow::Borrowed("this")),
            "true" => parts.push(Cow::Borrowed("true")),
            "false" => parts.push(Cow::Borrowed("false")),
            "null" => parts.push(Cow::from(ctx.handle_null(node, src).to_string())),
            "assignment_expression" => {
                let assignment_expression = handle_inner_assignment_expression(ctx, node, src);
                parts.push(Cow::from(assignment_expression));
            },
            "call_expression" => {
                let call_expression = handle_call_expression(ctx, node, src);
                parts.push(Cow::from(call_expression));
            },
            "cast_expression" => {
                let cast_expression = handle_cast_expression(ctx, node, src);
                parts.push(Cow::from(cast_expression));
            },
            "pointer_expression" => {
                let pointer_expression = handle_pointer_expression(ctx, node, src);
                parts.push(Cow::from(pointer_expression));
            },
            "identifier" => {
                let identifier = handle_identifier(ctx, node, src);
                parts.push(Cow::from(identifier));
            },
            "subscript_expression" => {
                let subscript_expression = handle_subscript_expression(ctx, node, src);
                parts.push(Cow::from(subscript_expression));
            },
            "field_expression" => {
                let field_expression = handle_field_expression(ctx, node, src);
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
                let string_literal = handle_string_literal(ctx, node, src);
                parts.push(Cow::from(string_literal));
            },
            "concatenated_string" => {
                let concatenated_string = handle_concatenated_string(ctx, node, src);
                parts.push(Cow::from(concatenated_string));
            },
            "raw_string_literal" => {
                let raw_string_literal = handle_raw_string_literal(node, src);
                parts.push(Cow::from(raw_string_literal));
            },
            "binary_expression" => {
                let binary_expression = handle_binary_expression(ctx, node, src);
                parts.push(Cow::from(binary_expression));
            },
            "unary_expression" => {
                let unary_expression = handle_unary_expression(ctx, node, src);
                parts.push(Cow::from(unary_expression));
            },
            "update_expression" => {
                let update_expression = handle_update_expression(node, src);
                parts.push(Cow::from(update_expression));
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(ctx, node, src);
                parts.push(Cow::from(parenthesized_expression));
            },
            "sizeof_expression" => {
                let sizeof_expression = handle_sizeof_expression(ctx, node, src);
                parts.push(Cow::from(sizeof_expression));
            },
            "new_expression" => {
                let new_expression = lang_node(ctx, node, src);
                parts.push(Cow::from(new_expression));
            },
            "lambda_expression" => {
                let lambda_expression = lang_node(ctx, node, src);
                parts.push(Cow::from(lambda_expression));
            },
            "compound_literal_expression" => {
                let compound_literal_expression = handle_compound_literal_expression(ctx, node, src);
                parts.push(Cow::from(compound_literal_expression));
            },
            "initializer_list" => {
                let initializer_list = handle_initializer_list(ctx, node, src);
                if !L::IS_CPP || parts.last().map(|s| s == "=" || s == "return" || s == "?" || s == ":").unwrap_or(false) || parts.is_empty() {
                    parts.push(Cow::from(initializer_list));
                } else if let Some(last) = parts.last_mut() {
                    *last = Cow::Owned(format!("{}{}", last, initializer_list));
                }
            },
            "conditional_expression" => {
                let conditional_expression = handle_conditional_expression(ctx, node, src);
                parts.push(Cow::from(conditional_expression));
            },
            "qualified_identifier" => {
                let qualified_identifier = lang_node(ctx, node, src);
                parts.push(Cow::from(qualified_identifier));
            },
            "template_function" => {
                let template_function = lang_node(ctx, node, src);
                parts.push(Cow::from(template_function));
            },
            "comment" => {
                let comment = handle_comment(ctx, node, src);
                utils::push_expr_comment(&mut parts, &comment, node.start_byte(), node.end_byte(), src);
            },
            "ERROR" => {
                if L::IS_CPP {
                    let error = handle_error(ctx, node, src);
                    parts.push(Cow::from(error));
                } else {
                    println!(
                        "Warning: encountered ERROR node, likely due to preprocessor macro interrupt.\n\t--> {:?}\n",
                        node.utf8_text(src.as_bytes()).unwrap_or("[unreadable]")
                    );
                }
            },
            _ => println!("You shouldn't be here (cpp assignment_expression): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let result = parts.join(" ");
    return result;
}

pub fn handle_inner_assignment_expression<L: AstLang>(ctx: &L, root: Node, src: &str) -> String {
    let mut parts = Vec::<Cow<'_, str>>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = handle_identifier(ctx, node, src);
                parts.push(Cow::from(identifier));
            },
            "call_expression" => {
                let call_expression = handle_call_expression(ctx, node, src);
                parts.push(Cow::from(call_expression));
            },
            "assignment_expression" => {
                let assignment_expression = handle_assignment_expression(ctx, node, src);
                parts.push(Cow::from(assignment_expression));
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(ctx, node, src);
                parts.push(Cow::from(parenthesized_expression));
            },
            "pointer_expression" => {
                let pointer_expression = handle_pointer_expression(ctx, node, src);
                parts.push(Cow::from(pointer_expression));
            },
            "conditional_expression" => {
                let conditional_expression = handle_conditional_expression(ctx, node, src);
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
            "string_literal" => {
                let string_literal = handle_string_literal(ctx, node, src);
                parts.push(Cow::from(string_literal));
            },
            "subscript_expression" => {
                let subscript_expression = handle_subscript_expression(ctx, node, src);
                parts.push(Cow::from(subscript_expression));
            },
            "field_expression" => {
                let field_expression = handle_field_expression(ctx, node, src);
                parts.push(Cow::from(field_expression));
            },
            "binary_expression" => {
                let binary_expression = handle_binary_expression(ctx, node, src);
                parts.push(Cow::from(binary_expression));
            },
            "unary_expression" => {
                let unary_expression = handle_unary_expression(ctx, node, src);
                parts.push(Cow::from(unary_expression));
            },
            "cast_expression" => {
                let cast_expression = handle_cast_expression(ctx, node, src);
                parts.push(Cow::from(cast_expression));
            },
            "sizeof_expression" => {
                let sizeof_expression = handle_sizeof_expression(ctx, node, src);
                parts.push(Cow::from(sizeof_expression));
            },
            "qualified_identifier" => {
                let qualified_identifier = lang_node(ctx, node, src);
                parts.push(Cow::from(qualified_identifier));
            },
            "update_expression" => {
                let update_expression = handle_update_expression(node, src);
                parts.push(Cow::from(update_expression));
            },
            "new_expression" => {
                let new_expression = lang_node(ctx, node, src);
                parts.push(Cow::from(new_expression));
            },
            "lambda_expression" => {
                let lambda_expression = lang_node(ctx, node, src);
                parts.push(Cow::from(lambda_expression));
            },
            "compound_literal_expression" => {
                let compound_literal_expression = handle_compound_literal_expression(ctx, node, src);
                parts.push(Cow::from(compound_literal_expression));
            },
            "initializer_list" => {
                let initializer_list = handle_initializer_list(ctx, node, src);
                if !L::IS_CPP || parts.last().map(|s| s == "=" || s == "return" || s == "?" || s == ":").unwrap_or(false) || parts.is_empty() {
                    parts.push(Cow::from(initializer_list));
                } else if let Some(last) = parts.last_mut() {
                    *last = Cow::Owned(format!("{}{}", last, initializer_list));
                }
            },
            "concatenated_string" => {
                let concatenated_string = handle_concatenated_string(ctx, node, src);
                parts.push(Cow::from(concatenated_string));
            },
            "raw_string_literal" => {
                let raw_string_literal = handle_raw_string_literal(node, src);
                parts.push(Cow::from(raw_string_literal));
            },
            "ERROR" => {
                if L::IS_CPP {
                    let error = handle_error(ctx, node, src);
                    parts.push(Cow::from(error));
                }
            },
            "this" => parts.push(Cow::Borrowed("this")),
            "true" => parts.push(Cow::Borrowed("true")),
            "false" => parts.push(Cow::Borrowed("false")),
            "null" => parts.push(Cow::from(ctx.handle_null(node, src).to_string())),
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
            "template_function" => {
                let template_function = lang_node(ctx, node, src);
                parts.push(Cow::from(template_function));
            },
            "comment" => {
                let comment = handle_comment(ctx, node, src);
                utils::push_expr_comment(&mut parts, &comment, node.start_byte(), node.end_byte(), src);
            },
            _ => println!("You shouldn't be here (cpp inner_assignment_expression): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let result = parts.join(" ").to_string();
    return result;
}

pub fn handle_inner_compound_statement<L: AstLang>(ctx: &L, root: Node, src: &str) -> String {
    let mut parts = Vec::<Cow<'_, str>>::new();
    let children: Vec<Node> = root.children(&mut root.walk()).collect();
    let mut idx = 0usize;
    while idx < children.len() {
        let node = children[idx];
        if matches!(
            node.grammar_name(),
            "function_definition" | "declaration" | "expression_statement" | "ERROR"
        ) {
            if let Some((split_out, new_end_idx)) = try_split_leading_statement_macro(ctx, &children, idx, src) {
                parts.push(Cow::from(split_out.trim_end_matches('\n').to_string()));
                idx = new_end_idx + 1;
                continue;
            }
        }
        idx += 1;
        match node.grammar_name() {
            "expression_statement" => {
                let expression_statement = handle_expression_statement(ctx, node, src);
                parts.push(Cow::from(format!("\t{}", expression_statement)));
            },
            "return_statement" => {
                let return_statement = handle_return_statement(ctx, node, src);
                parts.push(Cow::from(format!("\t{}", return_statement)));
            },
            "goto_statement" => {
                let goto_statement = handle_goto_statement(ctx, node, src);
                parts.push(Cow::from(format!("\t{}", goto_statement)));
            },
            "throw_statement" => {
                let throw_statement = lang_node(ctx, node, src);
                parts.push(Cow::from(format!("\t{}", throw_statement)));
            },
            "if_statement" => {
                let mut if_statement = handle_if_statement(ctx, node, src);
                if_statement = utils::add_all_leading_tabs(&if_statement);
                parts.push(Cow::from(if_statement));
            },
            "switch_statement" => {
                let mut switch_statement = handle_switch_statement(ctx, node, src);
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
                let mut case_statement = handle_case_statement(ctx, node, src);
                case_statement = utils::remove_blank_lines(case_statement.split("\n").collect::<Vec<&str>>());
                parts.push(Cow::from(case_statement));
            },
            "declaration" => {
                let mut declaration = handle_declaration(ctx, node, src);
                declaration = utils::add_all_leading_tabs(&declaration);
                parts.push(Cow::from(declaration));
            },
            "comment" => {
                let comment = handle_comment(ctx, node, src);
                utils::push_comment_to_parts(&mut parts, &comment, node.start_byte(), src, true);
            },
            "compound_statement" => {
                let compound_statement = handle_nested_inner_compound_statement(ctx, node, src);
                parts.push(Cow::from(format!("\t{}", compound_statement)));
            },
            "while_statement" => {
                let mut while_statement = handle_while_statement(ctx, node, src);
                while_statement = utils::add_all_leading_tabs(&while_statement);
                parts.push(Cow::from(while_statement));
            },
            "preproc_call" => {
                let preproc_call = handle_preproc_call(ctx, node, src);
                parts.push(Cow::from(format!("\t{}", preproc_call)));
            },
            "preproc_ifdef" => {
                let mut preproc_ifdef = handle_preproc_ifdef(ctx, node, src);
                let has_case = node.children(&mut node.walk()).any(|c| c.grammar_name() == "case_statement");
                if !has_case {
                    preproc_ifdef = utils::add_all_leading_tabs(&preproc_ifdef);
                }
                parts.push(Cow::from(preproc_ifdef));
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
            "try_statement" => {
                let mut try_statement = lang_node(ctx, node, src);
                try_statement = utils::add_all_leading_tabs(&try_statement);
                parts.push(Cow::from(try_statement));
            },
            "alias_declaration" => {
                let mut alias_declaration = lang_node(ctx, node, src);
                alias_declaration = utils::add_all_leading_tabs(&alias_declaration);
                parts.push(Cow::from(alias_declaration));
            },
            "using_declaration" => {
                let mut using_declaration = lang_node(ctx, node, src);
                using_declaration = utils::add_all_leading_tabs(&using_declaration);
                parts.push(Cow::from(using_declaration));
            },
            "static_assert_declaration" => {
                let mut static_assert_declaration = lang_node(ctx, node, src);
                static_assert_declaration = utils::add_all_leading_tabs(&static_assert_declaration);
                parts.push(Cow::from(static_assert_declaration));
            },
            "type_definition" => {
                let type_definition = handle_type_definition(ctx, node, src);
                parts.push(Cow::from(format!("\t{}", type_definition)));
            },
            "struct_specifier" => {
                let struct_specifier = handle_struct_specifier(ctx, node, src);
                parts.push(Cow::from(format!("\t{}", struct_specifier)));
            },
            "union_specifier" => {
                let union_specifier = handle_union_specifier(ctx, node, src);
                parts.push(Cow::from(format!("\t{}", union_specifier)));
            },
            "enum_specifier" => {
                let enum_specifier = handle_enum_specifier(ctx, node, src);
                parts.push(Cow::from(format!("\t{}", enum_specifier)));
            },
            "class_specifier" => {
                let class_specifier = lang_node(ctx, node, src);
                parts.push(Cow::from(format!("\t{}", class_specifier)));
            },
            "do_statement" => {
                let mut do_statement = handle_do_statement(ctx, node, src);
                do_statement = utils::add_all_leading_tabs(&do_statement);
                parts.push(Cow::from(do_statement));
            },
            "for_statement" => {
                let mut for_statement = handle_for_statement(ctx, node, src);
                for_statement = utils::add_all_leading_tabs(&for_statement);
                parts.push(Cow::from(for_statement));
            },
            "for_range_loop" => {
                let mut for_range_loop = lang_node(ctx, node, src);
                for_range_loop = utils::add_all_leading_tabs(&for_range_loop);
                parts.push(Cow::from(for_range_loop));
            },
            "labeled_statement" => {
                let mut labeled_statement = handle_labeled_statement(ctx, node, src);
                if !L::IS_CPP {
                    labeled_statement = utils::add_all_leading_tabs(&labeled_statement);
                }
                parts.push(Cow::from(labeled_statement));
            },
            "preproc_if" => {
                let mut preproc_if = handle_preproc_if(ctx, node, src);
                let has_case = node.children(&mut node.walk()).any(|c| c.grammar_name() == "case_statement");
                if !has_case {
                    preproc_if = utils::add_all_leading_tabs(&preproc_if);
                }
                parts.push(Cow::from(preproc_if));
            },
            "attributed_statement" => {
                let mut attributed_statement = handle_attributed_statement(ctx, node, src);
                attributed_statement = utils::add_all_leading_tabs(&attributed_statement);
                parts.push(Cow::from(attributed_statement));
            },
            "identifier" => {
                let identifier = handle_identifier(ctx, node, src);
                parts.push(Cow::from(format!("\t{}", identifier)));
            },
            "ERROR" => {
                let mut error = handle_error(ctx, node, src);
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

pub fn handle_call_expression<L: AstLang>(ctx: &L, root: Node, src: &str) -> String {
    let mut result = "".to_string();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "call_expression" => {
                let call_expression = handle_call_expression(ctx, node, src);
                result += &call_expression;
            },
            "identifier" => {
                let identifier = handle_identifier(ctx, node, src);
                result += &identifier;
            },
            "primitive_type" => {
                let primitive_type = handle_primitive_type(node, src);
                result += &primitive_type;
            },
            "qualified_identifier" => {
                let qualified_identifier = lang_node(ctx, node, src);
                result += &qualified_identifier;
            },
            "argument_list" => {
                let argument_list = handle_argument_list(ctx, node, src);
                result += &argument_list;
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(ctx, node, src);
                result += &parenthesized_expression;
            },
            "field_expression" => {
                let field_expression = handle_field_expression(ctx, node, src);
                result += &field_expression;
            },
            "subscript_expression" => {
                let subscript_expression = handle_subscript_expression(ctx, node, src);
                result += &subscript_expression;
            },
            "pointer_expression" => {
                let pointer_expression = handle_pointer_expression(ctx, node, src);
                result += &pointer_expression;
            },
            "cast_expression" => {
                let cast_expression = handle_cast_expression(ctx, node, src);
                result += &cast_expression;
            },
            "template_function" => {
                let template_function = lang_node(ctx, node, src);
                result += &template_function;
            },
            "template_type" => {
                let template_type = lang_node(ctx, node, src);
                result += &template_type;
            },
            "lambda_expression" => {
                let lambda_expression = lang_node(ctx, node, src);
                result += &lambda_expression;
            },
            "string_literal" => {
                let string_literal = handle_string_literal(ctx, node, src);
                result += &string_literal;
            },
            "concatenated_string" => {
                let concatenated_string = handle_concatenated_string(ctx, node, src);
                result += &concatenated_string;
            },
            "raw_string_literal" => {
                let raw_string_literal = handle_raw_string_literal(node, src);
                result += raw_string_literal;
            },
            "ERROR" => {
                let error = handle_error(ctx, node, src);
                result += &error;
            },
            ";" => result += ";",
            _ => println!("You shouldn't be here (cpp call_expression): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

pub fn handle_if_statement<L: AstLang>(ctx: &L, root: Node, src: &str) -> String {
    let mut parts = Vec::<Cow<'_, str>>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "if" => parts.push(Cow::Borrowed("if")),
            "constexpr" => parts.push(Cow::Borrowed("constexpr")),
            "compound_statement" => {
                let inner_compound_statement = handle_inner_compound_statement(ctx, node, src);
                parts.push(Cow::from(inner_compound_statement));
            },
            "return_statement" => {
                let return_statement = handle_return_statement(ctx, node, src);
                parts.push(Cow::from(return_statement));
            },
            "expression_statement" => {
                let expression_statement = handle_expression_statement(ctx, node, src);
                parts.push(Cow::from(expression_statement));
            },
            "if_statement" => {
                let if_statement = handle_if_statement(ctx, node, src);
                parts.push(Cow::from(if_statement));
            },
            "for_statement" => {
                let for_statement = handle_for_statement(ctx, node, src);
                parts.push(Cow::from(for_statement));
            },
            "for_range_loop" => {
                let for_range_loop = lang_node(ctx, node, src);
                parts.push(Cow::from(for_range_loop));
            },
            "while_statement" => {
                let while_statement = handle_while_statement(ctx, node, src);
                parts.push(Cow::from(while_statement));
            },
            "do_statement" => {
                let do_statement = handle_do_statement(ctx, node, src);
                parts.push(Cow::from(do_statement));
            },
            "continue_statement" => {
                parts.push(Cow::Borrowed("continue;"));
            },
            "comment" => {
                let comment = handle_comment(ctx, node, src);
                utils::push_expr_comment(&mut parts, &comment, node.start_byte(), node.end_byte(), src);
            },
            "else_clause" => {
                let else_clause = handle_else_clause(ctx, node, src);
                parts.push(Cow::from(else_clause));
            },
            "goto_statement" => {
                let goto_statement = handle_goto_statement(ctx, node, src);
                parts.push(Cow::from(goto_statement));
            },
            "throw_statement" => {
                let throw_statement = lang_node(ctx, node, src);
                parts.push(Cow::from(throw_statement));
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(ctx, node, src);
                parts.push(Cow::from(parenthesized_expression));
            },
            "switch_statement" => {
                let switch_statement = handle_switch_statement(ctx, node, src);
                parts.push(Cow::from(switch_statement));
            },
            "condition_clause" => {
                let condition_clause = lang_node(ctx, node, src);
                parts.push(Cow::from(condition_clause));
            },
            "break_statement" => {
                parts.push(Cow::Borrowed("break;"));
            },
            ";" => parts.push(Cow::Borrowed(";")),
            _ => println!("You shouldn't be here (cpp if_statement): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let result = parts.join(" ");
    return result;
}

pub fn handle_else_clause<L: AstLang>(ctx: &L, root: Node, src: &str) -> String {
    let mut pieces = Vec::<Cow<'_, str>>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => {
                let inner_compound_statement = handle_inner_compound_statement(ctx, node, src);
                pieces.push(Cow::from(inner_compound_statement));
            },
            "else" => {
                pieces.push(Cow::Borrowed("else"));
            },
            "switch_statement" => {
                let switch_statement = handle_switch_statement(ctx, node, src);
                pieces.push(Cow::from(switch_statement));
            },
            "if_statement" => {
                let if_statement = handle_if_statement(ctx, node, src);
                pieces.push(Cow::from(if_statement));
            },
            "return_statement" => {
                let return_statement = handle_return_statement(ctx, node, src);
                pieces.push(Cow::from(return_statement));
            },
            "expression_statement" => {
                let expression_statement = handle_expression_statement(ctx, node, src);
                pieces.push(Cow::from(expression_statement));
            },
            "for_statement" => {
                let for_statement = handle_for_statement(ctx, node, src);
                pieces.push(Cow::from(for_statement));
            },
            "for_range_loop" => {
                let for_range_loop = lang_node(ctx, node, src);
                pieces.push(Cow::from(for_range_loop));
            },
            "while_statement" => {
                let while_statement = handle_while_statement(ctx, node, src);
                pieces.push(Cow::from(while_statement));
            },
            "do_statement" => {
                let do_statement = handle_do_statement(ctx, node, src);
                pieces.push(Cow::from(do_statement));
            },
            "continue_statement" => pieces.push(Cow::Borrowed("continue;")),
            "break_statement" => pieces.push(Cow::Borrowed("break;")),
            "goto_statement" => {
                let goto_statement = handle_goto_statement(ctx, node, src);
                pieces.push(Cow::from(goto_statement));
            },
            "throw_statement" => {
                let throw_statement = lang_node(ctx, node, src);
                pieces.push(Cow::from(throw_statement));
            },
            "comment" => {
                let comment = handle_comment(ctx, node, src);
                utils::push_expr_comment(&mut pieces, &comment, node.start_byte(), node.end_byte(), src);
            },
            ";" => pieces.push(Cow::Borrowed(";")),
            _ => println!("You shouldn't be here (cpp else_clause): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let result = pieces.join(" ");
    return result;
}

pub fn handle_switch_statement<L: AstLang>(ctx: &L, root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "switch" => result += "switch ",
            "compound_statement" => {
                let compound_statement = handle_inner_compound_statement(ctx, node, src);
                result += &compound_statement;
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(ctx, node, src);
                result += format!("{} ", parenthesized_expression).as_str();
            },
            "condition_clause" => {
                let condition_clause = lang_node(ctx, node, src);
                result += format!("{} ", condition_clause).as_str();
            },
            _ => println!("You shouldn't be here (cpp switch_statement): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

pub fn handle_case_statement<L: AstLang>(ctx: &L, root: Node, src: &str) -> String {
    let mut result = String::new();
    let mut opened_inline_brace = false;
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "expression_statement" => {
                let expression_statement = handle_expression_statement(ctx, node, src);
                result += format!("\t{}\n", expression_statement).as_str();
            },
            "compound_statement" => {
                let compound_statement = handle_inner_compound_statement(ctx, node, src);
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
            "declaration" => {
                let mut declaration = handle_declaration(ctx, node, src);
                declaration = utils::add_all_leading_tabs(&declaration);
                result += format!("{}\n", declaration).as_str();
            },
            "return_statement" => {
                let return_statement = handle_return_statement(ctx, node, src);
                result += format!("\t{}\n", return_statement).as_str();
            },
            "if_statement" => {
                let if_statement = handle_if_statement(ctx, node, src);
                result += format!("\t{}\n", if_statement).as_str();
            },
            "goto_statement" => {
                let goto_statement = handle_goto_statement(ctx, node, src);
                result += format!("\t{}\n", goto_statement).as_str();
            },
            "identifier" => {
                let identifier = handle_identifier(ctx, node, src);
                result += &identifier;
            },
            "qualified_identifier" => {
                let qualified_identifier = lang_node(ctx, node, src);
                result += &qualified_identifier;
            },
            "switch_statement" => {
                let switch_statement = handle_switch_statement(ctx, node, src);
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
                let comment = handle_comment(ctx, node, src);
                utils::append_comment_to_result(&mut result, &comment, node.start_byte(), src, "\t", "\n", true);
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
            "while_statement" => {
                let while_statement = handle_while_statement(ctx, node, src);
                result += format!("\t{}\n", while_statement).as_str();
            },
            "for_statement" => {
                let for_statement = handle_for_statement(ctx, node, src);
                result += format!("\t{}\n", for_statement).as_str();
            },
            "for_range_loop" => {
                let for_range_loop = lang_node(ctx, node, src);
                result += format!("\t{}\n", for_range_loop).as_str();
            },
            "do_statement" => {
                let do_statement = handle_do_statement(ctx, node, src);
                result += format!("\t{}\n", do_statement).as_str();
            },
            "labeled_statement" => {
                let labeled_statement = handle_labeled_statement(ctx, node, src);
                result += format!("\t{}\n", labeled_statement).as_str();
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(ctx, node, src);
                result += &parenthesized_expression;
            },
            "binary_expression" => {
                let binary_expression = handle_binary_expression(ctx, node, src);
                result += &binary_expression;
            },
            "unary_expression" => {
                let unary_expression = handle_unary_expression(ctx, node, src);
                result += &unary_expression;
            },
            "cast_expression" => {
                let cast_expression = handle_cast_expression(ctx, node, src);
                result += &cast_expression;
            },
            "attributed_statement" => {
                let mut attributed_statement = handle_attributed_statement(ctx, node, src);
                attributed_statement = utils::add_all_leading_tabs(&attributed_statement);
                result += format!("{}\n", attributed_statement).as_str();
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

pub fn handle_goto_statement<L: AstLang>(ctx: &L, root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "goto" => {
                result += "goto ";
            },
            "identifier" => {
                let identifier = handle_identifier(ctx, node, src);
                result += &identifier;
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

pub fn handle_while_statement<L: AstLang>(ctx: &L, root: Node, src: &str) -> String {
    let mut parts = Vec::<Cow<'_, str>>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => {
                let compound_statement = handle_compound_statement(ctx, node, src);
                parts.push(Cow::from(compound_statement));
            },
            "while" => {
                parts.push(Cow::Borrowed("while"));
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(ctx, node, src);
                parts.push(Cow::from(parenthesized_expression));
            },
            "expression_statement" => {
                let expression_statement = handle_expression_statement(ctx, node, src);
                parts.push(Cow::from(expression_statement));
            },
            "switch_statement" => {
                let switch_statement = handle_switch_statement(ctx, node, src);
                parts.push(Cow::from(switch_statement));
            },
            "condition_clause" => {
                let condition_clause = lang_node(ctx, node, src);
                parts.push(Cow::from(condition_clause));
            },
            "if_statement" => {
                let if_statement = handle_if_statement(ctx, node, src);
                parts.push(Cow::from(if_statement));
            },
            "for_statement" => {
                let for_statement = handle_for_statement(ctx, node, src);
                parts.push(Cow::from(for_statement));
            },
            "for_range_loop" => {
                let for_range_loop = lang_node(ctx, node, src);
                parts.push(Cow::from(for_range_loop));
            },
            "while_statement" => {
                let while_statement = handle_while_statement(ctx, node, src);
                parts.push(Cow::from(while_statement));
            },
            "do_statement" => {
                let do_statement = handle_do_statement(ctx, node, src);
                parts.push(Cow::from(do_statement));
            },
            "return_statement" => {
                let return_statement = handle_return_statement(ctx, node, src);
                parts.push(Cow::from(return_statement));
            },
            "break_statement" => parts.push(Cow::Borrowed("break;")),
            "continue_statement" => parts.push(Cow::Borrowed("continue;")),
            "goto_statement" => {
                let goto_statement = handle_goto_statement(ctx, node, src);
                parts.push(Cow::from(goto_statement));
            },
            ";" => parts.push(Cow::Borrowed(";")),
            _ => println!("You shouldn't be here (cpp while_statement): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let result = parts.join(" ");
    return result;
}

pub fn handle_argument_list<L: AstLang>(ctx: &L, root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "," => result += ", ",
            "(" => result += "(",
            ")" => result += ")",
            "null" => result += ctx.handle_null(node, src),
            "true" => result += "true",
            "false" => result += "false",
            "this" => result += "this",
            "identifier" => {
                let identifier = handle_identifier(ctx, node, src);
                result += &identifier;
            },
            "qualified_identifier" => {
                let qualified_identifier = lang_node(ctx, node, src);
                result += &qualified_identifier;
            },
            "call_expression" => {
                let call_expression = handle_call_expression(ctx, node, src);
                result += &call_expression;
            },
            "cast_expression" => {
                let cast_expression = handle_cast_expression(ctx, node, src);
                result += &cast_expression;
            },
            "field_expression" => {
                let field_expression = handle_field_expression(ctx, node, src);
                result += &field_expression;
            },
            "binary_expression" => {
                let binary_expression = handle_binary_expression(ctx, node, src);
                result += &binary_expression;
            },
            "pointer_expression" => {
                let pointer_expression = handle_pointer_expression(ctx, node, src);
                result += &pointer_expression;
            },
            "subscript_expression" => {
                let subscript_expression = handle_subscript_expression(ctx, node, src);
                result += &subscript_expression;
            },
            "sizeof_expression" => {
                let sizeof_expression = handle_sizeof_expression(ctx, node, src);
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
                let parenthesized_expression = handle_parenthesized_expression(ctx, node, src);
                result += &parenthesized_expression;
            },
            "conditional_expression" => {
                let conditional_expression = handle_conditional_expression(ctx, node, src);
                result += &conditional_expression;
            },
            "unary_expression" => {
                let unary_expression = handle_unary_expression(ctx, node, src);
                result += &unary_expression;
            },
            "assignment_expression" => {
                let assignment_expression = handle_assignment_expression(ctx, node, src);
                result += &assignment_expression;
            },
            "string_literal" => {
                let string_literal = handle_string_literal(ctx, node, src);
                result += &string_literal;
            },
            "concatenated_string" => {
                let concatenated_string = handle_concatenated_string(ctx, node, src);
                result += &concatenated_string;
            },
            "raw_string_literal" => {
                let raw_string_literal = handle_raw_string_literal(node, src);
                result += &raw_string_literal;
            },
            "initializer_list" => {
                let initializer_list = handle_initializer_list(ctx, node, src);
                result += &initializer_list;
            },
            "new_expression" => {
                let new_expression = lang_node(ctx, node, src);
                result += &new_expression;
            },
            "parameter_pack_expansion" => {
                let parameter_pack_expansion = lang_node(ctx, node, src);
                result += &parameter_pack_expansion;
            },
            "lambda_expression" => {
                let mut lambda_expression = lang_node(ctx, node, src);
                lambda_expression = utils::add_all_leading_tabs(&lambda_expression).trim_start().to_string();
                result += &lambda_expression;
            },
            "compound_literal_expression" => {
                let compound_literal_expression = handle_compound_literal_expression(ctx, node, src);
                if L::IS_CPP && result.rsplit('\n').next().unwrap_or("").trim_start().starts_with('#') {
                    if let Some((head, tail)) = compound_literal_expression.split_once("){") {
                        result += head;
                        result += ")\n\t{";
                        result += tail;
                    } else {
                        result += &compound_literal_expression;
                    }
                } else {
                    result += &compound_literal_expression;
                }
            },
            "offsetof_expression" => {
                let offsetof_expression = handle_offsetof_expression(ctx, node, src);
                result += &offsetof_expression;
            },
            "template_function" => {
                let template_function = lang_node(ctx, node, src);
                result += &template_function;
            },
            "type_descriptor" => {
                let type_descriptor = handle_type_descriptor(ctx, node, src);
                result += &type_descriptor;
            },
            "comment" => {
                let comment = handle_comment(ctx, node, src);
                utils::append_expr_comment(&mut result, &comment, node.start_byte(), node.end_byte(), src);
            },
            "ERROR" => {
                let error = handle_error(ctx, node, src);
                result += &error;
            },
            _ => println!("You shouldn't be here (cpp argument_list): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

pub fn handle_return_statement<L: AstLang>(ctx: &L, root: Node, src: &str) -> String {
    let mut result: String;
    let mut parts = Vec::<Cow<'_, str>>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => {
                let compound_statement = handle_compound_statement(ctx, node, src);
                parts.push(Cow::from(compound_statement));
            },
            "compound_literal_expression" => {
                let compound_literal_expression = handle_compound_literal_expression(ctx, node, src);
                parts.push(Cow::from(compound_literal_expression));
            },
            "call_expression" => {
                let call_expression = handle_call_expression(ctx, node, src);
                parts.push(Cow::from(call_expression));
            },
            "binary_expression" => {
                let binary_expression = handle_binary_expression(ctx, node, src);
                parts.push(Cow::from(binary_expression));
            },
            "unary_expression" => {
                let unary_expression = handle_unary_expression(ctx, node, src);
                parts.push(Cow::from(unary_expression));
            },
            "identifier" => {
                let identifier = handle_identifier(ctx, node, src);
                parts.push(Cow::from(identifier));
            },
            "qualified_identifier" => {
                let qualified_identifier = lang_node(ctx, node, src);
                parts.push(Cow::from(qualified_identifier));
            },
            "conditional_expression" => {
                let conditional_expression = handle_conditional_expression(ctx, node, src);
                parts.push(Cow::from(conditional_expression));
            },
            "field_expression" => {
                let field_expression = handle_field_expression(ctx, node, src);
                parts.push(Cow::from(field_expression));
            },
            "cast_expression" => {
                let cast_expression = handle_cast_expression(ctx, node, src);
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
                let string_literal = handle_string_literal(ctx, node, src);
                parts.push(Cow::from(string_literal));
            },
            "subscript_expression" => {
                let subscript_expression = handle_subscript_expression(ctx, node, src);
                parts.push(Cow::from(subscript_expression));
            },
            "pointer_expression" => {
                let pointer_expression = handle_pointer_expression(ctx, node, src);
                parts.push(Cow::from(pointer_expression));
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(ctx, node, src);
                parts.push(Cow::from(parenthesized_expression));
            },
            "initializer_list" => {
                let initializer_list = handle_initializer_list(ctx, node, src);
                if !L::IS_CPP || parts.last().map(|s| s == "=" || s == "return" || s == "?" || s == ":").unwrap_or(false) || parts.is_empty() {
                    parts.push(Cow::from(initializer_list));
                } else if let Some(last) = parts.last_mut() {
                    *last = Cow::Owned(format!("{}{}", last, initializer_list));
                }
            },
            "sizeof_expression" => {
                let sizeof_expression = handle_sizeof_expression(ctx, node, src);
                parts.push(Cow::from(sizeof_expression));
            },
            "update_expression" => {
                let update_expression = handle_update_expression(node, src);
                parts.push(Cow::from(update_expression));
            },
            "assignment_expression" => {
                let assignment_expression = handle_assignment_expression(ctx, node, src);
                parts.push(Cow::from(assignment_expression));
            },
            "new_expression" => {
                let new_expression = lang_node(ctx, node, src);
                parts.push(Cow::from(new_expression));
            },
            "lambda_expression" => {
                let lambda_expression = lang_node(ctx, node, src);
                parts.push(Cow::from(lambda_expression));
            },
            "comma_expression" => {
                let comma_expression = handle_comma_expression(ctx, node, src);
                parts.push(Cow::from(comma_expression));
            },
            "concatenated_string" => {
                let concatenated_string = handle_concatenated_string(ctx, node, src);
                parts.push(Cow::from(concatenated_string));
            },
            "raw_string_literal" => {
                let raw_string_literal = handle_raw_string_literal(node, src);
                parts.push(Cow::from(raw_string_literal));
            },
            "return" => {
                parts.push(Cow::Borrowed("return"));
            },
            "null" => {
                parts.push(Cow::from(ctx.handle_null(node, src).to_string()));
            },
            "this" => {
                parts.push(Cow::Borrowed("this"));
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
                let label = if L::IS_CPP { "cpp return_statement" } else { "c return statement" };
                println!("You shouldn't be here ({}): {}: {}\n", label, node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or(""));
            },
        }
    }
    result = parts.join(" ");
    result = utils::remove_unnecessary_spaces(&result);
    return result;
}

pub fn handle_comment<L: AstLang>(ctx: &L, root: Node, src: &str) -> String {
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
    let (ref_byte, is_top_level) = find_comment_ref_byte(ctx, root, src);
    return utils::format_standalone_comment_indent(&result, root.start_byte(), ref_byte, src, is_top_level);
}

pub fn handle_function_declarator<L: AstLang>(ctx: &L, root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        if L::IS_CPP && node.grammar_name() == "operator_name" {
            result += &lang_node(ctx, node, src);
        } else if node.child_count() == 0 {
            if !result.is_empty() && !result.ends_with(' ') && node.grammar_name() == "identifier" {
                result += " ";
            }
            result += node.utf8_text(src.as_bytes()).unwrap();
        } else {
            if node.grammar_name() == "ERROR" && result.ends_with(')') {
                result += " ";
            }
            let mut temp = String::new();
            for subnode in node.children(&mut node.walk()) {
                match subnode.grammar_name() {
                    "," => temp += ", ",
                    "(" => temp += "(",
                    ")" => temp += ")",
                    "pointer_declarator" => {
                        let pointer_declarator = handle_pointer_declarator(ctx, subnode, src);
                        temp += &pointer_declarator;
                    },
                    "parenthesized_declarator" => {
                        let parenthesized_declarator = handle_parenthesized_declarator(ctx, subnode, src);
                        temp += &parenthesized_declarator;
                    },
                    "parameter_declaration" => {
                        let mut parameter_declaration = handle_parameter_declaration(ctx, subnode, src);
                        parameter_declaration = utils::remove_pointer_spaces(&parameter_declaration);
                        temp += &parameter_declaration;
                    },
                    "identifier" => {
                        let identifier = handle_identifier(ctx, subnode, src);
                        temp += &identifier;
                    },
                    "variadic_parameter" => {
                        let variadic_parameter = handle_variadic_parameter(ctx, subnode, src);
                        temp += &variadic_parameter;
                    },
                    "comment" => {
                        let comment = handle_comment(ctx, subnode, src);
                        temp += format!("{} ", comment).as_str();
                    },
                    "ERROR" => {
                        let error = handle_error(ctx, subnode, src);
                        temp += &error;
                    },
                    _ => {
                        if !ctx.handle_function_declarator_subnode(subnode, src, &mut temp) {
                            println!(
                                "You shouldn't be here ({} function_declarator): {}: {}\n",
                                L::LANG,
                                subnode.grammar_name(),
                                subnode.utf8_text(src.as_bytes()).unwrap_or("")
                            );
                        }
                    },
                }
            }
            result += &temp;
        }
    }
    if L::IS_CPP {
        result = utils::remove_reference_spaces(&result);
    }
    result
}

pub fn handle_pointer_declarator<L: AstLang>(ctx: &L, root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.kind() {
            "*" => {
                result += "*";
                let child_result = handle_pointer_declarator(ctx, node.named_child(0).unwrap_or(node), src);
                result += &child_result;
            },
            "pointer_declarator" => {
                let child_result = handle_pointer_declarator(ctx, node, src);
                result += &child_result;
            },
            "identifier" => {
                let identifier = handle_identifier(ctx, node, src);
                result += format!(" {}", identifier).as_str();
            },
            "qualified_identifier" => {
                let qualified_identifier = lang_node(ctx, node, src);
                result += format!(" {}", qualified_identifier).as_str();
            },
            "function_declarator" => {
                let function_declarator = handle_function_declarator(ctx, node, src);
                result += format!(" {}", function_declarator).as_str();
            },
            "parenthesized_declarator" => {
                let parenthesized_declarator = handle_parenthesized_declarator(ctx, node, src);
                result += format!(" {}", parenthesized_declarator).as_str();
            },
            "type_identifier" => {
                let type_identifier = handle_type_identifier(node, src);
                result += format!(" {}", type_identifier).as_str();
            },
            "type_qualifier" => {
                let type_qualifier = handle_type_qualifier(ctx, node, src);
                result += format!(" {}", type_qualifier).as_str();
            },
            "field_identifier" => {
                let field_identifier = handle_field_identifier(node, src);
                result += format!(" {}", field_identifier).as_str();
            },
            "array_declarator" => {
                let array_declarator = handle_array_declarator(ctx, node, src);
                result += format!(" {}", array_declarator).as_str();
            },
            "reference_declarator" => {
                let reference_declarator = lang_node(ctx, node, src);
                result += &reference_declarator;
            },
            "ms_pointer_modifier" => {
                let modifier = node.utf8_text(src.as_bytes()).unwrap_or("").trim();
                result += format!(" {}", modifier).as_str();
            },
            "ERROR" => {
                let error = handle_error(ctx, node, src);
                result += format!(" {}", error).as_str();
            },
            _ => println!("You shouldn't be here (cpp pointer_declarator): {}: {}\n", node.kind(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

pub fn handle_identifier<'a, L: AstLang>(ctx: &L, root: Node, src: &'a str) -> &'a str {
    if root.child_count() == 0 {
        return root.utf8_text(src.as_bytes()).unwrap_or("");
    }
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => return handle_identifier(ctx, node, src),
            ";" => return ";",
            _ => println!("You shouldn't be here (cpp identifier): {}: {}\n", root.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    root.utf8_text(src.as_bytes()).unwrap_or("")
}

pub fn handle_nested_inner_compound_statement<L: AstLang>(ctx: &L, root: Node, src: &str) -> String {
    return handle_inner_compound_statement(ctx, root, src);
}

pub fn handle_binary_expression<L: AstLang>(ctx: &L, root: Node, src: &str) -> String {
    let mut parts = Vec::<Cow<'_, str>>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = handle_identifier(ctx, node, src);
                parts.push(Cow::from(identifier));
            },
            "qualified_identifier" => {
                let qualified_identifier = lang_node(ctx, node, src);
                parts.push(Cow::from(qualified_identifier));
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(ctx, node, src);
                parts.push(Cow::from(parenthesized_expression));
            },
            "field_expression" => {
                let field_expression = handle_field_expression(ctx, node, src);
                parts.push(Cow::from(field_expression));
            },
            "number_literal" => {
                let number_literal = handle_number_literal(node, src);
                parts.push(Cow::from(number_literal));
            },
            "sizeof_expression" => {
                let sizeof_expression = handle_sizeof_expression(ctx, node, src);
                parts.push(Cow::from(sizeof_expression));
            },
            "subscript_expression" => {
                let subscript_expression = handle_subscript_expression(ctx, node, src);
                parts.push(Cow::from(subscript_expression));
            },
            "call_expression" => {
                let call_expression = handle_call_expression(ctx, node, src);
                parts.push(Cow::from(call_expression));
            },
            "ERROR" => {
                if L::IS_CPP {
                    let error = handle_error(ctx, node, src);
                    parts.push(Cow::from(error));
                }
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
                let binary_expression = handle_inner_binary_expression(ctx, node, src);
                parts.push(Cow::from(binary_expression));
            },
            "pointer_expression" => {
                let mut pointer_expression = handle_pointer_expression(ctx, node, src);
                if !L::IS_CPP {
                    pointer_expression = utils::remove_dereference_spaces(&pointer_expression);
                }
                parts.push(Cow::from(pointer_expression));
            },
            "cast_expression" => {
                let cast_expression = handle_cast_expression(ctx, node, src);
                parts.push(Cow::from(cast_expression));
            },
            "unary_expression" => {
                let unary_expression = handle_unary_expression(ctx, node, src);
                parts.push(Cow::from(unary_expression));
            },
            "string_literal" => {
                let string_literal = handle_string_literal(ctx, node, src);
                parts.push(Cow::from(string_literal));
            },
            "preproc_defined" => {
                let preproc_defined = handle_preproc_defined(ctx, node, src);
                parts.push(Cow::from(preproc_defined));
            },
            "offsetof_expression" => {
                let offsetof_expression = handle_offsetof_expression(ctx, node, src);
                parts.push(Cow::from(offsetof_expression));
            },
            "conditional_expression" => {
                let conditional_expression = handle_conditional_expression(ctx, node, src);
                parts.push(Cow::from(conditional_expression));
            },
            "compound_literal_expression" => {
                let compound_literal_expression = handle_compound_literal_expression(ctx, node, src);
                parts.push(Cow::from(compound_literal_expression));
            },
            "new_expression" => {
                let new_expression = lang_node(ctx, node, src);
                parts.push(Cow::from(new_expression));
            },
            "concatenated_string" => {
                let concatenated_string = handle_concatenated_string(ctx, node, src);
                parts.push(Cow::from(concatenated_string));
            },
            "raw_string_literal" => {
                let raw_string_literal = handle_raw_string_literal(node, src);
                parts.push(Cow::Borrowed(raw_string_literal));
            },
            "assignment_expression" => {
                let assignment_expression = handle_assignment_expression(ctx, node, src);
                parts.push(Cow::from(assignment_expression));
            },
            "template_function" => {
                let template_function = lang_node(ctx, node, src);
                parts.push(Cow::from(template_function));
            },
            "comment" => {
                let comment = handle_comment(ctx, node, src);
                utils::push_expr_comment(&mut parts, &comment, node.start_byte(), node.end_byte(), src);
            },
            "null" => parts.push(Cow::from(ctx.handle_null(node, src).to_string())),
            "true" => parts.push(Cow::Borrowed("true")),
            "false" => parts.push(Cow::Borrowed("false")),
            "this" => parts.push(Cow::Borrowed("this")),
            "+" => parts.push(Cow::Borrowed("+")),
            "-" => parts.push(Cow::Borrowed("-")),
            "*" => parts.push(Cow::Borrowed("*")),
            "/" => parts.push(Cow::Borrowed("/")),
            "%" => parts.push(Cow::Borrowed("%")),
            ">" => parts.push(Cow::Borrowed(">")),
            "<" => parts.push(Cow::Borrowed("<")),
            ">=" => parts.push(Cow::Borrowed(">=")),
            "<=" => parts.push(Cow::Borrowed("<=")),
            "<=>" => parts.push(Cow::Borrowed("<=>")),
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
            _ => println!("You shouldn't be here (cpp binary_expression): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let mut result = parts.join(" ");
    if !L::IS_CPP {
        result = utils::remove_unnecessary_spaces(&result);
    }
    return result;
}

pub fn handle_inner_binary_expression<L: AstLang>(ctx: &L, root: Node, src: &str) -> String {
    let mut parts = Vec::<Cow<'_, str>>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = handle_identifier(ctx, node, src);
                parts.push(Cow::from(identifier));
            },
            "qualified_identifier" => {
                let qualified_identifier = lang_node(ctx, node, src);
                parts.push(Cow::from(qualified_identifier));
            },
            "field_expression" => {
                let field_expression = handle_field_expression(ctx, node, src);
                parts.push(Cow::from(field_expression));
            },
            "unary_expression" => {
                let unary_expression = handle_unary_expression(ctx, node, src);
                parts.push(Cow::from(unary_expression));
            },
            "call_expression" => {
                let call_expression = handle_call_expression(ctx, node, src);
                parts.push(Cow::from(call_expression));
            },
            "pointer_expression" => {
                let pointer_expression = handle_pointer_expression(ctx, node, src);
                parts.push(Cow::from(pointer_expression));
            },
            "update_expression" => {
                let update_expression = handle_update_expression(node, src);
                parts.push(Cow::from(update_expression));
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(ctx, node, src);
                parts.push(Cow::from(parenthesized_expression));
            },
            "sizeof_expression" => {
                let sizeof_expression = handle_sizeof_expression(ctx, node, src);
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
                let binary_expression = handle_binary_expression(ctx, node, src);
                parts.push(Cow::from(binary_expression));
            },
            "subscript_expression" => {
                let subscript_expression = handle_subscript_expression(ctx, node, src);
                parts.push(Cow::from(subscript_expression));
            },
            "string_literal" => {
                let string_literal = handle_string_literal(ctx, node, src);
                parts.push(Cow::from(string_literal));
            },
            "preproc_defined" => {
                let preproc_defined = handle_preproc_defined(ctx, node, src);
                parts.push(Cow::from(preproc_defined));
            },
            "cast_expression" => {
                let cast_expression = handle_cast_expression(ctx, node, src);
                parts.push(Cow::from(cast_expression));
            },
            "conditional_expression" => {
                let conditional_expression = handle_conditional_expression(ctx, node, src);
                parts.push(Cow::from(conditional_expression));
            },
            "offsetof_expression" => {
                let offsetof_expression = handle_offsetof_expression(ctx, node, src);
                parts.push(Cow::from(offsetof_expression));
            },
            "compound_literal_expression" => {
                let compound_literal_expression = handle_compound_literal_expression(ctx, node, src);
                parts.push(Cow::from(compound_literal_expression));
            },
            "new_expression" => {
                let new_expression = lang_node(ctx, node, src);
                parts.push(Cow::from(new_expression));
            },
            "concatenated_string" => {
                let concatenated_string = handle_concatenated_string(ctx, node, src);
                parts.push(Cow::from(concatenated_string));
            },
            "raw_string_literal" => {
                let raw_string_literal = handle_raw_string_literal(node, src);
                parts.push(Cow::Borrowed(raw_string_literal));
            },
            "assignment_expression" => {
                let assignment_expression = handle_assignment_expression(ctx, node, src);
                parts.push(Cow::from(assignment_expression));
            },
            "template_function" => {
                let template_function = lang_node(ctx, node, src);
                parts.push(Cow::from(template_function));
            },
            "comment" => {
                let comment = handle_comment(ctx, node, src);
                utils::push_expr_comment(&mut parts, &comment, node.start_byte(), node.end_byte(), src);
            },
            "null" => parts.push(Cow::from(ctx.handle_null(node, src).to_string())),
            "this" => parts.push(Cow::Borrowed("this")),
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
            "<=>" => parts.push(Cow::Borrowed("<=>")),
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
            "ERROR" => {
                let error = handle_error(ctx, node, src);
                parts.push(Cow::from(error));
            },
            _ => println!("You shouldn't be here (cpp inner_binary_expression): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let result = parts.join(" ");
    return result;
}

pub fn handle_parenthesized_declarator<L: AstLang>(ctx: &L, root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = handle_identifier(ctx, node, src);
                result += &identifier;
            },
            "qualified_identifier" => {
                let qualified_identifier = lang_node(ctx, node, src);
                result += &qualified_identifier;
            },
            "pointer_declarator" => {
                let pointer_declarator = handle_pointer_declarator(ctx, node, src);
                result += &pointer_declarator;
            },
            "reference_declarator" => {
                let reference_declarator = lang_node(ctx, node, src);
                result += &reference_declarator;
            },
            "function_declarator" => {
                let function_declarator = handle_function_declarator(ctx, node, src);
                result += &function_declarator;
            },
            "array_declarator" => {
                let array_declarator = handle_array_declarator(ctx, node, src);
                result += &array_declarator;
            },
            "parenthesized_declarator" => {
                let parenthesized_declarator = handle_parenthesized_declarator(ctx, node, src);
                result += &parenthesized_declarator;
            },
            "pointer_expression" => {
                let pointer_expression = handle_pointer_expression(ctx, node, src);
                result += &pointer_expression;
            },
            "(" => result += "(",
            ")" => result += ")",
            _ => println!("You shouldn't be here (cpp parenthesized_declarator): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

pub fn handle_parenthesized_expression<L: AstLang>(ctx: &L, root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "call_expression" => {
                let call_expression = handle_call_expression(ctx, node, src);
                result += &call_expression;
            },
            "char_literal" => {
                let char_literal = handle_char_literal(node, src);
                result += &char_literal;
            },
            "number_literal" => {
                let number_literal = handle_number_literal(node, src);
                result += &number_literal;
            },
            "string_literal" => {
                let string_literal = handle_string_literal(ctx, node, src);
                result += &string_literal;
            },
            "concatenated_string" => {
                let concatenated_string = handle_concatenated_string(ctx, node, src);
                result += &concatenated_string;
            },
            "raw_string_literal" => {
                let raw_string_literal = handle_raw_string_literal(node, src);
                result += &raw_string_literal;
            },
            "update_expression" => {
                let update_expression = handle_update_expression(node, src);
                result += &update_expression;
            },
            "binary_expression" => {
                let binary_expression = handle_binary_expression(ctx, node, src);
                result += &binary_expression;
            },
            "unary_expression" => {
                let unary_expression = handle_unary_expression(ctx, node, src);
                result += &unary_expression;
            },
            "field_expression" => {
                let field_expression = handle_field_expression(ctx, node, src);
                result += &field_expression;
            },
            "subscript_expression" => {
                let subscript_expression = handle_subscript_expression(ctx, node, src);
                result += &subscript_expression;
            },
            "sizeof_expression" => {
                let sizeof_expression = handle_sizeof_expression(ctx, node, src);
                result += &sizeof_expression;
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(ctx, node, src);
                result += &parenthesized_expression;
            },
            "comma_expression" => {
                let comma_expression = handle_comma_expression(ctx, node, src);
                result += &comma_expression;
            },
            "compound_literal_expression" => {
                let compound_literal_expression = handle_compound_literal_expression(ctx, node, src);
                if L::IS_CPP && result.rsplit('\n').next().unwrap_or("").trim_start().starts_with('#') {
                    if let Some((head, tail)) = compound_literal_expression.split_once("){") {
                        result += head;
                        result += ")\n\t{";
                        result += tail;
                    } else {
                        result += &compound_literal_expression;
                    }
                } else {
                    result += &compound_literal_expression;
                }
            },
            "lambda_expression" => {
                let lambda_expression = lang_node(ctx, node, src);
                result += &lambda_expression;
            },
            "cast_expression" => {
                let cast_expression = handle_cast_expression(ctx, node, src);
                result += &cast_expression;
            },
            "assignment_expression" => {
                let assignment_expression = handle_assignment_expression(ctx, node, src);
                result += &assignment_expression;
            },
            "identifier" => {
                let identifier = handle_identifier(ctx, node, src);
                result += &identifier;
            },
            "qualified_identifier" => {
                let qualified_identifier = lang_node(ctx, node, src);
                result += &qualified_identifier;
            },
            "pointer_expression" => {
                let pointer_expression = handle_pointer_expression(ctx, node, src);
                result += &pointer_expression;
            },
            "conditional_expression" => {
                let conditional_expression = handle_conditional_expression(ctx, node, src);
                result += &conditional_expression;
            },
            "this" => result += "this",
            "true" => result += "true",
            "false" => result += "false",
            "null" => result += ctx.handle_null(node, src),
            "template_function" => {
                let template_function = lang_node(ctx, node, src);
                result += &template_function;
            },
            "template_type" => {
                let template_type = lang_node(ctx, node, src);
                result += &template_type;
            },
            "new_expression" => {
                let new_expression = lang_node(ctx, node, src);
                result += &new_expression;
            },
            "comment" => {
                let comment = handle_comment(ctx, node, src);
                utils::append_expr_comment(&mut result, &comment, node.start_byte(), node.end_byte(), src);
            },
            "ERROR" => {
                let error = handle_error(ctx, node, src);
                if L::IS_CPP {
                    result += &error;
                } else {
                    result += format!("{} ", error).as_str();
                }
            },
            "(" => result += "(",
            ")" => result += ")",
            _ => println!("You shouldn't be here (cpp parenthesized_expression): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

pub fn handle_pointer_expression<L: AstLang>(ctx: &L, root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = handle_identifier(ctx, node, src);
                result += &identifier;
            },
            "qualified_identifier" => {
                let qualified_identifier = lang_node(ctx, node, src);
                result += &qualified_identifier;
            },
            "pointer_expression" => {
                let pointer_expression = handle_inner_pointer_expression(ctx, node, src);
                result += &pointer_expression;
            },
            "subscript_expression" => {
                let subscript_expression = handle_subscript_expression(ctx, node, src);
                result += &subscript_expression;
            },
            "update_expression" => {
                let update_expression = handle_update_expression(node, src);
                result += &update_expression;
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(ctx, node, src);
                result += &parenthesized_expression;
            },
            "field_expression" => {
                let field_expression = handle_field_expression(ctx, node, src);
                result += &field_expression;
            },
            "cast_expression" => {
                let cast_expression = handle_cast_expression(ctx, node, src);
                result += &cast_expression;
            },
            "call_expression" => {
                let call_expression = handle_call_expression(ctx, node, src);
                result += &call_expression;
            },
            "binary_expression" => {
                let binary_expression = handle_binary_expression(ctx, node, src);
                result += &binary_expression;
            },
            "unary_expression" => {
                let unary_expression = handle_unary_expression(ctx, node, src);
                result += &unary_expression;
            },
            "*" => result += "*",
            "this" => result += "this",
            "&" => result += "&",
            ";" => result += ";",
            "ERROR" => {
                let error = handle_error(ctx, node, src);
                result += &error;
            },
            _ => println!("You shouldn't be here (cpp pointer_expression): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

pub fn handle_inner_pointer_expression<L: AstLang>(ctx: &L, root: Node, src: &str) -> String {
    return handle_pointer_expression(ctx, root, src);
}

pub fn handle_parameter_declaration<L: AstLang>(ctx: &L, root: Node, src: &str) -> String {
    let mut parts = Vec::<Cow<'_, str>>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = handle_identifier(ctx, node, src);
                parts.push(Cow::from(identifier));
            },
            "qualified_identifier" => {
                let qualified_identifier = lang_node(ctx, node, src);
                parts.push(Cow::from(qualified_identifier));
            },
            "pointer_declarator" => {
                let pointer_declarator = handle_pointer_declarator(ctx, node, src);
                parts.push(Cow::from(pointer_declarator));
            },
            "function_declarator" => {
                let function_declarator = handle_function_declarator(ctx, node, src);
                parts.push(Cow::from(function_declarator));
            },
            "parenthesized_declarator" => {
                let parenthesized_declarator = handle_parenthesized_declarator(ctx, node, src);
                parts.push(Cow::from(parenthesized_declarator));
            },
            "array_declarator" => {
                let array_declarator = handle_array_declarator(ctx, node, src);
                parts.push(Cow::from(array_declarator));
            },
            "primitive_type" => {
                let primitive_type = handle_primitive_type(node, src);
                parts.push(Cow::from(primitive_type));
            },
            "type_qualifier" => {
                let type_qualifier = handle_type_qualifier(ctx, node, src);
                parts.push(Cow::from(type_qualifier));
            },
            "storage_class_specifier" => {
                let storage_class_specifier = handle_storage_class_specifier(ctx, node, src);
                parts.push(Cow::from(storage_class_specifier));
            },
            "template_type" => {
                let template_type = lang_node(ctx, node, src);
                parts.push(Cow::from(template_type));
            },
            "struct_specifier" => {
                let struct_specifier = handle_struct_specifier(ctx, node, src);
                parts.push(Cow::from(struct_specifier));
            },
            "union_specifier" => {
                let union_specifier = handle_union_specifier(ctx, node, src);
                parts.push(Cow::from(union_specifier));
            },
            "enum_specifier" => {
                let enum_specifier = handle_enum_specifier(ctx, node, src);
                parts.push(Cow::from(enum_specifier));
            },
            "class_specifier" => {
                let class_specifier = lang_node(ctx, node, src);
                parts.push(Cow::from(class_specifier));
            },
            "placeholder_type_specifier" => {
                let placeholder_type_specifier = lang_node(ctx, node, src);
                parts.push(Cow::from(placeholder_type_specifier.trim().to_string()));
            },
            "abstract_array_declarator" => {
                let abstract_array_declarator = handle_abstract_array_declarator(ctx, node, src);
                parts.push(Cow::from(abstract_array_declarator));
            },
            "abstract_pointer_declarator" => {
                let abstract_pointer_declarator = handle_abstract_pointer_declarator(ctx, node, src);
                parts.push(Cow::from(abstract_pointer_declarator));
            },
            "abstract_function_declarator" => {
                let abstract_function_declarator = handle_abstract_function_declarator(ctx, node, src);
                parts.push(Cow::from(abstract_function_declarator));
            },
            "sized_type_specifier" => {
                let sized_type_specifier = handle_sized_type_specifier(ctx, node, src);
                parts.push(Cow::from(sized_type_specifier));
            },
            "abstract_reference_declarator" => {
                let abstract_reference_declarator = lang_node(ctx, node, src);
                parts.push(Cow::from(abstract_reference_declarator));
            },
            "reference_declarator" => {
                let reference_declarator = lang_node(ctx, node, src);
                parts.push(Cow::from(reference_declarator));
            },
            "dependent_type" => {
                let dependent_type = lang_node(ctx, node, src);
                parts.push(Cow::from(dependent_type));
            },
            "ERROR" => {
                let error = handle_error(ctx, node, src);
                parts.push(Cow::from(error));
            },
            _ => println!("You shouldn't be here (cpp parameter_declaration): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let mut result = parts.join(" ");
    result = utils::remove_unnecessary_spaces(&result);
    if L::IS_CPP {
        result = utils::remove_reference_spaces(&result);
    }
    return result;
}

pub fn handle_subscript_expression<L: AstLang>(ctx: &L, root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "[" => result += "[",
            "]" => result += "]",
            "identifier" => {
                let identifier = handle_identifier(ctx, node, src);
                result += &identifier;
            },
            "update_expression" => {
                let update_expression = handle_update_expression(node, src);
                result += &update_expression;
            },
            "binary_expression" => {
                let binary_expression = handle_binary_expression(ctx, node, src);
                result += &binary_expression;
            },
            "number_literal" => {
                let number_literal = handle_number_literal(node, src);
                result += &number_literal;
            },
            "field_expression" => {
                let field_expression = handle_field_expression(ctx, node, src);
                result += &field_expression;
            },
            "call_expression" => {
                let call_expression = handle_call_expression(ctx, node, src);
                result += &call_expression;
            },
            "cast_expression" => {
                let cast_expression = handle_cast_expression(ctx, node, src);
                result += &cast_expression;
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(ctx, node, src);
                result += &parenthesized_expression;
            },
            "subscript_expression" => {
                let subscript_expression = handle_subscript_expression(ctx, node, src);
                result += &subscript_expression;
            },
            "pointer_expression" => {
                let pointer_expression = handle_pointer_expression(ctx, node, src);
                result += &pointer_expression;
            },
            "subscript_argument_list" => {
                let subscript_argument_list = lang_node(ctx, node, src);
                result += &subscript_argument_list;
            },
            "qualified_identifier" => {
                let qualified_identifier = lang_node(ctx, node, src);
                result += &qualified_identifier;
            },
            "unary_expression" => {
                let unary_expression = handle_unary_expression(ctx, node, src);
                result += &unary_expression;
            },
            "conditional_expression" => {
                let conditional_expression = handle_conditional_expression(ctx, node, src);
                result += &conditional_expression;
            },
            "char_literal" if !L::IS_CPP => {
                let char_literal = handle_char_literal(node, src);
                result += &char_literal;
            },
            "this" if L::IS_CPP => result += "this",
            _ => println!("You shouldn't be here (cpp subscript_expression): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

pub fn handle_field_expression<L: AstLang>(ctx: &L, root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = handle_identifier(ctx, node, src);
                result += &identifier;
            },
            "qualified_identifier" => {
                let qualified_identifier = lang_node(ctx, node, src);
                result += &qualified_identifier;
            },
            "template_method" => {
                let template_method = lang_node(ctx, node, src);
                result += &template_method;
            },
            "subscript_expression" => {
                let subscript_expression = handle_subscript_expression(ctx, node, src);
                result += &subscript_expression;
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(ctx, node, src);
                result += &parenthesized_expression;
            },
            "pointer_expression" => {
                let pointer_expression = handle_pointer_expression(ctx, node, src);
                result += &pointer_expression;
            },
            "cast_expression" => {
                let cast_expression = handle_cast_expression(ctx, node, src);
                result += &cast_expression;
            },
            "call_expression" => {
                let call_expression = handle_call_expression(ctx, node, src);
                result += &call_expression;
            },
            "field_expression" => {
                let field_expression = handle_inner_field_expression(ctx, node, src);
                result += &field_expression;
            },
            "destructor_name" => {
                let destructor_name = lang_node(ctx, node, src);
                result += &destructor_name;
            },
            "ERROR" => {
                let error = handle_error(ctx, node, src);
                result += &error;
            },
            "this" => result += "this",
            "compound_literal_expression" if !L::IS_CPP => {
                let compound_literal_expression = handle_compound_literal_expression(ctx, node, src);
                result += &compound_literal_expression;
            },
            "->" => result += "->",
            "." => result += ".",
            _ => println!("You shouldn't be here (cpp field_expression): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

pub fn handle_error<L: AstLang>(ctx: &L, root: Node, src: &str) -> String {
    let raw = root.utf8_text(src.as_bytes()).unwrap_or("");
    if root.child_count() == 0 || (!raw.contains("#if") && !raw.contains("#endif")) {
        return raw.to_string();
    }
    let mut parts = Vec::<Cow<'_, str>>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                parts.push(Cow::from(handle_identifier(ctx, node, src).to_string()));
            },
            "binary_expression" => {
                parts.push(Cow::from(handle_binary_expression(ctx, node, src)));
            },
            "compound_statement" => {
                parts.push(Cow::from(handle_compound_statement(ctx, node, src)));
            },
            "bitfield_clause" => {
                parts.push(Cow::from(handle_bitfield_clause(ctx, node, src)));
            },
            "parenthesized_expression" => {
                parts.push(Cow::from(handle_parenthesized_expression(ctx, node, src)));
            },
            "primitive_type" => {
                parts.push(Cow::from(handle_primitive_type(node, src).to_string()));
            },
            "#ifdef" => {
                if L::IS_CPP {
                    parts.push(Cow::Borrowed("#ifdef"));
                }
            },
            "#endif" => {
                if L::IS_CPP {
                    parts.push(Cow::Borrowed("#endif"));
                }
            },
            "type_qualifier" if !L::IS_CPP => {
                parts.push(Cow::from(handle_type_qualifier(ctx, node, src).to_string()));
            },
            "ERROR" => {
                parts.push(Cow::from(node.utf8_text(src.as_bytes()).unwrap_or("").to_string()));
            },
            "operator" => {
                parts.push(Cow::Borrowed("operator"));
            },
            "switch" => {
                parts.push(Cow::Borrowed("switch"));
            },
            "{" => parts.push(Cow::Borrowed("{")),
            "}" => parts.push(Cow::Borrowed("}")),
            ";" => parts.push(Cow::Borrowed(";")),
            "::" => {
                if !node.is_missing() {
                    parts.push(Cow::Borrowed("::"));
                }
            },
            ":" => parts.push(Cow::Borrowed(":")),
            "=" => parts.push(Cow::Borrowed("=")),
            "(" => parts.push(Cow::Borrowed("(")),
            ")" => parts.push(Cow::Borrowed(")")),
            "-" => parts.push(Cow::Borrowed("-")),
            ">" => parts.push(Cow::Borrowed(">")),
            "->" => parts.push(Cow::Borrowed("->")),
            "." => parts.push(Cow::Borrowed(".")),
            "?" => parts.push(Cow::Borrowed("?")),
            "," => parts.push(Cow::Borrowed(",")),
            "false" => parts.push(Cow::Borrowed("false")),
            "true" => parts.push(Cow::Borrowed("true")),
            _ => parts.push(Cow::from(node.utf8_text(src.as_bytes()).unwrap_or("").to_string())),
        }
    }
    return parts.join(" ");
}

pub fn handle_cast_expression<L: AstLang>(ctx: &L, root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "(" => result += "(",
            ")" => result += ")",
            "null" => result += ctx.handle_null(node, src),
            "this" => result += "this",
            "true" => result += "true",
            "false" => result += "false",
            "type_descriptor" => {
                let type_descriptor = handle_type_descriptor(ctx, node, src);
                result += &type_descriptor;
            },
            "identifier" => {
                let identifier = handle_identifier(ctx, node, src);
                result += &identifier;
            },
            "qualified_identifier" => {
                let qualified_identifier = lang_node(ctx, node, src);
                result += &qualified_identifier;
            },
            "call_expression" => {
                let call_expression = handle_call_expression(ctx, node, src);
                result += &call_expression;
            },
            "cast_expression" => {
                let cast_expression = handle_cast_expression(ctx, node, src);
                result += &cast_expression;
            },
            "pointer_expression" => {
                let pointer_expression = handle_pointer_expression(ctx, node, src);
                result += &pointer_expression;
            },
            "subscript_expression" => {
                let subscript_expression = handle_subscript_expression(ctx, node, src);
                result += &subscript_expression;
            },
            "field_expression" => {
                let field_expression = handle_field_expression(ctx, node, src);
                result += &field_expression;
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(ctx, node, src);
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
                let string_literal = handle_string_literal(ctx, node, src);
                result += &string_literal;
            },
            "concatenated_string" if !L::IS_CPP => {
                let concatenated_string = handle_concatenated_string(ctx, node, src);
                result += &concatenated_string;
            },
            "unary_expression" => {
                let unary_expression = handle_unary_expression(ctx, node, src);
                result += &unary_expression;
            },
            "binary_expression" => {
                let binary_expression = handle_binary_expression(ctx, node, src);
                result += &binary_expression;
            },
            "sizeof_expression" => {
                let sizeof_expression = handle_sizeof_expression(ctx, node, src);
                result += &sizeof_expression;
            },
            "conditional_expression" => {
                let conditional_expression = handle_conditional_expression(ctx, node, src);
                result += &conditional_expression;
            },
            "update_expression" => {
                let update_expression = handle_update_expression(node, src);
                result += &update_expression;
            },
            _ => println!("You shouldn't be here (cpp cast_expression): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

pub fn handle_init_declarator<L: AstLang>(ctx: &L, root: Node, src: &str) -> String {
    let mut parts = Vec::<Cow<'_, str>>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = handle_identifier(ctx, node, src);
                parts.push(Cow::from(identifier));
            },
            "qualified_identifier" => {
                let qualified_identifier = lang_node(ctx, node, src);
                parts.push(Cow::from(qualified_identifier));
            },
            "number_literal" => {
                let number_literal = handle_number_literal(node, src);
                parts.push(Cow::from(number_literal));
            },
            "call_expression" => {
                let call_expression = handle_call_expression(ctx, node, src);
                parts.push(Cow::from(call_expression));
            },
            "field_expression" => {
                let field_expression = handle_field_expression(ctx, node, src);
                parts.push(Cow::from(field_expression));
            },
            "subscript_expression" => {
                let subscript_expression = handle_subscript_expression(ctx, node, src);
                parts.push(Cow::from(subscript_expression));
            },
            "pointer_declarator" => {
                let pointer_declarator = handle_pointer_declarator(ctx, node, src);
                parts.push(Cow::from(pointer_declarator));
            },
            "function_declarator" => {
                let function_declarator = handle_function_declarator(ctx, node, src);
                parts.push(Cow::from(function_declarator));
            },
            "parenthesized_declarator" => {
                let parenthesized_declarator = handle_parenthesized_declarator(ctx, node, src);
                parts.push(Cow::from(parenthesized_declarator));
            },
            "structured_binding_declarator" => {
                let structured_binding_declarator = lang_node(ctx, node, src);
                parts.push(Cow::from(structured_binding_declarator));
            },
            "pointer_expression" => {
                let pointer_expression = handle_pointer_expression(ctx, node, src);
                parts.push(Cow::from(pointer_expression));
            },
            "reference_declarator" => {
                let reference_declarator = lang_node(ctx, node, src);
                parts.push(Cow::from(reference_declarator));
            },
            "cast_expression" => {
                let cast_expression = handle_cast_expression(ctx, node, src);
                parts.push(Cow::from(cast_expression));
            },
            "binary_expression" => {
                let binary_expression = handle_binary_expression(ctx, node, src);
                parts.push(Cow::from(binary_expression));
            },
            "initializer_list" => {
                let initializer_list = handle_initializer_list(ctx, node, src);
                if !L::IS_CPP || parts.last().map(|s| s == "=" || s == "return" || s == "?" || s == ":").unwrap_or(false) || parts.is_empty() {
                    parts.push(Cow::from(initializer_list));
                } else if let Some(last) = parts.last_mut() {
                    *last = Cow::Owned(format!("{}{}", last, initializer_list));
                }
            },
            "compound_literal_expression" => {
                let compound_literal_expression = handle_compound_literal_expression(ctx, node, src);
                parts.push(Cow::from(compound_literal_expression));
            },
            "sizeof_expression" => {
                let sizeof_expression = handle_sizeof_expression(ctx, node, src);
                parts.push(Cow::from(sizeof_expression));
            },
            "offsetof_expression" => {
                let offsetof_expression = handle_offsetof_expression(ctx, node, src);
                parts.push(Cow::from(offsetof_expression));
            },
            "unary_expression" => {
                let unary_expression = handle_unary_expression(ctx, node, src);
                parts.push(Cow::from(unary_expression));
            },
            "conditional_expression" => {
                let conditional_expression = handle_conditional_expression(ctx, node, src);
                parts.push(Cow::from(conditional_expression));
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(ctx, node, src);
                if !L::IS_CPP || parts.last().map(|s| s == "=" || s == "return" || s == "?" || s == ":").unwrap_or(false) || parts.is_empty() {
                    parts.push(Cow::from(parenthesized_expression));
                } else if let Some(last) = parts.last_mut() {
                    *last = Cow::Owned(format!("{}{}", last, parenthesized_expression));
                }
            },
            "array_declarator" => {
                let array_declarator = handle_array_declarator(ctx, node, src);
                parts.push(Cow::from(array_declarator));
            },
            "string_literal" => {
                let string_literal = handle_string_literal(ctx, node, src);
                parts.push(Cow::from(string_literal));
            },
            "concatenated_string" => {
                let concatenated_string = handle_concatenated_string(ctx, node, src);
                parts.push(Cow::from(concatenated_string));
            },
            "raw_string_literal" => {
                let raw_string_literal = handle_raw_string_literal(node, src);
                parts.push(Cow::from(raw_string_literal));
            },
            "argument_list" => {
                let argument_list = handle_argument_list(ctx, node, src);
                if !L::IS_CPP || parts.last().map(|s| s == "=" || s == "return" || s == "?" || s == ":").unwrap_or(false) || parts.is_empty() {
                    parts.push(Cow::from(argument_list));
                } else if let Some(last) = parts.last_mut() {
                    *last = Cow::Owned(format!("{}{}", last, argument_list));
                }
            },
            "char_literal" => {
                let char_literal = handle_char_literal(node, src);
                parts.push(Cow::from(char_literal));
            },
            "new_expression" => {
                let new_expression = lang_node(ctx, node, src);
                parts.push(Cow::from(new_expression));
            },
            "lambda_expression" => {
                let mut lambda_expression = lang_node(ctx, node, src);
                lambda_expression = utils::add_all_leading_tabs(&lambda_expression);
                parts.push(Cow::from(lambda_expression));
            },
            "update_expression" => {
                let update_expression = handle_update_expression(node, src);
                parts.push(Cow::from(update_expression));
            },
            "assignment_expression" => {
                let assignment_expression = handle_assignment_expression(ctx, node, src);
                parts.push(Cow::from(assignment_expression));
            },
            "comma_expression" => {
                let comma_expression = handle_comma_expression(ctx, node, src);
                parts.push(Cow::from(comma_expression));
            },
            "template_function" => {
                let template_function = lang_node(ctx, node, src);
                parts.push(Cow::from(template_function));
            },
            "comment" => {
                let comment = handle_comment(ctx, node, src);
                utils::push_expr_comment(&mut parts, &comment, node.start_byte(), node.end_byte(), src);
            },
            "ERROR" => {
                let error = handle_error(ctx, node, src);
                parts.push(Cow::from(error));
            },
            "=" => parts.push(Cow::Borrowed("=")),
            "this" => parts.push(Cow::Borrowed("this")),
            "false" => parts.push(Cow::Borrowed("false")),
            "true" => parts.push(Cow::Borrowed("true")),
            "null" => parts.push(Cow::from(ctx.handle_null(node, src).to_string())),
            _ => println!("You shouldn't be here (cpp init_declarator): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let result = parts.join(" ");
    return result;
}

pub fn handle_type_descriptor<L: AstLang>(ctx: &L, root: Node, src: &str) -> String {
    let mut parts = Vec::<Cow<'_, str>>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = handle_identifier(ctx, node, src);
                parts.push(Cow::from(identifier));
            },
            "qualified_identifier" => {
                let qualified_identifier = lang_node(ctx, node, src);
                parts.push(Cow::from(qualified_identifier));
            },
            "primitive_type" => {
                let primitive_type = handle_primitive_type(node, src);
                parts.push(Cow::from(primitive_type));
            },
            "abstract_pointer_declarator" => {
                let abstract_pointer_declaration = handle_abstract_pointer_declarator(ctx, node, src);
                parts.push(Cow::from(abstract_pointer_declaration));
            },
            "abstract_array_declarator" => {
                let abstract_array_declarator = handle_abstract_array_declarator(ctx, node, src);
                parts.push(Cow::from(abstract_array_declarator));
            },
            "abstract_function_declarator" => {
                let abstract_function_declarator = handle_abstract_function_declarator(ctx, node, src);
                parts.push(Cow::from(abstract_function_declarator));
            },
            "struct_specifier" => {
                let struct_specifier = handle_struct_specifier(ctx, node, src);
                parts.push(Cow::from(struct_specifier.trim_end().to_string()));
            },
            "union_specifier" => {
                let union_specifier = handle_union_specifier(ctx, node, src);
                parts.push(Cow::from(union_specifier.trim_end().to_string()));
            },
            "enum_specifier" => {
                let enum_specifier = handle_enum_specifier(ctx, node, src);
                parts.push(Cow::from(enum_specifier.trim_end().to_string()));
            },
            "class_specifier" => {
                let class_specifier = lang_node(ctx, node, src);
                parts.push(Cow::from(class_specifier.trim_end().to_string()));
            },
            "sized_type_specifier" => {
                let sized_type_specifier = handle_sized_type_specifier(ctx, node, src);
                parts.push(Cow::from(sized_type_specifier));
            },
            "template_type" => {
                let template_type = lang_node(ctx, node, src);
                parts.push(Cow::from(template_type));
            },
            "type_qualifier" => {
                let type_qualifier = handle_type_qualifier(ctx, node, src);
                parts.push(Cow::from(type_qualifier));
            },
            "abstract_reference_declarator" => {
                let abstract_reference_declarator = lang_node(ctx, node, src);
                parts.push(Cow::from(abstract_reference_declarator));
            },
            "decltype" => {
                let decltype = lang_node(ctx, node, src);
                parts.push(Cow::from(decltype));
            },
            "dependent_type" => {
                let dependent_type = lang_node(ctx, node, src);
                parts.push(Cow::from(dependent_type));
            },
            "placeholder_type_specifier" => {
                let placeholder_type_specifier = lang_node(ctx, node, src);
                parts.push(Cow::from(placeholder_type_specifier.trim().to_string()));
            },
            "ERROR" if !L::IS_CPP => {
                let error = handle_error(ctx, node, src);
                parts.push(Cow::from(error));
            },
            _ => println!("You shouldn't be here (cpp type_descriptor): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let mut result = parts.join(" ");
    result = utils::remove_pointer_spaces(&result);
    if L::IS_CPP {
        result = utils::remove_reference_spaces(&result);
    }
    return result;
}

pub fn handle_initializer_list<L: AstLang>(ctx: &L, root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = handle_identifier(ctx, node, src);
                result += &identifier;
            },
            "qualified_identifier" => {
                let qualified_identifier = lang_node(ctx, node, src);
                result += &qualified_identifier;
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
                let comment = handle_comment(ctx, node, src);
                utils::append_expr_comment(&mut result, &comment, node.start_byte(), node.end_byte(), src);
            },
            "initializer_list" => {
                let initializer_list = handle_initializer_list(ctx, node, src);
                result += &initializer_list;
            },
            "string_literal" => {
                let string_literal = handle_string_literal(ctx, node, src);
                result += &string_literal;
            },
            "call_expression" => {
                let call_expression = handle_call_expression(ctx, node, src);
                result += &call_expression;
            },
            "pointer_expression" => {
                let pointer_expression = handle_pointer_expression(ctx, node, src);
                result += &pointer_expression;
            },
            "conditional_expression" => {
                let conditional_expression = handle_conditional_expression(ctx, node, src);
                result += &conditional_expression;
            },
            "binary_expression" => {
                let binary_expression = handle_binary_expression(ctx, node, src);
                result += &binary_expression;
            },
            "initializer_pair" => {
                let initializer_pair = handle_initializer_pair(ctx, node, src);
                result += &initializer_pair;
            },
            "cast_expression" => {
                let cast_expression = handle_cast_expression(ctx, node, src);
                result += &cast_expression;
            },
            "unary_expression" => {
                let unary_expression = handle_unary_expression(ctx, node, src);
                result += &unary_expression;
            },
            "field_expression" => {
                let field_expression = handle_field_expression(ctx, node, src);
                result += &field_expression;
            },
            "subscript_expression" => {
                let subscript_expression = handle_subscript_expression(ctx, node, src);
                result += &subscript_expression;
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(ctx, node, src);
                result += &parenthesized_expression;
            },
            "sizeof_expression" => {
                let sizeof_expression = handle_sizeof_expression(ctx, node, src);
                result += &sizeof_expression;
            },
            "compound_literal_expression" => {
                let compound_literal_expression = handle_compound_literal_expression(ctx, node, src);
                if L::IS_CPP && result.rsplit('\n').next().unwrap_or("").trim_start().starts_with('#') {
                    if let Some((head, tail)) = compound_literal_expression.split_once("){") {
                        result += head;
                        result += ")\n\t{";
                        result += tail;
                    } else {
                        result += &compound_literal_expression;
                    }
                } else {
                    result += &compound_literal_expression;
                }
            },
            "concatenated_string" => {
                let concatenated_string = handle_concatenated_string(ctx, node, src);
                result += &concatenated_string;
            },
            "raw_string_literal" => {
                let raw_string_literal = handle_raw_string_literal(node, src);
                result += &raw_string_literal;
            },
            "template_function" => {
                let template_function = lang_node(ctx, node, src);
                result += &template_function;
            },
            "lambda_expression" => {
                let mut lambda_expression = lang_node(ctx, node, src);
                lambda_expression = utils::add_all_leading_tabs(&lambda_expression).trim_start().to_string();
                result += &lambda_expression;
            },
            "new_expression" => {
                let new_expression = lang_node(ctx, node, src);
                result += &new_expression;
            },
            "update_expression" => {
                let update_expression = handle_update_expression(node, src);
                result += &update_expression;
            },
            "assignment_expression" => {
                let assignment_expression = handle_assignment_expression(ctx, node, src);
                result += &assignment_expression;
            },
            "ERROR" => {
                let error = handle_error(ctx, node, src);
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
            "null" => result += ctx.handle_null(node, src),
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

pub fn handle_sizeof_expression<L: AstLang>(ctx: &L, root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "sizeof" => result += "sizeof",
            "..." => result += "...",
            "(" => result += "(",
            ")" => result += ")",
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(ctx, node, src);
                result += &parenthesized_expression;
            },
            "type_descriptor" => {
                let type_descriptor = handle_type_descriptor(ctx, node, src);
                result += &type_descriptor;
            },
            "identifier" => {
                let identifier = handle_identifier(ctx, node, src);
                result += format!(" {}", identifier).as_str();
            },
            "qualified_identifier" => {
                let qualified_identifier = lang_node(ctx, node, src);
                result += format!(" {}", qualified_identifier).as_str();
            },
            "field_expression" => {
                let field_expression = handle_field_expression(ctx, node, src);
                result += format!(" {}", field_expression).as_str();
            },
            "subscript_expression" => {
                let subscript_expression = handle_subscript_expression(ctx, node, src);
                result += format!(" {}", subscript_expression).as_str();
            },
            "pointer_expression" => {
                let pointer_expression = handle_pointer_expression(ctx, node, src);
                result += format!(" {}", pointer_expression).as_str();
            },
            "unary_expression" => {
                let unary_expression = handle_unary_expression(ctx, node, src);
                result += format!(" {}", unary_expression).as_str();
            },
            "call_expression" => {
                let call_expression = handle_call_expression(ctx, node, src);
                result += format!(" {}", call_expression).as_str();
            },
            "string_literal" if !L::IS_CPP => {
                let string_literal = handle_string_literal(ctx, node, src);
                result += format!(" {}", string_literal).as_str();
            },
            _ => println!("You shouldn't be here (cpp sizeof_expression): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    if L::IS_CPP {
        result = result.replace("( ", "(");
    }
    return result;
}

pub fn handle_type_qualifier<'a, L: AstLang>(ctx: &L, root: Node, src: &'a str) -> &'a str {
    let mut result = "";
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "const" => result = "const",
            "constexpr" => result = "constexpr",
            "consteval" => result = "consteval",
            "constinit" => result = "constinit",
            "volatile" => result = "volatile",
            "mutable" => result = "mutable",
            "restrict" => result = "restrict",
            "__restrict" => result = "__restrict",
            "__restrict__" => result = "__restrict__",
            "_Atomic" if !L::IS_CPP => result = "_Atomic",
            _ => println!("You shouldn't be here (cpp type_qualifier): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    if result.is_empty() {
        return root.utf8_text(src.as_bytes()).unwrap_or("");
    }
    result
}

pub fn handle_abstract_pointer_declarator<L: AstLang>(ctx: &L, root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "abstract_pointer_declarator" => {
                let inner = handle_abstract_pointer_declarator(ctx, node, src);
                result += &inner;
            },
            "abstract_reference_declarator" => {
                let inner = lang_node(ctx, node, src);
                result += &inner;
            },
            "abstract_function_declarator" => {
                let inner = handle_abstract_function_declarator(ctx, node, src);
                result += &inner;
            },
            "abstract_array_declarator" => {
                let inner = handle_abstract_array_declarator(ctx, node, src);
                result += &inner;
            },
            "abstract_parenthesized_declarator" => {
                let inner = handle_abstract_parenthesized_declarator(ctx, node, src);
                result += &inner;
            },
            "ms_pointer_modifier" => {
                let modifier = node.utf8_text(src.as_bytes()).unwrap_or("").trim();
                result += format!(" {}", modifier).as_str();
            },
            "type_qualifier" => {
                let type_qualifier = handle_type_qualifier(ctx, node, src);
                result += format!(" {}", type_qualifier).as_str();
            },
            "*" => result += "*",
            _ => println!("You shouldn't be here (cpp abstract_pointer_declarator): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

pub fn handle_conditional_expression<L: AstLang>(ctx: &L, root: Node, src: &str) -> String {
    let mut parts = Vec::<Cow<'_, str>>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = handle_identifier(ctx, node, src);
                parts.push(Cow::from(identifier));
            },
            "qualified_identifier" => {
                let qualified_identifier = lang_node(ctx, node, src);
                parts.push(Cow::from(qualified_identifier));
            },
            "call_expression" => {
                let call_expression = handle_call_expression(ctx, node, src);
                parts.push(Cow::from(call_expression));
            },
            "cast_expression" => {
                let cast_expression = handle_cast_expression(ctx, node, src);
                parts.push(Cow::from(cast_expression));
            },
            "field_expression" => {
                let field_expression = handle_field_expression(ctx, node, src);
                parts.push(Cow::from(field_expression));
            },
            "binary_expression" => {
                let binary_expression = handle_binary_expression(ctx, node, src);
                parts.push(Cow::from(binary_expression));
            },
            "unary_expression" => {
                let unary_expression = handle_unary_expression(ctx, node, src);
                parts.push(Cow::from(unary_expression));
            },
            "pointer_expression" => {
                let pointer_expression = handle_pointer_expression(ctx, node, src);
                parts.push(Cow::from(pointer_expression));
            },
            "sizeof_expression" => {
                let sizeof_expression = handle_sizeof_expression(ctx, node, src);
                parts.push(Cow::from(sizeof_expression));
            },
            "conditional_expression" => {
                let conditional_expression = handle_conditional_expression(ctx, node, src);
                parts.push(Cow::from(conditional_expression));
            },
            "assignment_expression" => {
                let assignment_expression = handle_assignment_expression(ctx, node, src);
                parts.push(Cow::from(assignment_expression));
            },
            "update_expression" => {
                let update_expression = handle_update_expression(node, src);
                parts.push(Cow::from(update_expression));
            },
            "initializer_list" => {
                let initializer_list = handle_initializer_list(ctx, node, src);
                if !L::IS_CPP || parts.last().map(|s| s == "=" || s == "return" || s == "?" || s == ":").unwrap_or(false) || parts.is_empty() {
                    parts.push(Cow::from(initializer_list));
                } else if let Some(last) = parts.last_mut() {
                    *last = Cow::Owned(format!("{}{}", last, initializer_list));
                }
            },
            "compound_literal_expression" => {
                let compound_literal_expression = handle_compound_literal_expression(ctx, node, src);
                parts.push(Cow::from(compound_literal_expression));
            },
            "new_expression" => {
                let new_expression = lang_node(ctx, node, src);
                parts.push(Cow::from(new_expression));
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(ctx, node, src);
                parts.push(Cow::from(parenthesized_expression));
            },
            "comment" => {
                let comment = handle_comment(ctx, node, src);
                utils::push_expr_comment(&mut parts, &comment, node.start_byte(), node.end_byte(), src);
            },
            "subscript_expression" => {
                let subscript_expression = handle_subscript_expression(ctx, node, src);
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
                let string_literal = handle_string_literal(ctx, node, src);
                parts.push(Cow::from(string_literal));
            },
            "concatenated_string" => {
                let concatenated_string = handle_concatenated_string(ctx, node, src);
                parts.push(Cow::from(concatenated_string));
            },
            "raw_string_literal" => {
                let raw_string_literal = handle_raw_string_literal(node, src);
                parts.push(Cow::from(raw_string_literal));
            },
            "null" => parts.push(Cow::from(ctx.handle_null(node, src).to_string())),
            "this" => parts.push(Cow::Borrowed("this")),
            "true" => parts.push(Cow::Borrowed("true")),
            "false" => parts.push(Cow::Borrowed("false")),
            ":" => parts.push(Cow::Borrowed(":")),
            "?" => parts.push(Cow::Borrowed("?")),
            _ => println!("You shouldn't be here (cpp conditional_expression): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let result = parts.join(" ");
    return result;
}

pub fn handle_unary_expression<L: AstLang>(ctx: &L, root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = handle_identifier(ctx, node, src);
                result += &identifier;
            },
            "call_expression" => {
                let call_expression = handle_call_expression(ctx, node, src);
                result += &call_expression;
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(ctx, node, src);
                result += &parenthesized_expression;
            },
            "field_expression" => {
                let field_expression = handle_field_expression(ctx, node, src);
                result += &field_expression;
            },
            "pointer_expression" => {
                let pointer_expression = handle_pointer_expression(ctx, node, src);
                result += &pointer_expression;
            },
            "number_literal" => {
                let number_literal = handle_number_literal(node, src);
                result += &number_literal;
            },
            "compound_literal_expression" => {
                let compound_literal_expression = handle_compound_literal_expression(ctx, node, src);
                if L::IS_CPP && result.rsplit('\n').next().unwrap_or("").trim_start().starts_with('#') {
                    if let Some((head, tail)) = compound_literal_expression.split_once("){") {
                        result += head;
                        result += ")\n\t{";
                        result += tail;
                    } else {
                        result += &compound_literal_expression;
                    }
                } else {
                    result += &compound_literal_expression;
                }
            },
            "assignment_expression" => {
                let assignment_expression = handle_assignment_expression(ctx, node, src);
                result += &assignment_expression;
            },
            "preproc_defined" => {
                let preproc_defined = handle_preproc_defined(ctx, node, src);
                result += &preproc_defined;
            },
            "qualified_identifier" => {
                let qualified_identifier = lang_node(ctx, node, src);
                result += &qualified_identifier;
            },
            "subscript_expression" => {
                let subscript_expression = handle_subscript_expression(ctx, node, src);
                result += &subscript_expression;
            },
            "cast_expression" => {
                let cast_expression = handle_cast_expression(ctx, node, src);
                result += &cast_expression;
            },
            "unary_expression" => {
                let unary_expression = handle_unary_expression(ctx, node, src);
                result += &unary_expression;
            },
            "sizeof_expression" => {
                let sizeof_expression = handle_sizeof_expression(ctx, node, src);
                result += &sizeof_expression;
            },
            "char_literal" => {
                let char_literal = handle_char_literal(node, src);
                result += &char_literal;
            },
            "ERROR" => {
                let error = handle_error(ctx, node, src);
                if L::IS_CPP {
                    result += &error;
                } else {
                    result += format!("{} ", error).as_str();
                }
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

pub fn handle_array_declarator<L: AstLang>(ctx: &L, root: Node, src: &str) -> String {
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
                let identifier = handle_identifier(ctx, node, src);
                parts.push(Cow::from(identifier));
            },
            "qualified_identifier" => {
                let qualified_identifier = lang_node(ctx, node, src);
                parts.push(Cow::from(qualified_identifier));
            },
            "binary_expression" => {
                let binary_expression = handle_binary_expression(ctx, node, src);
                parts.push(Cow::from(binary_expression));
            },
            "cast_expression" => {
                let cast_expression = handle_cast_expression(ctx, node, src);
                parts.push(Cow::from(cast_expression));
            },
            "call_expression" => {
                let call_expression = handle_call_expression(ctx, node, src);
                parts.push(Cow::from(call_expression));
            },
            "sizeof_expression" => {
                let sizeof_expression = handle_sizeof_expression(ctx, node, src);
                parts.push(Cow::from(sizeof_expression));
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(ctx, node, src);
                parts.push(Cow::from(parenthesized_expression));
            },
            "field_expression" => {
                let field_expression = handle_field_expression(ctx, node, src);
                parts.push(Cow::from(field_expression));
            },
            "subscript_expression" => {
                let subscript_expression = handle_subscript_expression(ctx, node, src);
                parts.push(Cow::from(subscript_expression));
            },
            "pointer_declarator" => {
                let pointer_declarator = handle_pointer_declarator(ctx, node, src);
                parts.push(Cow::from(pointer_declarator));
            },
            "parenthesized_declarator" => {
                let parenthesized_declarator = handle_parenthesized_declarator(ctx, node, src);
                parts.push(Cow::from(parenthesized_declarator));
            },
            "array_declarator" => {
                let array_declarator = handle_array_declarator(ctx, node, src);
                parts.push(Cow::from(array_declarator));
            },
            "ERROR" => {
                let error = handle_error(ctx, node, src);
                parts.push(Cow::from(error));
            },
            "type_qualifier" if !L::IS_CPP => {
                let type_qualifier = handle_type_qualifier(ctx, node, src);
                parts.push(Cow::from(type_qualifier));
            },
            "static" if !L::IS_CPP => parts.push(Cow::Borrowed("static")),
            "*" if !L::IS_CPP => parts.push(Cow::Borrowed("*")),
            "[" => parts.push(Cow::Borrowed("[")),
            "]" => parts.push(Cow::Borrowed("]")),
            _ => println!("You shouldn't be here (cpp array_declarator): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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

pub fn handle_string_literal<L: AstLang>(ctx: &L, root: Node, src: &str) -> String {
    let mut parts = Vec::<Cow<'_, str>>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "string_content" | "escape_sequence" | "\"" | "L\"" | "u\"" | "U\"" | "u8\"" => {
                parts.push(Cow::Borrowed(node.utf8_text(src.as_bytes()).unwrap_or("")));
            },
            "ERROR" => {
                parts.push(Cow::from(handle_error(ctx, node, src)));
            },
            _ => println!("You shouldn't be here (cpp string_literal): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let result = parts.join("");
    return result;
}

pub fn handle_concatenated_string<L: AstLang>(ctx: &L, root: Node, src: &str) -> String {
    let mut parts = Vec::<Cow<'_, str>>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "string_literal" => {
                let string_literal = handle_string_literal(ctx, node, src);
                parts.push(Cow::from(string_literal));
            },
            "raw_string_literal" => {
                let raw_string_literal = handle_raw_string_literal(node, src);
                parts.push(Cow::Borrowed(raw_string_literal));
            },
            "identifier" => {
                let identifier = handle_identifier(ctx, node, src);
                parts.push(Cow::Borrowed(identifier));
            },
            "call_expression" => {
                let call_expression = handle_call_expression(ctx, node, src);
                parts.push(Cow::from(call_expression));
            },
            "comment" => {
                let comment = handle_comment(ctx, node, src);
                utils::push_expr_comment(&mut parts, &comment, node.start_byte(), node.end_byte(), src);
            },
            "ERROR" => {
                let error = handle_error(ctx, node, src);
                parts.push(Cow::from(error));
            },
            _ => println!("You shouldn't be here (cpp concatenated_string): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let result = parts.join(" ");
    return result;
}

pub fn handle_labeled_statement<L: AstLang>(ctx: &L, root: Node, src: &str) -> String {
    let same_line = root.start_position().row == root.end_position().row;
    let sep = if same_line { " " } else if L::IS_CPP { "\t" } else { "\n\t" };
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "expression_statement" => {
                let expression_statement = handle_expression_statement(ctx, node, src);
                if same_line && expression_statement.trim() == ";" {
                    result += ";";
                } else {
                    result += format!("{}{}", sep, expression_statement).as_str();
                }
            },
            "identifier" => {
                let identifier = handle_identifier(ctx, node, src);
                result += &identifier;
            },
            "return_statement" => {
                let return_statement = handle_return_statement(ctx, node, src);
                result += format!("{}{}", sep, return_statement).as_str();
            },
            "if_statement" => {
                let if_statement = handle_if_statement(ctx, node, src);
                result += format!("{}{}", sep, if_statement).as_str();
            },
            "switch_statement" => {
                let switch_statement = handle_switch_statement(ctx, node, src);
                result += format!("{}{}", sep, switch_statement).as_str();
            },
            "while_statement" => {
                let while_statement = handle_while_statement(ctx, node, src);
                result += format!("{}{}", sep, while_statement).as_str();
            },
            "for_statement" => {
                let for_statement = handle_for_statement(ctx, node, src);
                result += format!("{}{}", sep, for_statement).as_str();
            },
            "for_range_loop" => {
                let for_range_loop = lang_node(ctx, node, src);
                result += format!("{}{}", sep, for_range_loop).as_str();
            },
            "do_statement" => {
                let do_statement = handle_do_statement(ctx, node, src);
                result += format!("{}{}", sep, do_statement).as_str();
            },
            "goto_statement" => {
                let goto_statement = handle_goto_statement(ctx, node, src);
                result += format!("{}{}", sep, goto_statement).as_str();
            },
            "declaration" => {
                let declaration = handle_declaration(ctx, node, src);
                result += format!("{}{}", sep, declaration).as_str();
            },
            "compound_statement" => {
                let compound_statement = handle_inner_compound_statement(ctx, node, src);
                result += format!("{}{}", sep, compound_statement).as_str();
            },
            "labeled_statement" => {
                let labeled_statement = handle_labeled_statement(ctx, node, src);
                if !L::IS_CPP {
                    result += format!("{}{}", sep, labeled_statement).as_str();
                } else if same_line {
                    result += format!(" {}", labeled_statement.trim_start()).as_str();
                } else {
                    result += format!("\n{}", labeled_statement).as_str();
                }
            },
            "comment" => {
                let comment = handle_comment(ctx, node, src);
                if L::IS_CPP {
                    utils::append_comment_to_result(&mut result, &comment, node.start_byte(), src, "\t", "\n", true);
                } else {
                    utils::append_comment_to_result(&mut result, &comment, node.start_byte(), src, "\n\t", "", false);
                }
            },
            "break_statement" => result += format!("{}break;", sep).as_str(),
            "continue_statement" => result += format!("{}continue;", sep).as_str(),
            ":" => {
                if !L::IS_CPP || same_line {
                    result += ":";
                } else {
                    result += ":\n";
                }
            },
            ";" => result += ";",
            _ => println!("You shouldn't be here (cpp labeled_statement): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    if L::IS_CPP {
        result = utils::add_all_leading_tabs(&result);
    }
    return result;
}

pub fn handle_for_statement<L: AstLang>(ctx: &L, root: Node, src: &str) -> String {
    let mut vec = Vec::<Cow<'_, str>>::new();
    let mut temp = String::new();
    let mut reached_header_end = false;
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => {
                let compound_statement = handle_compound_statement(ctx, node, src);
                vec.push(Cow::from(compound_statement));
            },
            "if_statement" => {
                let if_statement = handle_if_statement(ctx, node, src);
                vec.push(Cow::from(if_statement));
            },
            "while_statement" => {
                let while_statement = handle_while_statement(ctx, node, src);
                vec.push(Cow::from(while_statement));
            },
            "for_statement" => {
                let for_statement = handle_for_statement(ctx, node, src);
                vec.push(Cow::from(for_statement));
            },
            "for_range_loop" => {
                let for_range_loop = lang_node(ctx, node, src);
                vec.push(Cow::from(for_range_loop));
            },
            "do_statement" => {
                let do_statement = handle_do_statement(ctx, node, src);
                vec.push(Cow::from(do_statement));
            },
            "switch_statement" => {
                let switch_statement = handle_switch_statement(ctx, node, src);
                vec.push(Cow::from(switch_statement));
            },
            "return_statement" => {
                let return_statement = handle_return_statement(ctx, node, src);
                vec.push(Cow::from(return_statement));
            },
            "break_statement" => vec.push(Cow::Borrowed("break;")),
            "continue_statement" => vec.push(Cow::Borrowed("continue;")),
            "goto_statement" => {
                let goto_statement = handle_goto_statement(ctx, node, src);
                vec.push(Cow::from(goto_statement));
            },
            "update_expression" => {
                let update_expression = handle_update_expression(node, src);
                if !reached_header_end { temp += &update_expression; }
                else { vec.push(Cow::from(update_expression)); }
            },
            "assignment_expression" => {
                let assignment_expression = handle_assignment_expression(ctx, node, src);
                if !reached_header_end { temp += &assignment_expression; }
                else { vec.push(Cow::from(assignment_expression)); }
            },
            "binary_expression" => {
                let binary_expression = handle_binary_expression(ctx, node, src);
                if !reached_header_end { temp += &binary_expression; }
                else { vec.push(Cow::from(binary_expression)); }
            },
            "call_expression" => {
                let call_expression = handle_call_expression(ctx, node, src);
                if !reached_header_end { temp += &call_expression; }
                else { vec.push(Cow::from(call_expression)); }
            },
            "unary_expression" => {
                let unary_expression = handle_unary_expression(ctx, node, src);
                if !reached_header_end { temp += &unary_expression; }
                else { vec.push(Cow::from(unary_expression)); }
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(ctx, node, src);
                if !reached_header_end { temp += &parenthesized_expression; }
                else { vec.push(Cow::from(parenthesized_expression)); }
            },
            "field_expression" => {
                let field_expression = handle_field_expression(ctx, node, src);
                if !reached_header_end { temp += &field_expression; }
                else { vec.push(Cow::from(field_expression)); }
            },
            "subscript_expression" => {
                let subscript_expression = handle_subscript_expression(ctx, node, src);
                if !reached_header_end { temp += &subscript_expression; }
                else { vec.push(Cow::from(subscript_expression)); }
            },
            "identifier" => {
                let identifier = handle_identifier(ctx, node, src);
                if !reached_header_end { temp += &identifier; }
                else { vec.push(Cow::from(identifier)); }
            },
            "qualified_identifier" => {
                let qualified_identifier = lang_node(ctx, node, src);
                if !reached_header_end { temp += &qualified_identifier; }
                else { vec.push(Cow::from(qualified_identifier)); }
            },
            "number_literal" => {
                let number_literal = handle_number_literal(node, src);
                if !reached_header_end { temp += &number_literal; }
                else { vec.push(Cow::from(number_literal)); }
            },
            "pointer_expression" => {
                let mut pointer_expression = handle_pointer_expression(ctx, node, src);
                if !L::IS_CPP {
                    pointer_expression = utils::remove_dereference_spaces(&pointer_expression);
                    temp += &pointer_expression;
                } else if !reached_header_end {
                    temp += &pointer_expression;
                } else {
                    vec.push(Cow::from(pointer_expression));
                }
            },
            "comma_expression" => {
                let comma_expression = handle_comma_expression(ctx, node, src);
                if !L::IS_CPP || !reached_header_end {
                    temp += &comma_expression;
                } else {
                    vec.push(Cow::from(comma_expression));
                }
            },
            "declaration" => {
                let declaration = handle_declaration(ctx, node, src);
                if !reached_header_end { temp += format!("{} ", declaration).as_str(); }
                else { vec.push(Cow::from(declaration)); }
            },
            "expression_statement" => {
                let expression_statement = handle_expression_statement(ctx, node, src);
                vec.push(Cow::from(expression_statement));
            },
            "true" => temp += "true",
            "false" => temp += "false",
            ";" => {
                if !reached_header_end { temp += "; "; }
                else { vec.push(Cow::Borrowed(";")); }
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
            _ => println!("You shouldn't be here (cpp for_statement): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let result = vec.join(" ");
    return result;
}

pub fn handle_struct_specifier<L: AstLang>(ctx: &L, root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = handle_identifier(ctx, node, src);
                result += format!(" {}", identifier).as_str();
            },
            "qualified_identifier" => {
                let qualified_identifier = lang_node(ctx, node, src);
                result += format!(" {}", qualified_identifier).as_str();
            },
            "template_type" => {
                let template_type = lang_node(ctx, node, src);
                result += format!(" {}", template_type).as_str();
            },
            "virtual_specifier" => {
                let virtual_specifier = lang_node(ctx, node, src);
                result += format!(" {}", virtual_specifier).as_str();
            },
            "base_class_clause" => {
                let base_class_clause = lang_node(ctx, node, src);
                result += format!(" {}", base_class_clause).as_str();
            },
            "attribute_specifier" | "attribute_declaration" => {
                let attribute = handle_attribute(node, src);
                result += format!(" {}", attribute).as_str();
            },
            "field_declaration_list" => {
                let field_declaration_list = handle_field_declaration_list(ctx, node, src);
                if result.contains("\n") {
                    result += &field_declaration_list;
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

pub fn handle_type_definition<L: AstLang>(ctx: &L, root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = handle_identifier(ctx, node, src);
                result += format!(" {}", identifier).as_str();
            },
            "qualified_identifier" => {
                let qualified_identifier = lang_node(ctx, node, src);
                result += format!(" {}", qualified_identifier).as_str();
            },
            "template_type" => {
                let template_type = lang_node(ctx, node, src);
                result += format!(" {}", template_type).as_str();
            },
            "typedef" => {
                if L::IS_CPP { result += "typedef"; } else { result += "typedef "; }
            },
            "primitive_type" => {
                let primitive_type = handle_primitive_type(node, src);
                if L::IS_CPP { result += format!(" {}", primitive_type).as_str(); } else { result += primitive_type; }
            },
            "type_qualifier" => {
                let type_qualifier = handle_type_qualifier(ctx, node, src);
                if L::IS_CPP { result += format!(" {}", type_qualifier).as_str(); } else { result += format!("{} ", type_qualifier).as_str(); }
            },
            "sized_type_specifier" => {
                let sized_type_specifier = handle_sized_type_specifier(ctx, node, src);
                if L::IS_CPP { result += format!(" {}", sized_type_specifier).as_str(); } else { result += &sized_type_specifier; }
            },
            "function_declarator" => {
                let function_declarator = handle_function_declarator(ctx, node, src);
                result += format!(" {}", function_declarator).as_str();
            },
            "parenthesized_declarator" => {
                let parenthesized_declarator = handle_parenthesized_declarator(ctx, node, src);
                result += format!(" {}", parenthesized_declarator).as_str();
            },
            "array_declarator" => {
                let array_declarator = handle_array_declarator(ctx, node, src);
                result += format!(" {}", array_declarator).as_str();
            },
            "struct_specifier" => {
                let struct_specifier = handle_struct_specifier(ctx, node, src);
                if L::IS_CPP { result += format!(" {}", struct_specifier).as_str(); } else { result += &struct_specifier; }
            },
            "union_specifier" => {
                let union_specifier = handle_union_specifier(ctx, node, src);
                if L::IS_CPP { result += format!(" {}", union_specifier).as_str(); } else { result += &union_specifier; }
            },
            "class_specifier" => {
                let class_specifier = lang_node(ctx, node, src);
                result += format!(" {}", class_specifier).as_str();
            },
            "pointer_declarator" => {
                let pointer_declarator = handle_pointer_declarator(ctx, node, src);
                result = result.trim_end().to_string();
                result += &pointer_declarator;
            },
            "pointer_type_declarator" => {
                let pointer_type_declarator = lang_node(ctx, node, src);
                result = result.trim_end().to_string();
                result += &pointer_type_declarator;
            },
            "enum_specifier" => {
                let enum_specifier = handle_enum_specifier(ctx, node, src);
                if !L::IS_CPP {
                    result = result.trim_end().to_string();
                }
                result += format!(" {}", enum_specifier.trim_end()).as_str();
            },
            "dependent_type" => {
                let dependent_type = lang_node(ctx, node, src);
                result += format!(" {}", dependent_type).as_str();
            },
            "reference_declarator" => {
                let reference_declarator = lang_node(ctx, node, src);
                result = result.trim_end().to_string();
                result += format!(" {}", reference_declarator).as_str();
            },
            ";" => result += ";",
            "," => result += ", ",
            _ => println!("You shouldn't be here (cpp type_definition): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

pub fn handle_field_declaration_list<L: AstLang>(ctx: &L, root: Node, src: &str) -> String {
    let mut parts = Vec::<Cow<'_, str>>::new();
    let mut pending_incomplete_decl = false;
    let mut skip_until_byte: usize = 0;
    for node in root.children(&mut root.walk()) {
        if node.start_byte() < skip_until_byte {
            continue;
        }
        match node.grammar_name() {
            "field_declaration" => {
                let missing_semi = node_has_missing_semicolon(node);
                let mut field_declaration = handle_field_declaration(ctx, node, src);
                if !L::IS_CPP && field_declaration.contains("*") {
                    field_declaration = utils::remove_pointer_spaces(&field_declaration);
                }
                if pending_incomplete_decl {
                    utils::merge_incomplete_decl_into_parts(&mut parts, &field_declaration);
                } else {
                    parts.push(Cow::from(utils::add_all_leading_tabs(&field_declaration)));
                }
                pending_incomplete_decl = missing_semi;
            },
            "declaration" => {
                let missing_semi = node_has_missing_semicolon(node);
                let declaration = handle_declaration(ctx, node, src);
                if pending_incomplete_decl {
                    utils::merge_incomplete_decl_into_parts(&mut parts, &declaration);
                } else {
                    parts.push(Cow::from(utils::add_all_leading_tabs(&declaration)));
                }
                pending_incomplete_decl = missing_semi;
            },
            "ERROR" => {
                let error = handle_error(ctx, node, src);
                if pending_incomplete_decl {
                    utils::merge_incomplete_decl_into_parts(&mut parts, &error);
                    if error.trim_end().ends_with(';') {
                        pending_incomplete_decl = false;
                    }
                } else {
                    parts.push(Cow::from(utils::add_all_leading_tabs(&error)));
                }
            },
            "template_declaration" => {
                pending_incomplete_decl = false;
                let mut template_declaration = lang_node(ctx, node, src);
                template_declaration = utils::add_all_leading_tabs(&template_declaration);
                parts.push(Cow::from(template_declaration));
            },
            "friend_declaration" => {
                pending_incomplete_decl = false;
                let friend_declaration = lang_node(ctx, node, src);
                parts.push(Cow::from(format!("\t{}", friend_declaration)));
            },
            "alias_declaration" => {
                pending_incomplete_decl = false;
                let alias_declaration = lang_node(ctx, node, src);
                parts.push(Cow::from(format!("\t{}", alias_declaration)));
            },
            "using_declaration" => {
                pending_incomplete_decl = false;
                let using_declaration = lang_node(ctx, node, src);
                parts.push(Cow::from(format!("\t{}", using_declaration)));
            },
            "static_assert_declaration" => {
                pending_incomplete_decl = false;
                let static_assert_declaration = lang_node(ctx, node, src);
                parts.push(Cow::from(format!("\t{}", static_assert_declaration)));
            },
            "struct_specifier" => {
                pending_incomplete_decl = false;
                let mut struct_specifier = handle_struct_specifier(ctx, node, src);
                struct_specifier = format!("{};", struct_specifier.trim_end());
                struct_specifier = utils::add_all_leading_tabs(&struct_specifier);
                parts.push(Cow::from(struct_specifier));
            },
            "union_specifier" => {
                pending_incomplete_decl = false;
                let mut union_specifier = handle_union_specifier(ctx, node, src);
                union_specifier = format!("{};", union_specifier.trim_end());
                union_specifier = utils::add_all_leading_tabs(&union_specifier);
                parts.push(Cow::from(union_specifier));
            },
            "enum_specifier" => {
                pending_incomplete_decl = false;
                let mut enum_specifier = handle_enum_specifier(ctx, node, src);
                enum_specifier = format!("{};", enum_specifier.trim_end());
                enum_specifier = utils::add_all_leading_tabs(&enum_specifier);
                parts.push(Cow::from(enum_specifier));
            },
            "class_specifier" => {
                pending_incomplete_decl = false;
                let mut class_specifier = lang_node(ctx, node, src);
                class_specifier = format!("{};", class_specifier.trim_end());
                class_specifier = utils::add_all_leading_tabs(&class_specifier);
                parts.push(Cow::from(class_specifier));
            },
            "comment" => {
                let comment = handle_comment(ctx, node, src);
                utils::push_comment_to_parts(&mut parts, &comment, node.start_byte(), src, true);
            },
            "access_specifier" => {
                pending_incomplete_decl = false;
                let access_specifier = lang_node(ctx, node, src);
                if parts.len() > 1 {
                    parts.push(Cow::from(format!("\n{}", access_specifier)));
                } else {
                    parts.push(Cow::from(access_specifier));
                }
            },
            "type_definition" => {
                pending_incomplete_decl = false;
                let type_definition = handle_type_definition(ctx, node, src);
                parts.push(Cow::from(format!("\t{}", type_definition)));
            },
            "preproc_ifdef" => {
                pending_incomplete_decl = false;
                let preproc_ifdef = handle_preproc_ifdef(ctx, node, src);
                parts.push(Cow::from(preproc_ifdef));
            },
            "preproc_if" => {
                pending_incomplete_decl = false;
                let preproc_if = handle_preproc_if(ctx, node, src);
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
                let preproc_call = handle_preproc_call(ctx, node, src);
                parts.push(Cow::from(preproc_call));
            },
            "function_definition" => {
                pending_incomplete_decl = false;
                let mut function_definition = handle_function_definition(ctx, node, src);
                function_definition = utils::add_all_leading_tabs(&function_definition);
                if function_definition.trim_end().ends_with(';') {
                    parts.push(Cow::from(function_definition));
                } else {
                    parts.push(Cow::from(format!("{}\n", function_definition)));
                }
            },
            "{" => {
                pending_incomplete_decl = false;
                parts.push(Cow::Borrowed("{"));
            },
            "}" => {
                pending_incomplete_decl = false;
                if L::IS_CPP {
                    if let Some(last) = parts.last_mut() {
                        if last.ends_with('\n') {
                            last.to_mut().pop();
                        }
                    }
                }
                parts.push(Cow::Borrowed("}"));
            },
            ":" => (),
            ";" => {
                if pending_incomplete_decl && !node.is_missing() {
                    if let Some(last) = parts.last_mut() {
                        last.to_mut().push(';');
                    }
                    pending_incomplete_decl = false;
                }
            },
            _ => println!("You shouldn't be here (cpp field_declaration_list): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let result = parts.join("\n");
    return result;
}

pub fn handle_field_declaration<L: AstLang>(ctx: &L, root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = handle_identifier(ctx, node, src);
                result += format!("{} ", identifier).as_str();
            },
            "qualified_identifier" => {
                let qualified_identifier = lang_node(ctx, node, src);
                result += format!("{} ", qualified_identifier).as_str();
            },
            "primitive_type" => {
                let primitive_type = handle_primitive_type(node, src);
                result += format!("{} ", primitive_type).as_str();
            },
            "sized_type_specifier" => {
                let sized_type_specifier = handle_sized_type_specifier(ctx, node, src);
                result += format!("{} ", sized_type_specifier).as_str();
            },
            "template_type" => {
                let template_type = lang_node(ctx, node, src);
                result += format!("{} ", template_type).as_str();
            },
            "type_qualifier" => {
                let type_qualifier = handle_type_qualifier(ctx, node, src);
                result += format!("{} ", type_qualifier).as_str();
            },
            "placeholder_type_specifier" => {
                let placeholder_type_specifier = lang_node(ctx, node, src);
                result += format!("{} ", placeholder_type_specifier.trim()).as_str();
            },
            "decltype" => {
                let decltype = lang_node(ctx, node, src);
                result += format!("{} ", decltype).as_str();
            },
            "explicit_function_specifier" => {
                let explicit_function_specifier = lang_node(ctx, node, src);
                result += format!("{} ", explicit_function_specifier).as_str();
            },
            "array_declarator" => {
                let array_declarator = handle_array_declarator(ctx, node, src);
                result += format!("{} ", array_declarator).as_str();
            },
            "reference_declarator" => {
                let reference_declarator = lang_node(ctx, node, src);
                result += format!("{} ", reference_declarator).as_str();
            },
            "pointer_declarator" => {
                let pointer_declarator = handle_pointer_declarator(ctx, node, src);
                result += format!("{} ", pointer_declarator).as_str();
            },
            "parenthesized_declarator" => {
                let parenthesized_declarator = handle_parenthesized_declarator(ctx, node, src);
                result += format!("{} ", parenthesized_declarator).as_str();
            },
            "compound_literal_expression" => {
                let compound_literal_expression = handle_compound_literal_expression(ctx, node, src);
                result += format!("{} ", compound_literal_expression).as_str();
            },
            "initializer_list" => {
                let initializer_list = handle_initializer_list(ctx, node, src);
                result = result.trim_end().to_string();
                if result.ends_with('=') {
                    result += format!(" {} ", initializer_list).as_str();
                } else {
                    result += format!("{} ", initializer_list).as_str();
                }
            },
            "function_declarator" => {
                let function_declarator = handle_function_declarator(ctx, node, src);
                result += format!("{} ", function_declarator).as_str();
            },
            "optional_parameter_declaration" => {
                let optional_parameter_declaration = lang_node(ctx, node, src);
                result += format!("{} ", optional_parameter_declaration).as_str();
            },
            "default_method_clause" => {
                let default_method_clause = lang_node(ctx, node, src);
                result = result.trim_end().to_string();
                result += &default_method_clause;
            },
            "delete_method_clause" => {
                let delete_method_clause = lang_node(ctx, node, src);
                result = result.trim_end().to_string();
                result += &delete_method_clause;
            },
            "bitfield_clause" => {
                let bitfield_clause = handle_bitfield_clause(ctx, node, src);
                result += format!("{} ", bitfield_clause).as_str();
            },
            "storage_class_specifier" => {
                let storage_class_specifier = handle_storage_class_specifier(ctx, node, src);
                result += format!("{} ", storage_class_specifier).as_str();
            },
            "struct_specifier" => {
                let struct_specifier = handle_struct_specifier(ctx, node, src);
                result += format!("{} ", struct_specifier).as_str();
            },
            "union_specifier" => {
                let union_specifier = handle_union_specifier(ctx, node, src);
                result += format!("{} ", union_specifier).as_str();
            },
            "class_specifier" => {
                let class_specifier = lang_node(ctx, node, src);
                result += format!("{} ", class_specifier).as_str();
            },
            "enum_specifier" => {
                let enum_specifier = handle_enum_specifier(ctx, node, src);
                result += format!("{} ", enum_specifier).as_str();
            },
            "number_literal" => {
                let number_literal = handle_number_literal(node, src);
                result += format!("{} ", number_literal).as_str();
            },
            "char_literal" => {
                let char_literal = handle_char_literal(node, src);
                result += format!("{} ", char_literal).as_str();
            },
            "string_literal" => {
                let string_literal = handle_string_literal(ctx, node, src);
                result += format!("{} ", string_literal).as_str();
            },
            "pointer_expression" => {
                let pointer_expression = handle_pointer_expression(ctx, node, src);
                result += format!("{} ", pointer_expression).as_str();
            },
            "call_expression" => {
                let call_expression = handle_call_expression(ctx, node, src);
                result += format!("{} ", call_expression).as_str();
            },
            "cast_expression" => {
                let cast_expression = handle_cast_expression(ctx, node, src);
                result += format!("{} ", cast_expression).as_str();
            },
            "binary_expression" => {
                let binary_expression = handle_binary_expression(ctx, node, src);
                result += format!("{} ", binary_expression).as_str();
            },
            "unary_expression" => {
                let unary_expression = handle_unary_expression(ctx, node, src);
                result += format!("{} ", unary_expression).as_str();
            },
            "conditional_expression" => {
                let conditional_expression = handle_conditional_expression(ctx, node, src);
                result += format!("{} ", conditional_expression).as_str();
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(ctx, node, src);
                result += format!("{} ", parenthesized_expression).as_str();
            },
            "sizeof_expression" => {
                let sizeof_expression = handle_sizeof_expression(ctx, node, src);
                result += format!("{} ", sizeof_expression).as_str();
            },
            "subscript_expression" => {
                let subscript_expression = handle_subscript_expression(ctx, node, src);
                result += format!("{} ", subscript_expression).as_str();
            },
            "field_expression" => {
                let field_expression = handle_field_expression(ctx, node, src);
                result += format!("{} ", field_expression).as_str();
            },
            "new_expression" => {
                let new_expression = lang_node(ctx, node, src);
                result += format!("{} ", new_expression).as_str();
            },
            "lambda_expression" => {
                let lambda_expression = lang_node(ctx, node, src);
                result += format!("{} ", lambda_expression).as_str();
            },
            "concatenated_string" => {
                let concatenated_string = handle_concatenated_string(ctx, node, src);
                result += format!("{} ", concatenated_string).as_str();
            },
            "raw_string_literal" => {
                let raw_string_literal = handle_raw_string_literal(node, src);
                result += format!("{} ", raw_string_literal).as_str();
            },
            "update_expression" => {
                let update_expression = handle_update_expression(node, src);
                result += format!("{} ", update_expression).as_str();
            },
            "dependent_type" => {
                let dependent_type = lang_node(ctx, node, src);
                result += format!("{} ", dependent_type).as_str();
            },
            "pure_virtual_clause" => {
                let pure_virtual_clause = lang_node(ctx, node, src);
                result = result.trim_end().to_string();
                result += &pure_virtual_clause;
            },
            "attribute_specifier" | "attribute_declaration" => {
                let attribute = handle_attribute(node, src);
                result += format!("{} ", attribute).as_str();
            },
            "comment" => {
                let comment = handle_comment(ctx, node, src);
                utils::append_expr_comment(&mut result, &comment, node.start_byte(), node.end_byte(), src);
            },
            "ERROR" => {
                let error = handle_error(ctx, node, src);
                result += format!("{} ", error).as_str();
            },
            "virtual" => result += "virtual ",
            "inline" => result += "inline ",
            "constexpr" => result += "constexpr ",
            "true" => result += "true",
            "false" => result += "false",
            "null" => result += ctx.handle_null(node, src),
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
    if L::IS_CPP {
        if result.contains("*") { result = utils::remove_pointer_spaces(&result); }
        if result.contains("&") { result = utils::remove_reference_spaces(&result); }
    }
    return result;
}

pub fn handle_abstract_array_declarator<L: AstLang>(ctx: &L, root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "number_literal" => {
                let number_literal = handle_number_literal(node, src);
                result += &number_literal;
            },
            "identifier" => {
                let identifier = handle_identifier(ctx, node, src);
                result += &identifier;
            },
            "qualified_identifier" => {
                let qualified_identifier = lang_node(ctx, node, src);
                result += &qualified_identifier;
            },
            "binary_expression" => {
                let binary_expression = handle_binary_expression(ctx, node, src);
                result += &binary_expression;
            },
            "subscript_expression" => {
                let subscript_expression = handle_subscript_expression(ctx, node, src);
                result += &subscript_expression;
            },
            "field_expression" => {
                let field_expression = handle_field_expression(ctx, node, src);
                result += &field_expression;
            },
            "call_expression" => {
                let call_expression = handle_call_expression(ctx, node, src);
                result += &call_expression;
            },
            "sizeof_expression" => {
                let sizeof_expression = handle_sizeof_expression(ctx, node, src);
                result += &sizeof_expression;
            },
            "cast_expression" => {
                let cast_expression = handle_cast_expression(ctx, node, src);
                result += &cast_expression;
            },
            "unary_expression" => {
                let unary_expression = handle_unary_expression(ctx, node, src);
                result += &unary_expression;
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(ctx, node, src);
                result += &parenthesized_expression;
            },
            "abstract_parenthesized_declarator" => {
                let abstract_parenthesized_declarator = handle_abstract_parenthesized_declarator(ctx, node, src);
                result += &abstract_parenthesized_declarator;
            },
            "abstract_array_declarator" => {
                let abstract_array_declarator = handle_abstract_array_declarator(ctx, node, src);
                result += &abstract_array_declarator;
            },
            "abstract_pointer_declarator" => {
                let abstract_pointer_declarator = handle_abstract_pointer_declarator(ctx, node, src);
                result += &abstract_pointer_declarator;
            },
            "*" => result += "*",
            "[" => result += "[",
            "]" => result += "]",
            _ => println!("You shouldn't be here (cpp abstract_array_declarator): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

pub fn handle_sized_type_specifier<L: AstLang>(ctx: &L, root: Node, src: &str) -> String {
    let mut parts = Vec::<Cow<'_, str>>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "primitive_type" => {
                let primitive_type = handle_primitive_type(node, src);
                parts.push(Cow::from(primitive_type));
            },
            "identifier" => {
                let identifier = handle_identifier(ctx, node, src);
                parts.push(Cow::from(identifier));
            },
            "long" => parts.push(Cow::Borrowed("long")),
            "short" => parts.push(Cow::Borrowed("short")),
            "unsigned" => parts.push(Cow::Borrowed("unsigned")),
            "signed" => parts.push(Cow::Borrowed("signed")),
            _ => println!("You shouldn't be here (cpp sized_type_specifier): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let result = parts.join(" ");
    return result;
}

pub fn handle_enum_specifier<L: AstLang>(ctx: &L, root: Node, src: &str) -> String {
    let single_line = root.start_position().row == root.end_position().row;
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = handle_identifier(ctx, node, src);
                result += format!(" {}", identifier).as_str();
            },
            "qualified_identifier" => {
                let qualified_identifier = lang_node(ctx, node, src);
                result += format!(" {}", qualified_identifier).as_str();
            },
            "primitive_type" => {
                let primitive_type = handle_primitive_type(node, src);
                result += format!(" {}", primitive_type).as_str();
            },
            "sized_type_specifier" => {
                let sized_type_specifier = handle_sized_type_specifier(ctx, node, src);
                result += format!(" {}", sized_type_specifier).as_str();
            },
            ":" => result += " :",
            "enumerator_list" => {
                let enumerator_list = handle_enumerator_list(ctx, node, src);
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
                let field_declaration_list = handle_field_declaration_list(ctx, node, src);
                if result.contains("\n") {
                    result += &field_declaration_list;
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

pub fn handle_union_specifier<L: AstLang>(ctx: &L, root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = handle_identifier(ctx, node, src);
                result += format!(" {}", identifier).as_str();
            },
            "qualified_identifier" => {
                let qualified_identifier = lang_node(ctx, node, src);
                result += format!(" {}", qualified_identifier).as_str();
            },
            "field_declaration_list" => {
                let field_declaration_list = handle_field_declaration_list(ctx, node, src);
                if result.contains("\n") {
                    result += &field_declaration_list;
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

pub fn handle_enumerator_list<L: AstLang>(ctx: &L, root: Node, src: &str) -> String {
    let mut parts = Vec::<Cow<'_, str>>::new();
    let mut temp = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "enumerator" => {
                let enumerator = handle_enumerator(ctx, node, src);
                temp += format!("\t{}", enumerator).as_str();
            },
            "comment" => {
                let comment = handle_comment(ctx, node, src);
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
                let preproc_if = handle_preproc_if(ctx, node, src);
                parts.push(Cow::from(preproc_if));
            },
            "preproc_ifdef" => {
                if !temp.is_empty() {
                    parts.push(Cow::from(temp));
                    temp = String::new();
                }
                let preproc_ifdef = handle_preproc_ifdef(ctx, node, src);
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
            "preproc_call" => {
                if !temp.is_empty() {
                    parts.push(Cow::from(temp));
                    temp = String::new();
                }
                let preproc_call = handle_preproc_call(ctx, node, src);
                parts.push(Cow::from(preproc_call));
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
            _ => println!("You shouldn't be here (cpp enumerator_list): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let result = parts.join("\n");
    return result;
}

pub fn handle_enumerator<L: AstLang>(ctx: &L, root: Node, src: &str) -> String {
    let mut parts = Vec::<Cow<'_, str>>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => parts.push(Cow::from(handle_identifier(ctx, node, src).to_string())),
            "=" => parts.push(Cow::Borrowed("=")),
            "number_literal" => parts.push(Cow::from(handle_number_literal(node, src).to_string())),
            "char_literal" => parts.push(Cow::from(handle_char_literal(node, src).to_string())),
            "binary_expression" => parts.push(Cow::from(handle_binary_expression(ctx, node, src))),
            "unary_expression" => parts.push(Cow::from(handle_unary_expression(ctx, node, src))),
            "parenthesized_expression" => parts.push(Cow::from(handle_parenthesized_expression(ctx, node, src))),
            "cast_expression" => parts.push(Cow::from(handle_cast_expression(ctx, node, src))),
            "call_expression" => parts.push(Cow::from(handle_call_expression(ctx, node, src))),
            "sizeof_expression" => parts.push(Cow::from(handle_sizeof_expression(ctx, node, src))),
            "conditional_expression" => parts.push(Cow::from(handle_conditional_expression(ctx, node, src))),
            "qualified_identifier" => parts.push(Cow::from(lang_node(ctx, node, src))),
            _ => parts.push(Cow::from(node.utf8_text(src.as_bytes()).unwrap_or("").to_string())),
        }
    }
    return parts.join(" ");
}

pub fn handle_storage_class_specifier<'a, L: AstLang>(_ctx: &L, root: Node, src: &'a str) -> Cow<'a, str> {
    if !L::IS_CPP {
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
            return Cow::Borrowed(root.utf8_text(src.as_bytes()).unwrap_or(""));
        }
        return Cow::Borrowed(result);
    }

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
    Cow::Owned(result.trim_end().to_string())
}

pub fn handle_preproc_ifdef<L: AstLang>(ctx: &L, root: Node, src: &str) -> String {
    let mut parts = Vec::<Cow<'_, str>>::new();
    let mut temp = String::new();
    let mut skip_until_byte: usize = 0;
    for node in root.children(&mut root.walk()) {
        if node.start_byte() < skip_until_byte {
            continue;
        }
        match node.grammar_name() {
            "#ifndef" => {
                temp += "#ifndef ";
            },
            "#ifdef" => {
                temp += "#ifdef ";
            },
            "#endif" => {
                parts.push(Cow::Borrowed("#endif"));
            },
            "identifier" => {
                let identifier = handle_identifier(ctx, node, src);
                temp += &identifier;
                parts.push(Cow::from(temp));
                temp = String::new();
            },
            "preproc_def" => {
                let (preproc_def, end_b) = utils::consume_define_node(node, src);
                skip_until_byte = end_b;
                parts.push(Cow::from(preproc_def));
            },
            "preproc_call" => {
                let preproc_call = handle_preproc_call(ctx, node, src);
                parts.push(Cow::from(preproc_call));
            },
            "field_declaration" => {
                let field_declaration = handle_field_declaration(ctx, node, src);
                parts.push(Cow::from(format!("\t{}", field_declaration)));
            },
            "declaration" => {
                let declaration = handle_declaration(ctx, node, src);
                parts.push(Cow::from(declaration));
            },
            "type_definition" => {
                let type_definition = handle_type_definition(ctx, node, src);
                parts.push(Cow::from(type_definition));
            },
            "alias_declaration" => {
                let alias_declaration = lang_node(ctx, node, src);
                parts.push(Cow::from(alias_declaration));
            },
            "using_declaration" => {
                let using_declaration = lang_node(ctx, node, src);
                parts.push(Cow::from(using_declaration));
            },
            "template_declaration" => {
                let template_declaration = lang_node(ctx, node, src);
                parts.push(Cow::from(template_declaration));
            },
            "template_instantiation" => {
                let template_instantiation = lang_node(ctx, node, src);
                parts.push(Cow::from(template_instantiation));
            },
            "linkage_specification" => {
                let linkage_specification = lang_node(ctx, node, src);
                parts.push(Cow::from(linkage_specification));
            },
            "namespace_definition" => {
                let namespace_definition = lang_node(ctx, node, src);
                parts.push(Cow::from(namespace_definition));
            },
            "namespace_alias_definition" => {
                let namespace_alias_definition = lang_node(ctx, node, src);
                parts.push(Cow::from(namespace_alias_definition));
            },
            "static_assert_declaration" => {
                let static_assert_declaration = lang_node(ctx, node, src);
                parts.push(Cow::from(static_assert_declaration));
            },
            "class_specifier" => {
                let class_specifier = lang_node(ctx, node, src);
                parts.push(Cow::from(format!("{};", class_specifier.trim_end())));
            },
            "struct_specifier" => {
                let struct_specifier = handle_struct_specifier(ctx, node, src);
                parts.push(Cow::from(format!("{};", struct_specifier.trim_end())));
            },
            "union_specifier" => {
                let union_specifier = handle_union_specifier(ctx, node, src);
                parts.push(Cow::from(format!("{};", union_specifier.trim_end())));
            },
            "enum_specifier" => {
                let enum_specifier = handle_enum_specifier(ctx, node, src);
                parts.push(Cow::from(format!("{};", enum_specifier.trim_end())));
            },
            "if_statement" => {
                let if_statement = handle_if_statement(ctx, node, src);
                parts.push(Cow::from(if_statement));
            },
            "for_statement" => {
                let for_statement = handle_for_statement(ctx, node, src);
                parts.push(Cow::from(for_statement));
            },
            "for_range_loop" => {
                let for_range_loop = lang_node(ctx, node, src);
                parts.push(Cow::from(for_range_loop));
            },
            "while_statement" => {
                let while_statement = handle_while_statement(ctx, node, src);
                parts.push(Cow::from(while_statement));
            },
            "do_statement" => {
                let do_statement = handle_do_statement(ctx, node, src);
                parts.push(Cow::from(do_statement));
            },
            "switch_statement" => {
                let switch_statement = handle_switch_statement(ctx, node, src);
                parts.push(Cow::from(switch_statement));
            },
            "break_statement" => parts.push(Cow::Borrowed("break;")),
            "continue_statement" => parts.push(Cow::Borrowed("continue;")),
            "goto_statement" => {
                let goto_statement = handle_goto_statement(ctx, node, src);
                parts.push(Cow::from(goto_statement));
            },
            "compound_statement" => {
                let compound_statement = handle_compound_statement(ctx, node, src);
                parts.push(Cow::from(compound_statement));
            },
            "access_specifier" => {
                let access_specifier = lang_node(ctx, node, src);
                if parts.len() > 1 {
                    parts.push(Cow::from(format!("\n{}", access_specifier)));
                } else {
                    parts.push(Cow::from(access_specifier));
                }
            },
            "expression_statement" => {
                let expression_statement = handle_expression_statement(ctx, node, src);
                parts.push(Cow::from(expression_statement));
            },
            "preproc_include" => {
                let preproc_include = handle_preproc_include(node, src);
                parts.push(Cow::from(preproc_include));
            },
            "preproc_if" => {
                let preproc_if = handle_preproc_if(ctx, node, src);
                parts.push(Cow::from(preproc_if));
            },
            "preproc_ifdef" => {
                let preproc_ifdef = handle_preproc_ifdef(ctx, node, src);
                parts.push(Cow::from(preproc_ifdef));
            },
            "preproc_elif" => {
                let preproc_elif = handle_preproc_elif(ctx, node, src);
                parts.push(Cow::from(preproc_elif));
            },
            "preproc_else" => {
                let preproc_else = handle_preproc_else(ctx, node, src);
                parts.push(Cow::from(preproc_else));
            },
            "preproc_function_def" => {
                let (preproc_function_def, end_b) = utils::consume_define_node(node, src);
                skip_until_byte = end_b;
                parts.push(Cow::from(preproc_function_def));
            },
            "function_definition" => {
                let function_definition = handle_function_definition(ctx, node, src);
                parts.push(Cow::from(function_definition));
            },
            "comment" => {
                let comment = handle_comment(ctx, node, src);
                if !temp.is_empty() {
                    temp += format!(" {}", comment).as_str();
                    parts.push(Cow::from(temp));
                    temp = String::new();
                } else {
                    utils::push_comment_to_parts(&mut parts, &comment, node.start_byte(), src, false);
                }
            },
            "return_statement" => {
                let return_statement = handle_return_statement(ctx, node, src);
                parts.push(Cow::from(return_statement));
            },
            "enumerator" => {
                let enumerator = handle_enumerator(ctx, node, src);
                parts.push(Cow::from(format!("\t{},", enumerator)));
            },
            "case_statement" => {
                let mut case_statement = handle_case_statement(ctx, node, src);
                case_statement = utils::remove_blank_lines(case_statement.split("\n").collect::<Vec<&str>>());
                parts.push(Cow::from(case_statement));
            },
            "attributed_statement" => {
                let attributed_statement = handle_attributed_statement(ctx, node, src);
                parts.push(Cow::from(attributed_statement));
            },
            "labeled_statement" => {
                let labeled_statement = handle_labeled_statement(ctx, node, src);
                parts.push(Cow::from(labeled_statement));
            },
            "ERROR" => {
                let error = handle_error(ctx, node, src);
                parts.push(Cow::from(error));
            },
            "," => (),
            ";" => (),
            ":" => (),
            "::" => {
                if !node.is_missing() {
                    parts.push(Cow::Borrowed("::"));
                }
            },
            "\n" => (),
            "}" => parts.push(Cow::Borrowed("}")),
            _ => println!("You shouldn't be here (cpp preproc_ifdef): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let result = parts.join("\n");
    return result;
}

pub fn handle_preproc_if<L: AstLang>(ctx: &L, root: Node, src: &str) -> String {
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
                let binary_expression = handle_binary_expression(ctx, node, src);
                if !temp.is_empty() {
                    temp += &binary_expression;
                    parts.push(Cow::from(temp));
                    temp = String::new();
                } else {
                    parts.push(Cow::from(binary_expression));
                }
            },
            "unary_expression" => {
                let unary_expression = handle_unary_expression(ctx, node, src);
                if !temp.is_empty() {
                    temp += &unary_expression;
                    parts.push(Cow::from(temp));
                    temp = String::new();
                } else {
                    parts.push(Cow::from(unary_expression));
                }
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(ctx, node, src);
                if !temp.is_empty() {
                    temp += &parenthesized_expression;
                    parts.push(Cow::from(temp));
                    temp = String::new();
                } else {
                    parts.push(Cow::from(parenthesized_expression));
                }
            },
            "return_statement" => {
                let return_statement = handle_return_statement(ctx, node, src);
                parts.push(Cow::from(return_statement));
            },
            "preproc_if" => {
                let preproc_if = handle_preproc_if(ctx, node, src);
                parts.push(Cow::from(preproc_if));
            },
            "preproc_ifdef" => {
                let preproc_ifdef = handle_preproc_ifdef(ctx, node, src);
                parts.push(Cow::from(preproc_ifdef));
            },
            "declaration" => {
                let declaration = handle_declaration(ctx, node, src);
                parts.push(Cow::from(declaration));
            },
            "field_declaration" => {
                let field_declaration = handle_field_declaration(ctx, node, src);
                parts.push(Cow::from(format!("\t{}", field_declaration)));
            },
            "type_definition" => {
                let type_definition = handle_type_definition(ctx, node, src);
                parts.push(Cow::from(type_definition));
            },
            "alias_declaration" => {
                let alias_declaration = lang_node(ctx, node, src);
                parts.push(Cow::from(alias_declaration));
            },
            "using_declaration" => {
                let using_declaration = lang_node(ctx, node, src);
                parts.push(Cow::from(using_declaration));
            },
            "template_declaration" => {
                let template_declaration = lang_node(ctx, node, src);
                parts.push(Cow::from(template_declaration));
            },
            "template_instantiation" => {
                let template_instantiation = lang_node(ctx, node, src);
                parts.push(Cow::from(template_instantiation));
            },
            "linkage_specification" => {
                let linkage_specification = lang_node(ctx, node, src);
                parts.push(Cow::from(linkage_specification));
            },
            "namespace_definition" => {
                let namespace_definition = lang_node(ctx, node, src);
                parts.push(Cow::from(namespace_definition));
            },
            "static_assert_declaration" => {
                let static_assert_declaration = lang_node(ctx, node, src);
                parts.push(Cow::from(static_assert_declaration));
            },
            "class_specifier" => {
                let class_specifier = lang_node(ctx, node, src);
                parts.push(Cow::from(format!("{};", class_specifier.trim_end())));
            },
            "struct_specifier" => {
                let struct_specifier = handle_struct_specifier(ctx, node, src);
                parts.push(Cow::from(format!("{};", struct_specifier.trim_end())));
            },
            "union_specifier" => {
                let union_specifier = handle_union_specifier(ctx, node, src);
                parts.push(Cow::from(format!("{};", union_specifier.trim_end())));
            },
            "enum_specifier" => {
                let enum_specifier = handle_enum_specifier(ctx, node, src);
                parts.push(Cow::from(format!("{};", enum_specifier.trim_end())));
            },
            "if_statement" => {
                let if_statement = handle_if_statement(ctx, node, src);
                parts.push(Cow::from(if_statement));
            },
            "for_statement" => {
                let for_statement = handle_for_statement(ctx, node, src);
                parts.push(Cow::from(for_statement));
            },
            "for_range_loop" => {
                let for_range_loop = lang_node(ctx, node, src);
                parts.push(Cow::from(for_range_loop));
            },
            "while_statement" => {
                let while_statement = handle_while_statement(ctx, node, src);
                parts.push(Cow::from(while_statement));
            },
            "do_statement" => {
                let do_statement = handle_do_statement(ctx, node, src);
                parts.push(Cow::from(do_statement));
            },
            "switch_statement" => {
                let switch_statement = handle_switch_statement(ctx, node, src);
                parts.push(Cow::from(switch_statement));
            },
            "break_statement" => parts.push(Cow::Borrowed("break;")),
            "continue_statement" => parts.push(Cow::Borrowed("continue;")),
            "goto_statement" => {
                let goto_statement = handle_goto_statement(ctx, node, src);
                parts.push(Cow::from(goto_statement));
            },
            "compound_statement" => {
                let compound_statement = handle_compound_statement(ctx, node, src);
                parts.push(Cow::from(compound_statement));
            },
            "access_specifier" => {
                let access_specifier = lang_node(ctx, node, src);
                if parts.len() > 1 {
                    parts.push(Cow::from(format!("\n{}", access_specifier)));
                } else {
                    parts.push(Cow::from(access_specifier));
                }
            },
            "function_definition" => {
                let function_definition = handle_function_definition(ctx, node, src);
                parts.push(Cow::from(function_definition));
            },
            "preproc_else" => {
                let preproc_else = handle_preproc_else(ctx, node, src);
                parts.push(Cow::from(preproc_else));
            },
            "preproc_call" => {
                let preproc_call = handle_preproc_call(ctx, node, src);
                parts.push(Cow::from(preproc_call));
            },
            "call_expression" => {
                let call_expression = handle_call_expression(ctx, node, src);
                if !temp.is_empty() {
                    temp += &call_expression;
                    parts.push(Cow::from(temp));
                    temp = String::new();
                } else {
                    parts.push(Cow::from(call_expression));
                }
            },
            "comment" => {
                let comment = handle_comment(ctx, node, src);
                if !temp.is_empty() {
                    temp += format!(" {}", comment).as_str();
                    parts.push(Cow::from(temp));
                    temp = String::new();
                } else {
                    utils::push_comment_to_parts(&mut parts, &comment, node.start_byte(), src, false);
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
            "expression_statement" => {
                let expression_statement = handle_expression_statement(ctx, node, src);
                parts.push(Cow::from(expression_statement));
            },
            "namespace_alias_definition" => {
                let namespace_alias_definition = lang_node(ctx, node, src);
                parts.push(Cow::from(namespace_alias_definition));
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
            "preproc_elif" => {
                let preproc_elif = handle_preproc_elif(ctx, node, src);
                parts.push(Cow::from(preproc_elif));
            },
            "preproc_defined" => {
                let preproc_defined = handle_preproc_defined(ctx, node, src);
                if !temp.is_empty() {
                    temp += &preproc_defined;
                    parts.push(Cow::from(temp));
                    temp = String::new();
                } else {
                    parts.push(Cow::from(preproc_defined));
                }
            },
            "preproc_function_def" => {
                if !temp.is_empty() {
                    parts.push(Cow::from(std::mem::take(&mut temp)));
                }
                let (preproc_function_def, end_b) = utils::consume_define_node(node, src);
                skip_until_byte = end_b;
                parts.push(Cow::from(preproc_function_def));
            },
            "identifier" => {
                let identifier = handle_identifier(ctx, node, src);
                if !temp.is_empty() {
                    temp += &identifier;
                    parts.push(Cow::from(temp));
                    temp = String::new();
                } else {
                    parts.push(Cow::from(identifier));
                }
            },
            "enumerator" => {
                let enumerator = handle_enumerator(ctx, node, src);
                parts.push(Cow::from(format!("\t{},", enumerator)));
            },
            "case_statement" => {
                let mut case_statement = handle_case_statement(ctx, node, src);
                case_statement = utils::remove_blank_lines(case_statement.split("\n").collect::<Vec<&str>>());
                parts.push(Cow::from(case_statement));
            },
            "attributed_statement" => {
                let attributed_statement = handle_attributed_statement(ctx, node, src);
                parts.push(Cow::from(attributed_statement));
            },
            "labeled_statement" => {
                let labeled_statement = handle_labeled_statement(ctx, node, src);
                parts.push(Cow::from(labeled_statement));
            },
            "ERROR" => {
                let error = handle_error(ctx, node, src);
                parts.push(Cow::from(error));
            },
            "," => (),
            "\n" => (),
            ";" => (),
            ":" => (),
            "::" => {
                if !node.is_missing() {
                    parts.push(Cow::Borrowed("::"));
                }
            },
            "}" => parts.push(Cow::Borrowed("}")),
            _ => println!("You shouldn't be here (cpp preproc_if): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let result = parts.join("\n");
    return result;
}

pub fn handle_preproc_else<L: AstLang>(ctx: &L, root: Node, src: &str) -> String {
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
                let return_statement = handle_return_statement(ctx, node, src);
                parts.push(Cow::from(return_statement));
            },
            "declaration" => {
                let declaration = handle_declaration(ctx, node, src);
                parts.push(Cow::from(declaration));
            },
            "field_declaration" => {
                let field_declaration = handle_field_declaration(ctx, node, src);
                parts.push(Cow::from(format!("\t{}", field_declaration)));
            },
            "type_definition" => {
                let type_definition = handle_type_definition(ctx, node, src);
                parts.push(Cow::from(type_definition));
            },
            "alias_declaration" => {
                let alias_declaration = lang_node(ctx, node, src);
                parts.push(Cow::from(alias_declaration));
            },
            "using_declaration" => {
                let using_declaration = lang_node(ctx, node, src);
                parts.push(Cow::from(using_declaration));
            },
            "expression_statement" => {
                let expression_statement = handle_expression_statement(ctx, node, src);
                parts.push(Cow::from(expression_statement));
            },
            "template_declaration" => {
                let template_declaration = lang_node(ctx, node, src);
                parts.push(Cow::from(template_declaration));
            },
            "template_instantiation" => {
                let template_instantiation = lang_node(ctx, node, src);
                parts.push(Cow::from(template_instantiation));
            },
            "linkage_specification" => {
                let linkage_specification = lang_node(ctx, node, src);
                parts.push(Cow::from(linkage_specification));
            },
            "namespace_definition" => {
                let namespace_definition = lang_node(ctx, node, src);
                parts.push(Cow::from(namespace_definition));
            },
            "static_assert_declaration" => {
                let static_assert_declaration = lang_node(ctx, node, src);
                parts.push(Cow::from(static_assert_declaration));
            },
            "class_specifier" => {
                let class_specifier = lang_node(ctx, node, src);
                parts.push(Cow::from(format!("{};", class_specifier.trim_end())));
            },
            "struct_specifier" => {
                let struct_specifier = handle_struct_specifier(ctx, node, src);
                parts.push(Cow::from(format!("{};", struct_specifier.trim_end())));
            },
            "union_specifier" => {
                let union_specifier = handle_union_specifier(ctx, node, src);
                parts.push(Cow::from(format!("{};", union_specifier.trim_end())));
            },
            "enum_specifier" => {
                let enum_specifier = handle_enum_specifier(ctx, node, src);
                parts.push(Cow::from(format!("{};", enum_specifier.trim_end())));
            },
            "if_statement" => {
                let if_statement = handle_if_statement(ctx, node, src);
                parts.push(Cow::from(if_statement));
            },
            "for_statement" => {
                let for_statement = handle_for_statement(ctx, node, src);
                parts.push(Cow::from(for_statement));
            },
            "for_range_loop" => {
                let for_range_loop = lang_node(ctx, node, src);
                parts.push(Cow::from(for_range_loop));
            },
            "while_statement" => {
                let while_statement = handle_while_statement(ctx, node, src);
                parts.push(Cow::from(while_statement));
            },
            "do_statement" => {
                let do_statement = handle_do_statement(ctx, node, src);
                parts.push(Cow::from(do_statement));
            },
            "switch_statement" => {
                let switch_statement = handle_switch_statement(ctx, node, src);
                parts.push(Cow::from(switch_statement));
            },
            "break_statement" => parts.push(Cow::Borrowed("break;")),
            "continue_statement" => parts.push(Cow::Borrowed("continue;")),
            "goto_statement" => {
                let goto_statement = handle_goto_statement(ctx, node, src);
                parts.push(Cow::from(goto_statement));
            },
            "compound_statement" => {
                let compound_statement = handle_compound_statement(ctx, node, src);
                parts.push(Cow::from(compound_statement));
            },
            "access_specifier" => {
                let access_specifier = lang_node(ctx, node, src);
                if parts.len() > 1 {
                    parts.push(Cow::from(format!("\n{}", access_specifier)));
                } else {
                    parts.push(Cow::from(access_specifier));
                }
            },
            "preproc_if" => {
                let preproc_if = handle_preproc_if(ctx, node, src);
                parts.push(Cow::from(preproc_if));
            },
            "preproc_call" => {
                let preproc_call = handle_preproc_call(ctx, node, src);
                parts.push(Cow::from(preproc_call));
            },
            "binary_expression" => {
                let binary_expression = handle_binary_expression(ctx, node, src);
                parts.push(Cow::from(binary_expression));
            },
            "preproc_def" => {
                let (preproc_def, end_b) = utils::consume_define_node(node, src);
                skip_until_byte = end_b;
                parts.push(Cow::from(preproc_def));
            },
            "preproc_ifdef" => {
                let preproc_ifdef = handle_preproc_ifdef(ctx, node, src);
                parts.push(Cow::from(preproc_ifdef));
            },
            "preproc_elif" => {
                let preproc_elif = handle_preproc_elif(ctx, node, src);
                parts.push(Cow::from(preproc_elif));
            },
            "preproc_include" => {
                let preproc_include = handle_preproc_include(node, src);
                parts.push(Cow::from(preproc_include));
            },
            "comment" => {
                let comment = handle_comment(ctx, node, src);
                utils::push_comment_to_parts(&mut parts, &comment, node.start_byte(), src, false);
            },
            "function_definition" => {
                let function_definition = handle_function_definition(ctx, node, src);
                parts.push(Cow::from(function_definition));
            },
            "preproc_defined" => {
                let preproc_defined = handle_preproc_defined(ctx, node, src);
                parts.push(Cow::from(preproc_defined));
            },
            "preproc_function_def" => {
                let (preproc_function_def, end_b) = utils::consume_define_node(node, src);
                skip_until_byte = end_b;
                parts.push(Cow::from(preproc_function_def));
            },
            "enumerator" => {
                let enumerator = handle_enumerator(ctx, node, src);
                parts.push(Cow::from(format!("\t{},", enumerator)));
            },
            "case_statement" => {
                let mut case_statement = handle_case_statement(ctx, node, src);
                case_statement = utils::remove_blank_lines(case_statement.split("\n").collect::<Vec<&str>>());
                parts.push(Cow::from(case_statement));
            },
            "attributed_statement" => {
                let attributed_statement = handle_attributed_statement(ctx, node, src);
                parts.push(Cow::from(attributed_statement));
            },
            "labeled_statement" => {
                let labeled_statement = handle_labeled_statement(ctx, node, src);
                parts.push(Cow::from(labeled_statement));
            },
            "ERROR" => {
                let error = handle_error(ctx, node, src);
                parts.push(Cow::from(error));
            },
            "," => (),
            ";" => (),
            ":" => (),
            "::" => {
                if !node.is_missing() {
                    parts.push(Cow::Borrowed("::"));
                }
            },
            "\n" => (),
            "}" => parts.push(Cow::Borrowed("}")),
            _ => println!("You shouldn't be here (cpp preproc_else): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let result = parts.join("\n");
    return result;
}

pub fn handle_variadic_parameter<L: AstLang>(ctx: &L, root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "..." => result += "...",
            _ => println!("You shouldn't be here (cpp variadic_parameter): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

pub fn handle_comma_expression<L: AstLang>(ctx: &L, root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = handle_identifier(ctx, node, src);
                result += &identifier;
            },
            "number_literal" => {
                let number_literal = handle_number_literal(node, src);
                result += &number_literal;
            },
            "assignment_expression" => {
                let assignment_expression = handle_assignment_expression(ctx, node, src);
                result += &assignment_expression;
            },
            "call_expression" => {
                let call_expression = handle_call_expression(ctx, node, src);
                result += &call_expression;
            },
            "binary_expression" => {
                let binary_expression = handle_binary_expression(ctx, node, src);
                result += &binary_expression;
            },
            "unary_expression" => {
                let unary_expression = handle_unary_expression(ctx, node, src);
                result += &unary_expression;
            },
            "pointer_expression" => {
                let pointer_expression = handle_pointer_expression(ctx, node, src);
                result += &pointer_expression;
            },
            "comma_expression" => {
                let comma_expression = handle_comma_expression(ctx, node, src);
                result += &comma_expression;
            },
            "update_expression" => {
                let update_expression = handle_update_expression(node, src);
                result += &update_expression;
            },
            "compound_literal_expression" => {
                let compound_literal_expression = handle_compound_literal_expression(ctx, node, src);
                if L::IS_CPP && result.rsplit('\n').next().unwrap_or("").trim_start().starts_with('#') {
                    if let Some((head, tail)) = compound_literal_expression.split_once("){") {
                        result += head;
                        result += ")\n\t{";
                        result += tail;
                    } else {
                        result += &compound_literal_expression;
                    }
                } else {
                    result += &compound_literal_expression;
                }
            },
            "ERROR" => {
                let error = handle_error(ctx, node, src);
                result += &error;
            },
            "," => result += ", ",
            _ => println!("You shouldn't be here (cpp comma_expression): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

pub fn handle_preproc_params<L: AstLang>(ctx: &L, root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = handle_identifier(ctx, node, src);
                result += &identifier;
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

pub fn handle_preproc_call<L: AstLang>(ctx: &L, root: Node, src: &str) -> String {
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
                parts.push(Cow::from(handle_comment(ctx, node, src)));
            },
            _ => println!("You shouldn't be here (cpp preproc_call): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return parts.join(" ");
}

pub fn handle_do_statement<L: AstLang>(ctx: &L, root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "do" => result += "do ",
            "while" => result += " while ",
            ";" => result += ";",
            "compound_statement" => {
                let compound_statement = handle_compound_statement(ctx, node, src);
                result += &compound_statement;
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(ctx, node, src);
                result += &parenthesized_expression;
            },
            "expression_statement" => {
                let expression_statement = handle_expression_statement(ctx, node, src);
                result += &expression_statement;
            },
            _ => println!("You shouldn't be here (cpp do_statement): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

pub fn handle_inner_field_expression<L: AstLang>(ctx: &L, root: Node, src: &str) -> String {
    return handle_field_expression(ctx, root, src);
}

pub fn handle_parameter_list<L: AstLang>(ctx: &L, root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "(" => result += "(",
            ")" => result += ")",
            "()" => result += "()",
            "," => result += ", ",
            "..." => {
                if result.ends_with(' ') || result.ends_with('(') {
                    result += "...";
                } else {
                    result += " ...";
                }
            },
            "parameter_declaration" => {
                let mut parameter_declaration = handle_parameter_declaration(ctx, node, src);
                parameter_declaration = utils::remove_pointer_spaces(&parameter_declaration);
                if L::IS_CPP {
                    parameter_declaration = utils::remove_reference_spaces(&parameter_declaration);
                }
                result += &parameter_declaration;
            },
            "optional_parameter_declaration" => {
                let mut optional_parameter_declaration = lang_node(ctx, node, src);
                optional_parameter_declaration = utils::remove_pointer_spaces(&optional_parameter_declaration);
                optional_parameter_declaration = utils::remove_reference_spaces(&optional_parameter_declaration);
                result += &optional_parameter_declaration;
            },
            "variadic_parameter_declaration" => {
                let mut variadic_parameter_declaration = lang_node(ctx, node, src);
                variadic_parameter_declaration = utils::remove_pointer_spaces(&variadic_parameter_declaration);
                variadic_parameter_declaration = utils::remove_reference_spaces(&variadic_parameter_declaration);
                result += &variadic_parameter_declaration;
            },
            "variadic_parameter" => {
                let variadic_parameter = handle_variadic_parameter(ctx, node, src);
                result += &variadic_parameter;
            },
            "comment" => {
                let comment = handle_comment(ctx, node, src);
                utils::append_expr_comment(&mut result, &comment, node.start_byte(), node.end_byte(), src);
            },
            _ => println!("You shouldn't be here (cpp parameter_list): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

pub fn handle_initializer_pair<L: AstLang>(ctx: &L, root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "field_designator" => result += handle_field_designator(ctx, node, src).as_str(),
            "subscript_designator" => result += handle_subscript_designator(ctx, node, src).as_str(),
            "identifier" => result += handle_identifier(ctx, node, src),
            "qualified_identifier" => result += lang_node(ctx, node, src).as_str(),
            "number_literal" => result += handle_number_literal(node, src),
            "char_literal" => result += handle_char_literal(node, src),
            "string_literal" => result += handle_string_literal(ctx, node, src).as_str(),
            "concatenated_string" => result += handle_concatenated_string(ctx, node, src).as_str(),
            "binary_expression" => result += handle_binary_expression(ctx, node, src).as_str(),
            "unary_expression" => result += handle_unary_expression(ctx, node, src).as_str(),
            "cast_expression" => result += handle_cast_expression(ctx, node, src).as_str(),
            "call_expression" => result += handle_call_expression(ctx, node, src).as_str(),
            "field_expression" => result += handle_field_expression(ctx, node, src).as_str(),
            "subscript_expression" => result += handle_subscript_expression(ctx, node, src).as_str(),
            "pointer_expression" => result += handle_pointer_expression(ctx, node, src).as_str(),
            "conditional_expression" => result += handle_conditional_expression(ctx, node, src).as_str(),
            "parenthesized_expression" => result += handle_parenthesized_expression(ctx, node, src).as_str(),
            "sizeof_expression" => result += handle_sizeof_expression(ctx, node, src).as_str(),
            "initializer_list" => result += handle_initializer_list(ctx, node, src).as_str(),
            "compound_literal_expression" => result += handle_compound_literal_expression(ctx, node, src).as_str(),
            "raw_string_literal" => result += handle_raw_string_literal(node, src),
            "lambda_expression" => result += lang_node(ctx, node, src).as_str(),
            "template_function" => result += lang_node(ctx, node, src).as_str(),
            "new_expression" => result += lang_node(ctx, node, src).as_str(),
            "ERROR" => result += handle_error(ctx, node, src).as_str(),
            "null" => result += ctx.handle_null(node, src),
            "this" => result += "this",
            "true" => result += "true",
            "false" => result += "false",
            "=" => result += " = ",
            ":" => result += ": ",
            _ => println!("You shouldn't be here (cpp initializer_pair): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

pub fn handle_field_designator<L: AstLang>(ctx: &L, root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "." => result += ".",
            "identifier" => result += handle_identifier(ctx, node, src),
            _ => println!("You shouldn't be here (cpp field_designator): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

pub fn handle_subscript_designator<L: AstLang>(ctx: &L, root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "[" => result += "[",
            "]" => result += "]",
            "number_literal" => result += handle_number_literal(node, src),
            "identifier" => result += handle_identifier(ctx, node, src),
            "binary_expression" => result += handle_binary_expression(ctx, node, src).as_str(),
            "char_literal" if !L::IS_CPP => result += handle_char_literal(node, src),
            _ => println!("You shouldn't be here (cpp subscript_designator): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

pub fn handle_compound_literal_expression<L: AstLang>(ctx: &L, root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "(" => result += "(",
            ")" => result += ")",
            "type_descriptor" => result += handle_type_descriptor(ctx, node, src).as_str(),
            "identifier" => {
                let identifier = handle_identifier(ctx, node, src);
                result += &identifier;
            },
            "qualified_identifier" => {
                let qualified_identifier = lang_node(ctx, node, src);
                result += &qualified_identifier;
            },
            "primitive_type" => {
                let primitive_type = handle_primitive_type(node, src);
                result += &primitive_type;
            },
            "initializer_list" => {
                let initializer_list = handle_initializer_list(ctx, node, src);
                result += &initializer_list;
            },
            "template_type" => {
                let template_type = lang_node(ctx, node, src);
                result += &template_type;
            },
            "ERROR" => {
                let error = handle_error(ctx, node, src);
                result += &error;
            },
            _ => println!("You shouldn't be here (cpp compound_literal_expression): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

pub fn handle_offsetof_expression<L: AstLang>(ctx: &L, root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "offsetof" => result += "offsetof",
            "(" => result += "(",
            ")" => result += ")",
            "," => result += ", ",
            "type_descriptor" => result += handle_type_descriptor(ctx, node, src).as_str(),
            "identifier" => result += handle_identifier(ctx, node, src),
            "field_expression" => result += handle_field_expression(ctx, node, src).as_str(),
            _ => println!("You shouldn't be here (cpp offsetof_expression): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

pub fn handle_bitfield_clause<L: AstLang>(ctx: &L, root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "call_expression" => {
                let call_expression = handle_call_expression(ctx, node, src);
                result += &call_expression;
            },
            "number_literal" => {
                let number_literal = handle_number_literal(node, src);
                result += &number_literal;
            },
            "identifier" => {
                let identifier = handle_identifier(ctx, node, src);
                result += &identifier;
            },
            "qualified_identifier" => {
                let qualified_identifier = lang_node(ctx, node, src);
                result += &qualified_identifier;
            },
            "binary_expression" => {
                let binary_expression = handle_binary_expression(ctx, node, src);
                result += &binary_expression;
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(ctx, node, src);
                result += &parenthesized_expression;
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

pub fn handle_abstract_function_declarator<L: AstLang>(ctx: &L, root: Node, src: &str) -> String {
    let mut parts = Vec::<Cow<'_, str>>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "parameter_list" => {
                let parameter_list = handle_parameter_list(ctx, node, src);
                parts.push(Cow::from(parameter_list));
            },
            "type_qualifier" => {
                let type_qualifier = handle_type_qualifier(ctx, node, src);
                parts.push(Cow::from(type_qualifier));
            },
            "trailing_return_type" => {
                let trailing_return_type = lang_node(ctx, node, src);
                parts.push(Cow::from(trailing_return_type));
            },
            "abstract_parenthesized_declarator" => {
                let abstract_parenthesized_declarator = handle_abstract_parenthesized_declarator(ctx, node, src);
                parts.push(Cow::from(abstract_parenthesized_declarator));
            },
            "noexcept" => parts.push(Cow::Borrowed("noexcept")),
            "mutable" => parts.push(Cow::Borrowed("mutable")),
            "constexpr" => parts.push(Cow::Borrowed("constexpr")),
            "attribute_specifier" | "attribute_declaration" | "ms_call_modifier" => {
                parts.push(Cow::Borrowed(node.utf8_text(src.as_bytes()).unwrap_or("").trim()));
            },
            "comment" => {
                let comment = handle_comment(ctx, node, src);
                utils::push_expr_comment(&mut parts, &comment, node.start_byte(), node.end_byte(), src);
            },
            "ERROR" => {
                let error = handle_error(ctx, node, src);
                parts.push(Cow::from(error));
            },
            _ => println!("You shouldn't be here (cpp abstract_function_declarator): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    if L::IS_CPP {
        parts.join(" ")
    } else {
        parts.join("")
    }
}

pub fn handle_abstract_parenthesized_declarator<L: AstLang>(ctx: &L, root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "abstract_reference_declarator" => {
                let abstract_reference_declarator = lang_node(ctx, node, src);
                result += &abstract_reference_declarator;
            },
            "abstract_pointer_declarator" => {
                let abstract_pointer_declarator = handle_abstract_pointer_declarator(ctx, node, src);
                result += &abstract_pointer_declarator;
            },
            "abstract_function_declarator" => {
                let abstract_function_declarator = handle_abstract_function_declarator(ctx, node, src);
                result += &abstract_function_declarator;
            },
            "abstract_array_declarator" if !L::IS_CPP => {
                let abstract_array_declarator = handle_abstract_array_declarator(ctx, node, src);
                result += &abstract_array_declarator;
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

pub fn handle_preproc_defined<L: AstLang>(ctx: &L, root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "defined" => result += "defined",
            "(" => result += "(",
            ")" => result += ")",
            "identifier" => {
                let identifier = handle_identifier(ctx, node, src);
                result += format!(" {}", identifier).as_str();
            },
            _ => println!("You shouldn't be here (cpp preproc_defined): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    result = result.replace("( ", "(");
    return result;
}

pub fn handle_preproc_elif<L: AstLang>(ctx: &L, root: Node, src: &str) -> String {
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
                let binary_expression = handle_binary_expression(ctx, node, src);
                if !temp.is_empty() {
                    temp += &binary_expression;
                    parts.push(Cow::from(temp));
                    temp = String::new();
                } else {
                    parts.push(Cow::from(binary_expression));
                }
            },
            "unary_expression" => {
                let unary_expression = handle_unary_expression(ctx, node, src);
                if !temp.is_empty() {
                    temp += &unary_expression;
                    parts.push(Cow::from(temp));
                    temp = String::new();
                } else {
                    parts.push(Cow::from(unary_expression));
                }
            },
            "parenthesized_expression" => {
                let parenthesized_expression = handle_parenthesized_expression(ctx, node, src);
                if !temp.is_empty() {
                    temp += &parenthesized_expression;
                    parts.push(Cow::from(temp));
                    temp = String::new();
                } else {
                    parts.push(Cow::from(parenthesized_expression));
                }
            },
            "identifier" => {
                let identifier = handle_identifier(ctx, node, src);
                if !temp.is_empty() {
                    temp += &identifier;
                    parts.push(Cow::from(temp));
                    temp = String::new();
                } else {
                    parts.push(Cow::from(identifier));
                }
            },
            "preproc_defined" => {
                let preproc_defined = handle_preproc_defined(ctx, node, src);
                if !temp.is_empty() {
                    temp += &preproc_defined;
                    parts.push(Cow::from(temp));
                    temp = String::new();
                } else {
                    parts.push(Cow::from(preproc_defined));
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
            "call_expression" => {
                let call_expression = handle_call_expression(ctx, node, src);
                if !temp.is_empty() {
                    temp += &call_expression;
                    parts.push(Cow::from(temp));
                    temp = String::new();
                } else {
                    parts.push(Cow::from(call_expression));
                }
            },
            "preproc_def" => {
                let (preproc_def, end_b) = utils::consume_define_node(node, src);
                skip_until_byte = end_b;
                parts.push(Cow::from(preproc_def));
            },
            "preproc_include" => parts.push(Cow::from(handle_preproc_include(node, src))),
            "preproc_if" => parts.push(Cow::from(handle_preproc_if(ctx, node, src))),
            "preproc_ifdef" => parts.push(Cow::from(handle_preproc_ifdef(ctx, node, src))),
            "preproc_elif" => parts.push(Cow::from(handle_preproc_elif(ctx, node, src))),
            "preproc_function_def" => {
                let (preproc_function_def, end_b) = utils::consume_define_node(node, src);
                skip_until_byte = end_b;
                parts.push(Cow::from(preproc_function_def));
            },
            "preproc_else" => parts.push(Cow::from(handle_preproc_else(ctx, node, src))),
            "preproc_call" => parts.push(Cow::from(handle_preproc_call(ctx, node, src))),
            "expression_statement" => parts.push(Cow::from(handle_expression_statement(ctx, node, src))),
            "declaration" => parts.push(Cow::from(handle_declaration(ctx, node, src))),
            "field_declaration" => parts.push(Cow::from(format!("\t{}", handle_field_declaration(ctx, node, src)))),
            "function_definition" => parts.push(Cow::from(handle_function_definition(ctx, node, src))),
            "type_definition" => parts.push(Cow::from(handle_type_definition(ctx, node, src))),
            "return_statement" => parts.push(Cow::from(handle_return_statement(ctx, node, src))),
            "if_statement" => parts.push(Cow::from(handle_if_statement(ctx, node, src))),
            "for_statement" => parts.push(Cow::from(handle_for_statement(ctx, node, src))),
            "for_range_loop" => parts.push(Cow::from(lang_node(ctx, node, src))),
            "while_statement" => parts.push(Cow::from(handle_while_statement(ctx, node, src))),
            "do_statement" => parts.push(Cow::from(handle_do_statement(ctx, node, src))),
            "switch_statement" => parts.push(Cow::from(handle_switch_statement(ctx, node, src))),
            "case_statement" => {
                let mut case_statement = handle_case_statement(ctx, node, src);
                case_statement = utils::remove_blank_lines(case_statement.split("\n").collect::<Vec<&str>>());
                parts.push(Cow::from(case_statement));
            },
            "compound_statement" => parts.push(Cow::from(handle_compound_statement(ctx, node, src))),
            "break_statement" => parts.push(Cow::Borrowed("break;")),
            "continue_statement" => parts.push(Cow::Borrowed("continue;")),
            "goto_statement" => parts.push(Cow::from(handle_goto_statement(ctx, node, src))),
            "labeled_statement" => parts.push(Cow::from(handle_labeled_statement(ctx, node, src))),
            "attributed_statement" => parts.push(Cow::from(handle_attributed_statement(ctx, node, src))),
            "class_specifier" => parts.push(Cow::from(format!("{};", lang_node(ctx, node, src).trim_end()))),
            "struct_specifier" => parts.push(Cow::from(format!("{};", handle_struct_specifier(ctx, node, src).trim_end()))),
            "union_specifier" => parts.push(Cow::from(format!("{};", handle_union_specifier(ctx, node, src).trim_end()))),
            "enum_specifier" => parts.push(Cow::from(format!("{};", handle_enum_specifier(ctx, node, src).trim_end()))),
            "alias_declaration" => parts.push(Cow::from(lang_node(ctx, node, src))),
            "using_declaration" => parts.push(Cow::from(lang_node(ctx, node, src))),
            "template_declaration" => parts.push(Cow::from(lang_node(ctx, node, src))),
            "template_instantiation" => parts.push(Cow::from(lang_node(ctx, node, src))),
            "linkage_specification" => parts.push(Cow::from(lang_node(ctx, node, src))),
            "namespace_definition" => parts.push(Cow::from(lang_node(ctx, node, src))),
            "namespace_alias_definition" => parts.push(Cow::from(lang_node(ctx, node, src))),
            "static_assert_declaration" => parts.push(Cow::from(lang_node(ctx, node, src))),
            "access_specifier" => {
                let access_specifier = lang_node(ctx, node, src);
                if parts.len() > 1 {
                    parts.push(Cow::from(format!("\n{}", access_specifier)));
                } else {
                    parts.push(Cow::from(access_specifier));
                }
            },
            "enumerator" => parts.push(Cow::from(format!("\t{},", handle_enumerator(ctx, node, src)))),
            "ERROR" => parts.push(Cow::from(handle_error(ctx, node, src))),
            "comment" => {
                let comment = handle_comment(ctx, node, src);
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
            "::" => {
                if !node.is_missing() {
                    parts.push(Cow::Borrowed("::"));
                }
            },
            "}" => parts.push(Cow::Borrowed("}")),
            _ => println!("You shouldn't be here (cpp preproc_elif): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return parts.join("\n");
}

pub fn handle_declaration_list<L: AstLang>(ctx: &L, root: Node, src: &str) -> String {
    let mut parts = Vec::<Cow<'_, str>>::new();
    let mut pending_incomplete_decl = false;
    let mut skip_until_byte: usize = 0;
    for node in root.children(&mut root.walk()) {
        if node.start_byte() < skip_until_byte {
            continue;
        }
        match node.grammar_name() {
            "{" => {
                pending_incomplete_decl = false;
                parts.push(Cow::Borrowed("{"));
            },
            "}" => {
                pending_incomplete_decl = false;
                parts.push(Cow::Borrowed("}"));
            },
            ";" => {
                if L::IS_CPP && pending_incomplete_decl && !node.is_missing() {
                    if let Some(last) = parts.last_mut() {
                        last.to_mut().push(';');
                    }
                    pending_incomplete_decl = false;
                }
            },
            "comment" => {
                let comment = handle_comment(ctx, node, src);
                utils::push_comment_to_parts(&mut parts, &comment, node.start_byte(), src, false);
            },
            "declaration" => {
                if L::IS_CPP {
                    let missing_semi = node_has_missing_semicolon(node);
                    let declaration = handle_declaration(ctx, node, src);
                    if pending_incomplete_decl {
                        utils::merge_incomplete_decl_into_parts(&mut parts, &declaration);
                    } else {
                        parts.push(Cow::from(declaration));
                    }
                    pending_incomplete_decl = missing_semi;
                } else {
                    let mut declaration = handle_declaration(ctx, node, src);
                    if declaration.contains("*") {
                        declaration = utils::remove_pointer_spaces(&declaration);
                    }
                    parts.push(Cow::from(declaration));
                }
            },
            "function_definition" => {
                pending_incomplete_decl = false;
                let function_definition = handle_function_definition(ctx, node, src);
                parts.push(Cow::from(format!("{}\n", function_definition)));
            },
            "struct_specifier" => {
                let struct_specifier = handle_struct_specifier(ctx, node, src);
                parts.push(Cow::from(format!("{};\n", struct_specifier.trim_end())));
            },
            "union_specifier" => {
                let union_specifier = handle_union_specifier(ctx, node, src);
                parts.push(Cow::from(format!("{};\n", union_specifier.trim_end())));
            },
            "enum_specifier" => {
                let enum_specifier = handle_enum_specifier(ctx, node, src);
                parts.push(Cow::from(format!("{};\n", enum_specifier.trim_end())));
            },
            "class_specifier" => {
                let class_specifier = lang_node(ctx, node, src);
                parts.push(Cow::from(format!("{};\n", class_specifier.trim_end())));
            },
            "template_declaration" => {
                let template_declaration = lang_node(ctx, node, src);
                parts.push(Cow::from(format!("{}\n", template_declaration)));
            },
            "template_instantiation" => {
                let template_instantiation = lang_node(ctx, node, src);
                parts.push(Cow::from(template_instantiation));
            },
            "type_definition" => {
                let td = handle_type_definition(ctx, node, src);
                if !L::IS_CPP && td.contains('{') {
                    parts.push(Cow::from(format!("{}\n", td)));
                } else {
                    parts.push(Cow::from(td));
                }
            },
            "alias_declaration" => {
                let alias_declaration = lang_node(ctx, node, src);
                parts.push(Cow::from(alias_declaration));
            },
            "using_declaration" => {
                let using_declaration = lang_node(ctx, node, src);
                parts.push(Cow::from(using_declaration));
            },
            "namespace_definition" => {
                let namespace_definition = lang_node(ctx, node, src);
                parts.push(Cow::from(namespace_definition));
            },
            "namespace_alias_definition" => {
                let namespace_alias_definition = lang_node(ctx, node, src);
                parts.push(Cow::from(namespace_alias_definition));
            },
            "linkage_specification" => {
                let linkage_specification = lang_node(ctx, node, src);
                parts.push(Cow::from(linkage_specification));
            },
            "static_assert_declaration" => {
                let static_assert_declaration = lang_node(ctx, node, src);
                parts.push(Cow::from(static_assert_declaration));
            },
            "expression_statement" => {
                let expression_statement = handle_expression_statement(ctx, node, src);
                parts.push(Cow::from(expression_statement));
            },
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
            "preproc_call" => parts.push(Cow::from(handle_preproc_call(ctx, node, src))),
            "preproc_ifdef" => {
                let preproc_ifdef = handle_preproc_ifdef(ctx, node, src);
                parts.push(Cow::from(preproc_ifdef));
            },
            "preproc_if" => {
                let preproc_if = handle_preproc_if(ctx, node, src);
                parts.push(Cow::from(preproc_if));
            },
            "preproc_elif" => {
                let preproc_elif = handle_preproc_elif(ctx, node, src);
                parts.push(Cow::from(preproc_elif));
            },
            "preproc_else" => {
                let preproc_else = handle_preproc_else(ctx, node, src);
                parts.push(Cow::from(preproc_else));
            },
            "friend_declaration" => {
                let friend_declaration = lang_node(ctx, node, src);
                parts.push(Cow::from(friend_declaration));
            },
            "if_statement" => {
                let if_statement = handle_if_statement(ctx, node, src);
                parts.push(Cow::from(if_statement));
            },
            "for_statement" => {
                let for_statement = handle_for_statement(ctx, node, src);
                parts.push(Cow::from(for_statement));
            },
            "for_range_loop" => {
                let for_range_loop = lang_node(ctx, node, src);
                parts.push(Cow::from(for_range_loop));
            },
            "while_statement" => {
                let while_statement = handle_while_statement(ctx, node, src);
                parts.push(Cow::from(while_statement));
            },
            "do_statement" => {
                let do_statement = handle_do_statement(ctx, node, src);
                parts.push(Cow::from(do_statement));
            },
            "switch_statement" => {
                let switch_statement = handle_switch_statement(ctx, node, src);
                parts.push(Cow::from(switch_statement));
            },
            "return_statement" => {
                let return_statement = handle_return_statement(ctx, node, src);
                parts.push(Cow::from(return_statement));
            },
            "attributed_statement" => {
                let attributed_statement = handle_attributed_statement(ctx, node, src);
                parts.push(Cow::from(attributed_statement));
            },
            "ERROR" => {
                let error = handle_error(ctx, node, src);
                parts.push(Cow::from(error));
            },
            _ => println!("You shouldn't be here (cpp declaration_list): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let result = parts.join("\n");
    return result;
}

pub fn handle_attributed_statement<L: AstLang>(ctx: &L, root: Node, src: &str) -> String {
    let mut attrs = Vec::<Cow<'_, str>>::new();
    let mut stmt = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "attribute_declaration" | "attribute_specifier" => {
                attrs.push(Cow::Borrowed(handle_attribute(node, src)));
            },
            "expression_statement" => stmt = handle_expression_statement(ctx, node, src),
            "return_statement" => stmt = handle_return_statement(ctx, node, src),
            "if_statement" => stmt = handle_if_statement(ctx, node, src),
            "switch_statement" => stmt = handle_switch_statement(ctx, node, src),
            "for_statement" => stmt = handle_for_statement(ctx, node, src),
            "for_range_loop" => stmt = lang_node(ctx, node, src),
            "while_statement" => stmt = handle_while_statement(ctx, node, src),
            "do_statement" => stmt = handle_do_statement(ctx, node, src),
            "compound_statement" => stmt = handle_inner_compound_statement(ctx, node, src),
            "break_statement" => stmt = "break;".to_string(),
            "continue_statement" => stmt = "continue;".to_string(),
            "goto_statement" => stmt = handle_goto_statement(ctx, node, src),
            "case_statement" => stmt = handle_case_statement(ctx, node, src),
            "labeled_statement" => stmt = handle_labeled_statement(ctx, node, src),
            "attributed_statement" => stmt = handle_attributed_statement(ctx, node, src),
            "throw_statement" => stmt = lang_node(ctx, node, src),
            "try_statement" => stmt = lang_node(ctx, node, src),
            ";" => {
                if !stmt.ends_with(';') {
                    stmt += ";";
                }
            },
            "comment" => {
                let comment = handle_comment(ctx, node, src);
                utils::append_expr_comment(&mut stmt, &comment, node.start_byte(), node.end_byte(), src);
            },
            _ => println!("You shouldn't be here (cpp attributed_statement): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
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
