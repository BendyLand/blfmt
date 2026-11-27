use std::borrow::Cow;
use tree_sitter::{Node, Tree};
use crate::{ast, c_format, utils};

pub struct CAst;

const CTX: CAst = CAst;

impl ast::AstLang for CAst {
    const IS_CPP: bool = false;
    const LANG: &'static str = "c";

    fn parse_wrapped_block(&self, wrapped: &str) -> Tree {
        c_format::parse_existing_c_file(wrapped)
    }

    fn handle_null<'a>(&self, _node: Node, _src: &'a str) -> &'a str {
        "NULL"
    }

    fn dispatch_node<'a>(&self, node: Node, src: &'a str) -> Option<Cow<'a, str>> {
        match node.grammar_name() {
            "macro_type_specifier" => Some(Cow::Owned(handle_macro_type_specifier(node, src))),
            _ => None,
        }
    }
}

pub fn traverse_c_ast(ast_tree: &Tree, src: &str, style: utils::Style) -> String {
    let root = ast_tree.root_node();
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
                let preproc_include = ast::handle_preproc_include(child, src);
                result += format!("{}\n", preproc_include).as_str();
                last_group_kind = "preproc_include";
            },
            "declaration" => {
                let missing_semi = ast::node_has_missing_semicolon(child);
                let mut declaration = ast::handle_declaration(&CTX, child, src);
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
                let function_definition = ast::handle_function_definition(&CTX, child, src);
                result += format!("{}\n\n", function_definition).as_str();
                last_group_kind = "function_definition";
            },
            "expression_statement" => {
                let missing_semi = ast::node_has_missing_semicolon(child);
                let expression_statement = ast::handle_expression_statement(&CTX, child, src);
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
                let compound_statement = ast::handle_compound_statement(&CTX, child, src);
                result += format!("{}\n\n", compound_statement).as_str();
                last_group_kind = "compound_statement";
            },
            "if_statement" => {
                let if_statement = ast::handle_if_statement(&CTX, child, src);
                result += format!("{}\n\n", if_statement).as_str();
                last_group_kind = "if_statement";
            },
            "switch_statement" => {
                let switch_statement = ast::handle_switch_statement(&CTX, child, src);
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
                let return_statement = ast::handle_return_statement(&CTX, child, src);
                result += format!("{}\n\n", return_statement).as_str();
                last_group_kind = "return_statement";
            },
            "comment" => {
                let comment = ast::handle_comment(&CTX, child, src);
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
                let error = ast::handle_error(&CTX, child, src);
                result += format!("{}\n\n", error).as_str();
                last_group_kind = "ERROR";
            },
            "type_definition" => {
                if last_group_kind.contains("preproc") { result += "\n"; }
                let type_definition = ast::handle_type_definition(&CTX, child, src);
                result += format!("{}\n\n", type_definition).as_str();
                last_group_kind = "type_definition";
            },
            "struct_specifier" => {
                let struct_specifier = ast::handle_struct_specifier(&CTX, child, src);
                result += format!("{};\n\n", struct_specifier.trim_end()).as_str();
                last_group_kind = "struct_specifier";
            },
            "union_specifier" => {
                let union_specifier = ast::handle_union_specifier(&CTX, child, src);
                result += format!("{};\n\n", union_specifier.trim_end()).as_str();
                last_group_kind = "union_specifier";
            },
            "enum_specifier" => {
                let enum_specifier = ast::handle_enum_specifier(&CTX, child, src);
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
                let preproc_ifdef = ast::handle_preproc_ifdef(&CTX, child, src);
                result += format!("{}\n\n", preproc_ifdef).as_str();
                last_group_kind = "preproc_ifdef";
            },
            "preproc_if" => {
                let preproc_if = ast::handle_preproc_if(&CTX, child, src);
                result += format!("{}\n\n", preproc_if).as_str();
                last_group_kind = "preproc_if";
            },
            "preproc_elif" => {
                let preproc_elif = ast::handle_preproc_elif(&CTX, child, src);
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
                let preproc_call = ast::handle_preproc_call(&CTX, child, src);
                result += format!("{}\n\n", preproc_call).as_str();
                last_group_kind = "preproc_call";
            },
            "sized_type_specifier" => {
                let sized_type_specifier = ast::handle_sized_type_specifier(&CTX, child, src);
                result += format!("{}\n\n", sized_type_specifier).as_str();
                last_group_kind = "sized_type_specifier";
            },
            "declaration_list" => {
                let declaration_list = ast::handle_declaration_list(&CTX, child, src);
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

fn handle_macro_type_specifier(root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => result += ast::handle_identifier(&CTX, node, src),
            "(" => result += "(",
            ")" => result += ")",
            "type_descriptor" => result += ast::handle_type_descriptor(&CTX, node, src).as_str(),
            _ => println!("You shouldn't be here (c macro_type_specifier): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}
