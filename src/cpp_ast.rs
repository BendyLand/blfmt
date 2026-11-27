use std::borrow::Cow;
use tree_sitter::{Node, Tree};
use crate::{ast, cpp_format, utils};

pub struct CppAst;

const CTX: CppAst = CppAst;

impl ast::AstLang for CppAst {
    const IS_CPP: bool = true;
    const LANG: &'static str = "cpp";

    fn parse_wrapped_block(&self, wrapped: &str) -> Tree {
        cpp_format::parse_existing_cpp_file(wrapped)
    }

    fn handle_null<'a>(&self, node: Node, src: &'a str) -> &'a str {
        handle_null(node, src)
    }

    fn dispatch_node<'a>(&self, node: Node, src: &'a str) -> Option<Cow<'a, str>> {
        match node.grammar_name() {
            "abstract_reference_declarator" => Some(Cow::Owned(handle_abstract_reference_declarator(node, src))),
            "access_specifier" => Some(Cow::Owned(handle_access_specifier(node, src))),
            "alias_declaration" => Some(Cow::Owned(handle_alias_declaration(node, src))),
            "base_class_clause" => Some(Cow::Owned(handle_base_class_clause(node, src))),
            "class_specifier" => Some(Cow::Owned(handle_class_specifier(node, src))),
            "condition_clause" => Some(Cow::Owned(handle_condition_clause(node, src))),
            "decltype" => Some(Cow::Owned(handle_decltype(node, src))),
            "default_method_clause" => Some(Cow::Owned(handle_default_method_clause(node, src))),
            "delete_expression" => Some(Cow::Owned(handle_delete_expression(node, src))),
            "delete_method_clause" => Some(Cow::Owned(handle_delete_method_clause(node, src))),
            "dependent_type" => Some(Cow::Owned(handle_dependent_type(node, src))),
            "destructor_name" => Some(Cow::Owned(handle_destructor_name(node, src))),
            "explicit_function_specifier" => Some(Cow::Owned(handle_explicit_function_specifier(node, src))),
            "field_initializer_list" => Some(Cow::Owned(handle_field_initializer_list(node, src))),
            "for_range_loop" => Some(Cow::Owned(handle_for_range_loop(node, src))),
            "friend_declaration" => Some(Cow::Owned(handle_friend_declaration(node, src))),
            "lambda_expression" => Some(Cow::Owned(handle_lambda_expression(node, src))),
            "linkage_specification" => Some(Cow::Owned(handle_linkage_specification(node, src))),
            "namespace_alias_definition" => Some(Cow::Owned(handle_namespace_alias_definition(node, src))),
            "namespace_definition" => Some(Cow::Owned(handle_namespace_definition(node, src))),
            "new_expression" => Some(Cow::Owned(handle_new_expression(node, src))),
            "operator_cast" => Some(Cow::Owned(handle_operator_cast(node, src))),
            "operator_name" => Some(Cow::Owned(handle_operator_name(node, src))),
            "optional_parameter_declaration" => Some(Cow::Owned(handle_optional_parameter_declaration(node, src))),
            "parameter_pack_expansion" => Some(Cow::Owned(handle_parameter_pack_expansion(node, src))),
            "placeholder_type_specifier" => Some(Cow::Owned(handle_placeholder_type_specifier(node, src))),
            "pointer_type_declarator" => Some(Cow::Owned(handle_pointer_type_declarator(node, src))),
            "pure_virtual_clause" => Some(Cow::Owned(handle_pure_virtual_clause(node, src))),
            "qualified_identifier" => Some(Cow::Owned(handle_qualified_identifier(node, src))),
            "reference_declarator" => Some(Cow::Owned(handle_reference_declarator(node, src))),
            "static_assert_declaration" => Some(Cow::Owned(handle_static_assert_declaration(node, src))),
            "structured_binding_declarator" => Some(Cow::Owned(handle_structured_binding_declarator(node, src))),
            "subscript_argument_list" => Some(Cow::Owned(handle_subscript_argument_list(node, src))),
            "template_declaration" => Some(Cow::Owned(handle_template_declaration(node, src))),
            "template_function" => Some(Cow::Owned(handle_template_function(node, src))),
            "template_instantiation" => Some(Cow::Owned(handle_template_instantiation(node, src))),
            "template_method" => Some(Cow::Owned(handle_template_method(node, src))),
            "template_type" => Some(Cow::Owned(handle_template_type(node, src))),
            "throw_statement" => Some(Cow::Owned(handle_throw_statement(node, src))),
            "trailing_return_type" => Some(Cow::Owned(handle_trailing_return_type(node, src))),
            "try_statement" => Some(Cow::Owned(handle_try_statement(node, src))),
            "using_declaration" => Some(Cow::Owned(handle_using_declaration(node, src))),
            "variadic_parameter_declaration" => Some(Cow::Owned(handle_variadic_parameter_declaration(node, src))),
            "virtual_specifier" => Some(Cow::Borrowed(handle_virtual_specifier(node, src))),
            _ => None,
        }
    }

    fn handle_function_declarator_subnode(
        &self,
        subnode: Node,
        src: &str,
        temp: &mut String,
    ) -> bool {
        match subnode.grammar_name() {
            "~" => *temp += "~",
            "()" => *temp += "()",
            "::" => {
                if subnode.is_missing() {
                    if !temp.is_empty() && !temp.ends_with(' ') {
                        *temp += " ";
                    }
                } else {
                    *temp += "::";
                }
            },
            "<<" => *temp += "operator<<",
            ">>" => *temp += "operator>>",
            "^" => *temp += "operator^",
            "&" => *temp += "operator&",
            "<" => *temp += "operator<",
            "<=" => *temp += "operator<=",
            ">" => *temp += "operator>",
            ">=" => *temp += "operator>=",
            "+" => *temp += "operator+",
            "-" => *temp += "operator-",
            "*" => *temp += "operator*",
            "/" => *temp += "operator/",
            "%" => *temp += "operator%",
            "&&" => *temp += "operator&&",
            "|" => *temp += "operator|",
            "||" => *temp += "operator||",
            "!" => *temp += "operator!",
            "[]" => *temp += "operator[]",
            "[" => *temp += "[",
            "]" => *temp += "]",
            "=" => *temp += "operator=",
            "==" => *temp += "operator==",
            "!=" => *temp += "operator!=",
            "|=" => *temp += "operator|=",
            "&=" => *temp += "operator&=",
            "^=" => *temp += "operator^=",
            "+=" => *temp += "operator+=",
            "-=" => *temp += "operator-=",
            "*=" => *temp += "operator*=",
            "/=" => *temp += "operator/=",
            "%=" => *temp += "operator%=",
            "<<=" => *temp += "operator<<=",
            ">>=" => *temp += "operator>>=",
            "<=>" => *temp += "operator<=>",
            "->" => *temp += "operator->",
            "operator" => *temp += "operator",
            "const" => *temp += " const",
            "override" => *temp += " override",
            "final" => *temp += " final",
            "noexcept" => *temp += " noexcept",
            "reference_declarator" => {
                *temp += &handle_reference_declarator(subnode, src);
            },
            "trailing_return_type" => {
                *temp += format!(" {}", handle_trailing_return_type(subnode, src)).as_str();
            },
            "type_descriptor" => {
                *temp += &ast::handle_type_descriptor(&CTX, subnode, src);
            },
            "argument_list" => {
                *temp += &ast::handle_argument_list(&CTX, subnode, src);
            },
            "__attribute__" => {
                *temp += ast::handle_attribute(subnode, src);
            },
            "pointer_type_declarator" => {
                *temp += &handle_pointer_type_declarator(subnode, src);
            },
            "variadic_parameter_declaration" => {
                let s = handle_variadic_parameter_declaration(subnode, src);
                *temp += &utils::remove_pointer_spaces(&s);
            },
            "optional_parameter_declaration" => {
                let s = handle_optional_parameter_declaration(subnode, src);
                *temp += &utils::remove_pointer_spaces(&s);
            },
            "qualified_identifier" => {
                *temp += &handle_qualified_identifier(subnode, src);
            },
            "template_function" => {
                *temp += &handle_template_function(subnode, src);
            },
            "operator_name" => {
                *temp += &handle_operator_name(subnode, src);
            },
            "destructor_name" => {
                *temp += &handle_destructor_name(subnode, src);
            },
            "template_type" => {
                *temp += &handle_template_type(subnode, src);
            },
            "operator_cast" => {
                *temp += &handle_operator_cast(subnode, src);
            },
            "dependent_name" => {
                *temp += subnode.utf8_text(src.as_bytes()).unwrap_or("");
            },
            "template" => {
                *temp += "template ";
            },
            "attribute_specifier" | "attribute_declaration" | "ms_call_modifier" => {
                *temp += format!("{} ", subnode.utf8_text(src.as_bytes()).unwrap_or("").trim()).as_str();
            },
            "..." => {
                if temp.ends_with(' ') || temp.ends_with('(') {
                    *temp += "...";
                } else {
                    *temp += " ...";
                }
            },
            _ => return false,
        }
        true
    }
}

pub fn traverse_cpp_ast(ast_tree: &Tree, src: &str, style: utils::Style) -> String {
    let root = ast_tree.root_node();
    let mut result = String::new();
    let mut last_group_kind: &str = "";
    let mut pending_incomplete_decl = false;
    let mut skip_until_byte: usize = 0;
    for child in root.children(&mut root.walk()) {
        if child.start_byte() < skip_until_byte {
            continue;
        }
        match child.grammar_name() {
            "preproc_include" => {
                pending_incomplete_decl = false;
                if !last_group_kind.contains("preproc") { result += "\n"; }
                let preproc_include = ast::handle_preproc_include(child, src);
                result += format!("{}\n", preproc_include).as_str();
                last_group_kind = "preproc_include";
            },
            "declaration" => {
                let missing_semi = ast::node_has_missing_semicolon(child);
                let declaration = ast::handle_declaration(&CTX, child, src);
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
                let should_add_space = {
                    last_group_kind.contains("preproc")    ||
                    last_group_kind == "using_declaration" ||
                    last_group_kind == "declaration"
                };
                if should_add_space { result += "\n"; }
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
                    if last_group_kind.contains("preproc") { result += "\n"; }
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
                if last_group_kind.contains("preproc") { result += "\n"; }
                let struct_specifier = ast::handle_struct_specifier(&CTX, child, src);
                result += format!("{};\n\n", struct_specifier.trim_end()).as_str();
                last_group_kind = "struct_specifier";
            },
            "union_specifier" => {
                if last_group_kind.contains("preproc") { result += "\n"; }
                let union_specifier = ast::handle_union_specifier(&CTX, child, src);
                result += format!("{};\n\n", union_specifier.trim_end()).as_str();
                last_group_kind = "union_specifier";
            },
            "preproc_def" => {
                if !last_group_kind.is_empty()
                    && last_group_kind != "comment"
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
                if !last_group_kind.contains("preproc") { result += "\n"; }
                let preproc_ifdef = ast::handle_preproc_ifdef(&CTX, child, src);
                result += format!("{}\n\n", preproc_ifdef).as_str();
                last_group_kind = "preproc_ifdef";
            },
            "preproc_if" => {
                if !last_group_kind.contains("preproc") { result += "\n"; }
                let preproc_if = ast::handle_preproc_if(&CTX, child, src);
                result += format!("{}\n\n", preproc_if).as_str();
                last_group_kind = "preproc_if";
            },
            "preproc_elif" => {
                if !last_group_kind.contains("preproc") { result += "\n"; }
                let preproc_elif = ast::handle_preproc_elif(&CTX, child, src);
                result += format!("{}\n\n", preproc_elif).as_str();
                last_group_kind = "preproc_elif";
            },
            "preproc_function_def" => {
                if !last_group_kind.is_empty()
                    && last_group_kind != "comment"
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
                if !last_group_kind.contains("preproc") || last_group_kind == "preproc_include" {
                    result += "\n";
                }
                let preproc_call = ast::handle_preproc_call(&CTX, child, src);
                result += format!("{}\n\n", preproc_call).as_str();
                last_group_kind = "preproc_call";
            },
            "sized_type_specifier" => {
                let sized_type_specifier = ast::handle_sized_type_specifier(&CTX, child, src);
                result += format!("{}\n\n", sized_type_specifier).as_str();
                last_group_kind = "sized_type_specifier";
            },
            "using_declaration" => {
                let using_declaration = handle_using_declaration(child, src);
                if last_group_kind != "using_declaration" { result += "\n"; }
                result += format!("{}\n", using_declaration).as_str();
                last_group_kind = "using_declaration";
            },
            "enum_specifier" => {
                let enum_specifier = ast::handle_enum_specifier(&CTX, child, src);
                if last_group_kind.contains("preproc") { result += "\n"; }
                result += format!("{};\n\n", enum_specifier.trim_end()).as_str();
                last_group_kind = "enum_specifier";
            },
            "class_specifier" => {
                let class_specifier = handle_class_specifier(child, src);
                if last_group_kind.contains("preproc") { result += "\n"; }
                result += format!("{};\n\n", class_specifier.trim_end()).as_str();
                last_group_kind = "class_specifier";
            },
            "template_declaration" => {
                if last_group_kind.contains("preproc") { result += "\n"; }
                let template_declaration = handle_template_declaration(child, src);
                result += format!("{}\n\n", template_declaration).as_str();
                last_group_kind = "template_declaration";
            },
            "template_instantiation" => {
                let template_instantiation = handle_template_instantiation(child, src);
                result += format!("{}\n", template_instantiation).as_str();
                last_group_kind = "template_instantiation";
            },
            "linkage_specification" => {
                let linkage_specification = handle_linkage_specification(child, src);
                result += format!("{}\n\n", linkage_specification).as_str();
                last_group_kind = "linkage_specification";
            },
            "friend_declaration" => {
                let friend_declaration = handle_friend_declaration(child, src);
                result += format!("{}\n", friend_declaration).as_str();
                last_group_kind = "friend_declaration";
            },
            "namespace_alias_definition" => {
                if last_group_kind != "namespace_alias_definition" { result += "\n"; }
                let namespace_alias_definition = handle_namespace_alias_definition(child, src);
                result += format!("{}\n\n", namespace_alias_definition).as_str();
                last_group_kind = "namespace_alias_definition";
            },
            "namespace_definition" => {
                if last_group_kind != "namespace_definition" && last_group_kind != "comment" { result += "\n"; }
                let namespace_definition = handle_namespace_definition(child, src);
                result += format!("{}\n\n", namespace_definition).as_str();
                last_group_kind = "namespace_definition";
            },
            "alias_declaration" => {
                if last_group_kind != "alias_declaration" { result += "\n"; }
                let alias_declaration = handle_alias_declaration(child, src);
                result += format!("{}\n\n", alias_declaration).as_str();
                last_group_kind = "alias_declaration";
            },
            "static_assert_declaration" => {
                if last_group_kind != "static_assert_declaration" { result += "\n"; }
                let static_assert_declaration = handle_static_assert_declaration(child, src);
                result += format!("{}\n\n", static_assert_declaration).as_str();
                last_group_kind = "static_assert_declaration";
            },
            "namespace" => {
                if last_group_kind != "namespace" { result += "\n"; }
                let namespace = handle_namespace(child, src);
                result += format!("{}\n\n", namespace).as_str();
                last_group_kind = "namespace";
            },
            "identifier" => {
                if result.ends_with("#ifdef ") || result.ends_with("#ifndef ") || result.ends_with("#if ") {
                    let identifier = ast::handle_identifier(&CTX, child, src);
                    result += format!("{}\n", identifier).as_str();
                } else {
                    if last_group_kind != "identifier" { result += "\n"; }
                    let identifier = ast::handle_identifier(&CTX, child, src);
                    result += format!("{}\n\n", identifier).as_str();
                }
                last_group_kind = "identifier";
            },
            "function_declarator" => {
                if last_group_kind != "function_declarator" { result += "\n"; }
                let function_declarator = ast::handle_function_declarator(&CTX, child, src);
                result += format!("{}\n\n", function_declarator).as_str();
                last_group_kind = "function_declarator";
            },
            "for_statement" => {
                let for_statement = ast::handle_for_statement(&CTX, child, src);
                result += format!("{}\n\n", for_statement).as_str();
                last_group_kind = "for_statement";
            },
            "for_range_loop" => {
                let for_range_loop = handle_for_range_loop(child, src);
                result += format!("{}\n\n", for_range_loop).as_str();
                last_group_kind = "for_range_loop";
            },
            "while_statement" => {
                let while_statement = ast::handle_while_statement(&CTX, child, src);
                result += format!("{}\n\n", while_statement).as_str();
                last_group_kind = "while_statement";
            },
            "do_statement" => {
                let do_statement = ast::handle_do_statement(&CTX, child, src);
                result += format!("{}\n\n", do_statement).as_str();
                last_group_kind = "do_statement";
            },
            "labeled_statement" => {
                let labeled_statement = ast::handle_labeled_statement(&CTX, child, src);
                result += format!("{}\n", labeled_statement).as_str();
                last_group_kind = "labeled_statement";
            },
            "attributed_statement" => {
                let attributed_statement = ast::handle_attributed_statement(&CTX, child, src);
                result += format!("{}\n", attributed_statement).as_str();
                last_group_kind = "attributed_statement";
            },
            "preproc_else" => {
                let preproc_else = ast::handle_preproc_else(&CTX, child, src);
                result += format!("{}\n\n", preproc_else).as_str();
                last_group_kind = "preproc_else";
            },
            "primitive_type" => {
                let primitive_type = ast::handle_primitive_type(child, src);
                result += format!("{} ", primitive_type).as_str();
                last_group_kind = "primitive_type";
            },
            "template" => {
                if last_group_kind.contains("preproc") { result += "\n"; }
                result += "template ";
                last_group_kind = "template";
            },
            "template_parameter_list" => {
                let tpl = handle_template_parameter_list(child, src);
                result += format!("{}\n", tpl).as_str();
                last_group_kind = "template_parameter_list";
            },
            "condition_clause" => {
                let cc = handle_condition_clause(child, src);
                result += format!("{}\n", cc).as_str();
                last_group_kind = "condition_clause";
            },
            "else_clause" => {
                let ec = ast::handle_else_clause(&CTX, child, src);
                result += format!("{}\n\n", ec).as_str();
                last_group_kind = "else_clause";
            },
            "string_literal" => {
                let sl = ast::handle_string_literal(&CTX, child, src);
                result += format!("{}\n", sl).as_str();
                last_group_kind = "string_literal";
            },
            "preproc_defined" => {
                let pd = ast::handle_preproc_defined(&CTX, child, src);
                result += format!("{}\n", pd).as_str();
                last_group_kind = "preproc_defined";
            },
            "#ifdef" => {
                if !last_group_kind.contains("preproc") { result += "\n"; }
                result += "#ifdef ";
                last_group_kind = "preproc_ifdef";
            },
            "#ifndef" => {
                if !last_group_kind.contains("preproc") { result += "\n"; }
                result += "#ifndef ";
                last_group_kind = "preproc_ifdef";
            },
            "#if" => {
                if !last_group_kind.contains("preproc") { result += "\n"; }
                result += "#if ";
                last_group_kind = "preproc_if";
            },
            "#endif" => {
                result += "#endif\n\n";
                last_group_kind = "preproc_ifdef";
            },
            "#else" => {
                result += "#else\n";
                last_group_kind = "preproc_else";
            },
            "extern" => {
                result += "extern ";
                last_group_kind = "extern";
            },
            "if" => {
                result += "if ";
                last_group_kind = "if";
            },
            "else" => {
                result += "else ";
                last_group_kind = "else";
            },
            "while" => {
                result += "while ";
                last_group_kind = "while";
            },
            "\n" => (),
            ";" => (), // handled in functions above
            "{" => (), // handled in functions above
            "}" => {
                result += "}\n\n";
                last_group_kind = "}";
            },
            _ => println!("Unknown cpp grammar name 1: {}\n", &child.grammar_name()),
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
    result = result.trim_start().to_string();
    return result;
}

fn handle_null<'a>(root: Node, src: &'a str) -> &'a str {
    root.utf8_text(src.as_bytes()).unwrap_or("nullptr")
}

fn handle_using_declaration(root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "using" => result += "using ",
            "namespace" => result += "namespace ",
            "identifier" => {
                let identifier = ast::handle_identifier(&CTX, node, src);
                result += &identifier;
            },
            "qualified_identifier" => {
                let qualified_identifier = handle_qualified_identifier(node, src);
                result += &qualified_identifier;
            },
            ";" => result += ";",
            _ => println!("Using declaration: {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_for_range_loop(root: Node, src: &str) -> String {
    let mut vec = Vec::<Cow<'_, str>>::new();
    let mut temp = String::new();
    let mut reached_compound = false;
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "compound_statement" => {
                reached_compound = true;
                let compound_statement = ast::handle_compound_statement(&CTX, node, src);
                vec.push(Cow::from(compound_statement));
            },
            "if_statement" => {
                let if_statement = ast::handle_if_statement(&CTX, node, src);
                vec.push(Cow::from(if_statement));
            },
            "update_expression" => {
                let update_expression = ast::handle_update_expression(node, src);
                if !reached_compound { temp += &update_expression; }
                else { vec.push(Cow::from(update_expression)); }
            },
            "assignment_expression" => {
                let assignment_expression = ast::handle_assignment_expression(&CTX, node, src);
                vec.push(Cow::from(assignment_expression));
            },
            "binary_expression" => {
                let binary_expression = ast::handle_binary_expression(&CTX, node, src);
                if !reached_compound { temp += format!(" {}", binary_expression).as_str(); }
                else { vec.push(Cow::from(binary_expression)); }
            },
            "declaration" => {
                let declaration = ast::handle_declaration(&CTX, node, src);
                if !reached_compound { temp += &declaration; }
                else { vec.push(Cow::from(declaration)); }
            },
            "expression_statement" => {
                let expression_statement = ast::handle_expression_statement(&CTX, node, src);
                vec.push(Cow::from(expression_statement));
            },
            "identifier" => {
                let identifier = ast::handle_identifier(&CTX, node, src);
                temp += format!("{} ", identifier).as_str();
            },
            "qualified_identifier" => {
                let qualified_identifier = handle_qualified_identifier(node, src);
                temp += format!("{} ", qualified_identifier).as_str();
            },
            "primitive_type" => {
                let primitive_type = ast::handle_primitive_type(node, src);
                temp += format!("{} ", primitive_type).as_str();
            },
            "template_type" => {
                let template_type = handle_template_type(node, src);
                temp += format!("{} ", template_type).as_str();
            },
            "field_expression" => {
                let field_expression = ast::handle_field_expression(&CTX, node, src);
                temp += format!("{} ", field_expression).as_str();
            },
            "reference_declarator" => {
                let reference_declarator = handle_reference_declarator(node, src);
                temp += format!("{} ", reference_declarator).as_str();
            },
            "type_qualifier" => {
                let type_qualifier = ast::handle_type_qualifier(&CTX, node, src);
                temp += format!("{} ", type_qualifier).as_str();
            },
            "call_expression" => {
                let call_expression = ast::handle_call_expression(&CTX, node, src);
                temp += format!("{} ", call_expression).as_str();
            },
            "pointer_declarator" => {
                let pointer_declarator = ast::handle_pointer_declarator(&CTX, node, src);
                temp += format!("{} ", pointer_declarator).as_str();
            },
            "structured_binding_declarator" => {
                let structured_binding_declarator = handle_structured_binding_declarator(node, src);
                temp += format!("{} ", structured_binding_declarator).as_str();
            },
            "sized_type_specifier" => {
                let sized_type_specifier = ast::handle_sized_type_specifier(&CTX, node, src);
                temp += format!("{} ", sized_type_specifier).as_str();
            },
            "pointer_expression" => {
                let pointer_expression = ast::handle_pointer_expression(&CTX, node, src);
                temp += format!("{} ", pointer_expression).as_str();
            },
            "subscript_expression" => {
                let subscript_expression = ast::handle_subscript_expression(&CTX, node, src);
                temp += format!("{} ", subscript_expression).as_str();
            },
            "unary_expression" => {
                let unary_expression = ast::handle_unary_expression(&CTX, node, src);
                temp += format!("{} ", unary_expression).as_str();
            },
            "parenthesized_expression" => {
                let parenthesized_expression = ast::handle_parenthesized_expression(&CTX, node, src);
                temp += format!("{} ", parenthesized_expression).as_str();
            },
            "return_statement" => {
                let return_statement = ast::handle_return_statement(&CTX, node, src);
                vec.push(Cow::from(return_statement));
            },
            "break_statement" => vec.push(Cow::Borrowed("break;")),
            "continue_statement" => vec.push(Cow::Borrowed("continue;")),
            "while_statement" => {
                let while_statement = ast::handle_while_statement(&CTX, node, src);
                vec.push(Cow::from(while_statement));
            },
            "for_statement" => {
                let for_statement = ast::handle_for_statement(&CTX, node, src);
                vec.push(Cow::from(for_statement));
            },
            "for_range_loop" => {
                let for_range_loop = handle_for_range_loop(node, src);
                vec.push(Cow::from(for_range_loop));
            },
            "initializer_list" => {
                let initializer_list = ast::handle_initializer_list(&CTX, node, src);
                if !reached_compound {
                    temp += format!("{} ", initializer_list).as_str();
                } else {
                    vec.push(Cow::from(initializer_list));
                }
            },
            "attributed_statement" => {
                let attributed_statement = ast::handle_attributed_statement(&CTX, node, src);
                vec.push(Cow::from(attributed_statement));
            },
            "do_statement" => {
                let do_statement = ast::handle_do_statement(&CTX, node, src);
                vec.push(Cow::from(do_statement));
            },
            "switch_statement" => {
                let switch_statement = ast::handle_switch_statement(&CTX, node, src);
                vec.push(Cow::from(switch_statement));
            },
            "goto_statement" => {
                let goto_statement = ast::handle_goto_statement(&CTX, node, src);
                vec.push(Cow::from(goto_statement));
            },
            "comment" => {
                let comment = ast::handle_comment(&CTX, node, src);
                if !reached_compound {
                    utils::append_expr_comment(&mut temp, &comment, node.start_byte(), node.end_byte(), src);
                } else {
                    utils::push_expr_comment(&mut vec, &comment, node.start_byte(), node.end_byte(), src);
                }
            },
            "ERROR" => {
                let error = ast::handle_error(&CTX, node, src);
                if !reached_compound {
                    temp += format!("{} ", error).as_str();
                } else {
                    vec.push(Cow::from(error));
                }
            },
            "placeholder_type_specifier" => temp += "auto ",
            ":" => temp += ": ",
            ";" => temp += "; ",
            "(" => temp += "(",
            ")" => {
                reached_compound = true;
                temp = temp.trim_end().to_string();
                temp += ")";
                temp = utils::remove_pointer_spaces(&temp);
                temp = utils::remove_reference_spaces(&temp);
                vec.push(Cow::from(temp));
                temp = "".to_string();
            },
            "for" => temp += "for ",
            _ => println!("You shouldn't be here (cpp for_range_loop): {} : {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap()),
        }
    }
    let result = vec.join(" ");
    return result;
}

fn handle_qualified_identifier(root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = ast::handle_identifier(&CTX, node, src);
                result += &identifier;
            },
            "template_type" => {
                let template_type = handle_template_type(node, src);
                result += &template_type;
            },
            "qualified_identifier" => {
                let qualified_identifier = handle_qualified_identifier(node, src);
                result += &qualified_identifier;
            },
            "template_function" => {
                let template_function = handle_template_function(node, src);
                result += &template_function;
            },
            "operator_name" => {
                let operator_name = handle_operator_name(node, src);
                result += &operator_name;
            },
            "destructor_name" => {
                let destructor_name = handle_destructor_name(node, src);
                result += &destructor_name;
            },
            "operator_cast" => {
                let operator_cast = handle_operator_cast(node, src);
                result += &operator_cast;
            },
            "dependent_name" => {
                result += node.utf8_text(src.as_bytes()).unwrap_or("");
            },
            "ERROR" => {
                let error = ast::handle_error(&CTX, node, src);
                result += &error;
            },
            "template" => result += "template ",
            "::" => {
                if node.is_missing() {
                    if !result.is_empty() && !result.ends_with(' ') {
                        result += " ";
                    }
                } else {
                    result += "::";
                }
            },
            _ => println!("You shouldn't be here (cpp qualified_identifier): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_template_type(root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = ast::handle_identifier(&CTX, node, src);
                result += &identifier;
            },
            "qualified_identifier" => {
                let qualified_identifier = handle_qualified_identifier(node, src);
                result += &qualified_identifier;
            },
            "template_argument_list" => {
                let template_argument_list = handle_template_argument_list(node, src);
                result += &template_argument_list;
            },
            _ => println!("You shouldn't be here (cpp template_type): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_template_function(root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = ast::handle_identifier(&CTX, node, src);
                result += &identifier;
            },
            "qualified_identifier" => {
                let qualified_identifier = handle_qualified_identifier(node, src);
                result += &qualified_identifier;
            },
            "operator_name" => {
                let operator_name = handle_operator_name(node, src);
                result += &operator_name;
            },
            "template_argument_list" => {
                let template_argument_list = handle_template_argument_list(node, src);
                result += &template_argument_list;
            },
            _ => println!("You shouldn't be here (cpp template_function): {} : {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_template_argument_list(root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "type_descriptor" => {
                let type_descriptor = ast::handle_type_descriptor(&CTX, node, src);
                result += &type_descriptor;
            },
            "template_function" => {
                let template_function = handle_template_function(node, src);
                result += &template_function;
            },
            "qualified_identifier" => {
                let qualified_identifier = handle_qualified_identifier(node, src);
                result += &qualified_identifier;
            },
            "identifier" => {
                let identifier = ast::handle_identifier(&CTX, node, src);
                result += &identifier;
            },
            "call_expression" => {
                let call_expression = ast::handle_call_expression(&CTX, node, src);
                result += &call_expression;
            },
            "parenthesized_expression" => {
                let parenthesized_expression = ast::handle_parenthesized_expression(&CTX, node, src);
                result += &parenthesized_expression;
            },
            "unary_expression" => {
                let unary_expression = ast::handle_unary_expression(&CTX, node, src);
                result += &unary_expression;
            },
            "field_expression" => {
                let field_expression = ast::handle_field_expression(&CTX, node, src);
                result += &field_expression;
            },
            "pointer_expression" => {
                let pointer_expression = ast::handle_pointer_expression(&CTX, node, src);
                result += &pointer_expression;
            },
            "binary_expression" => {
                let binary_expression = ast::handle_binary_expression(&CTX, node, src);
                result += &binary_expression;
            },
            "parameter_pack_expansion" => {
                let parameter_pack_expansion = handle_parameter_pack_expansion(node, src);
                result += &parameter_pack_expansion;
            },
            "number_literal" => {
                let number_literal = ast::handle_number_literal(node, src);
                result += &number_literal;
            },
            "true" => result += "true",
            "false" => result += "false",
            "null" => result += handle_null(node, src),
            "<" => result += "<",
            ">" => result += ">",
            "," => result += ", ",
            _ => println!("You shouldn't be here (cpp template_argument_list): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_condition_clause(root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "binary_expression" => {
                let binary_expression = ast::handle_binary_expression(&CTX, node, src);
                result += &binary_expression;
            },
            "unary_expression" => {
                let unary_expression = ast::handle_unary_expression(&CTX, node, src);
                result += &unary_expression;
            },
            "call_expression" => {
                let call_expression = ast::handle_call_expression(&CTX, node, src);
                result += &call_expression;
            },
            "identifier" => {
                let identifier = ast::handle_identifier(&CTX, node, src);
                result += &identifier;
            },
            "qualified_identifier" => {
                let qualified_identifier = handle_qualified_identifier(node, src);
                result += &qualified_identifier;
            },
            "field_expression" => {
                let field_expression = ast::handle_field_expression(&CTX, node, src);
                result += &field_expression;
            },
            "pointer_expression" => {
                let pointer_expression = ast::handle_pointer_expression(&CTX, node, src);
                result += &pointer_expression;
            },
            "subscript_expression" => {
                let subscript_expression = ast::handle_subscript_expression(&CTX, node, src);
                result += &subscript_expression;
            },
            "parenthesized_expression" => {
                let parenthesized_expression = ast::handle_parenthesized_expression(&CTX, node, src);
                result += &parenthesized_expression;
            },
            "cast_expression" => {
                let cast_expression = ast::handle_cast_expression(&CTX, node, src);
                result += &cast_expression;
            },
            "assignment_expression" => {
                let assignment_expression = ast::handle_assignment_expression(&CTX, node, src);
                result += &assignment_expression;
            },
            "conditional_expression" => {
                let conditional_expression = ast::handle_conditional_expression(&CTX, node, src);
                result += &conditional_expression;
            },
            "update_expression" => {
                let update_expression = ast::handle_update_expression(node, src);
                result += &update_expression;
            },
            "declaration" => {
                let declaration = ast::handle_declaration(&CTX, node, src);
                result += &declaration;
            },
            "init_declarator" => {
                let init_declarator = ast::handle_init_declarator(&CTX, node, src);
                result += &init_declarator;
            },
            "number_literal" => {
                let number_literal = ast::handle_number_literal(node, src);
                result += &number_literal;
            },
            "char_literal" => {
                let char_literal = ast::handle_char_literal(node, src);
                result += &char_literal;
            },
            "init_statement" => {
                let init_statement = handle_init_statement(node, src);
                result += &init_statement;
                if !result.ends_with(' ') {
                    result += " ";
                }
            },
            "sizeof_expression" => {
                let sizeof_expression = ast::handle_sizeof_expression(&CTX, node, src);
                result += &sizeof_expression;
            },
            "string_literal" => {
                let string_literal = ast::handle_string_literal(&CTX, node, src);
                result += &string_literal;
            },
            "concatenated_string" => {
                let concatenated_string = ast::handle_concatenated_string(&CTX, node, src);
                result += &concatenated_string;
            },
            "raw_string_literal" => {
                let raw_string_literal = ast::handle_raw_string_literal(node, src);
                result += raw_string_literal;
            },
            "template_function" => {
                let template_function = handle_template_function(node, src);
                result += &template_function;
            },
            "comment" => {
                let comment = ast::handle_comment(&CTX, node, src);
                utils::append_expr_comment(&mut result, &comment, node.start_byte(), node.end_byte(), src);
            },
            "ERROR" => {
                let error = ast::handle_error(&CTX, node, src);
                result += &error;
            },
            "(" => result += "(",
            ")" => result += ")",
            ";" => result += "; ",
            "true" => result += "true",
            "false" => result += "false",
            "null" => result += handle_null(node, src),
            "this" => result += "this",
            _ => println!("You shouldn't be here (cpp condition_clause): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_subscript_argument_list(root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "number_literal" => {
                let number_literal = ast::handle_number_literal(node, src);
                result += &number_literal;
            },
            "identifier" => {
                let identifier = ast::handle_identifier(&CTX, node, src);
                result += &identifier;
            },
            "qualified_identifier" => {
                let qualified_identifier = handle_qualified_identifier(node, src);
                result += &qualified_identifier;
            },
            "field_expression" => {
                let field_expression = ast::handle_field_expression(&CTX, node, src);
                result += &field_expression;
            },
            "update_expression" => {
                let update_expression = ast::handle_update_expression(node, src);
                result += &update_expression;
            },
            "cast_expression" => {
                let cast_expression = ast::handle_cast_expression(&CTX, node, src);
                result += &cast_expression;
            },
            "parenthesized_expression" => {
                let parenthesized_expression = ast::handle_parenthesized_expression(&CTX, node, src);
                result += &parenthesized_expression;
            },
            "pointer_expression" => {
                let pointer_expression = ast::handle_pointer_expression(&CTX, node, src);
                result += &pointer_expression;
            },
            "unary_expression" => {
                let unary_expression = ast::handle_unary_expression(&CTX, node, src);
                result += &unary_expression;
            },
            "conditional_expression" => {
                let conditional_expression = ast::handle_conditional_expression(&CTX, node, src);
                result += &conditional_expression;
            },
            "binary_expression" => {
                let binary_expression = ast::handle_binary_expression(&CTX, node, src);
                result += &binary_expression;
            },
            "string_literal" => {
                let string_literal = ast::handle_string_literal(&CTX, node, src);
                result += &string_literal;
            },
            "char_literal" => {
                let char_literal = ast::handle_char_literal(node, src);
                result += &char_literal;
            },
            "subscript_expression" => {
                let subscript_expression = ast::handle_subscript_expression(&CTX, node, src);
                result += &subscript_expression;
            },
            "call_expression" => {
                let call_expression = ast::handle_call_expression(&CTX, node, src);
                result += &call_expression;
            },
            "initializer_list" => {
                let initializer_list = ast::handle_initializer_list(&CTX, node, src);
                result += &initializer_list;
            },
            "concatenated_string" => {
                let concatenated_string = ast::handle_concatenated_string(&CTX, node, src);
                result += &concatenated_string;
            },
            "raw_string_literal" => {
                let raw_string_literal = ast::handle_raw_string_literal(node, src);
                result += raw_string_literal;
            },
            "sizeof_expression" => {
                let sizeof_expression = ast::handle_sizeof_expression(&CTX, node, src);
                result += &sizeof_expression;
            },
            "assignment_expression" => {
                let assignment_expression = ast::handle_assignment_expression(&CTX, node, src);
                result += &assignment_expression;
            },
            "this" => result += "this",
            "true" => result += "true",
            "false" => result += "false",
            "ERROR" => {
                let error = ast::handle_error(&CTX, node, src);
                result += &error;
            },
            "," => result += ", ",
            "[" => result += "[",
            "]" => result += "]",
            _ => println!("You shouldn't be here (cpp subscript_argument_list): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_field_initializer_list(root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "field_initializer" => {
                let field_initializer = handle_field_initializer(node, src);
                result += &field_initializer;
            },
            "comment" => {
                let comment = ast::handle_comment(&CTX, node, src);
                utils::append_expr_comment(&mut result, &comment, node.start_byte(), node.end_byte(), src);
            },
            "ERROR" => {
                let error = ast::handle_error(&CTX, node, src);
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
            "..." => result += "...",
            ":" => result += "\t: ",
            "," => result += ", ",
            _ => println!("You shouldn't be here (cpp field_initializer_list): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_field_initializer(root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = ast::handle_identifier(&CTX, node, src);
                result += &identifier;
            },
            "qualified_identifier" => {
                let qualified_identifier = handle_qualified_identifier(node, src);
                result += &qualified_identifier;
            },
            "template_type" => {
                let template_type = handle_template_type(node, src);
                result += &template_type;
            },
            "argument_list" => {
                let argument_list = ast::handle_argument_list(&CTX, node, src);
                result += &argument_list;
            },
            "initializer_list" => {
                let initializer_list = ast::handle_initializer_list(&CTX, node, src);
                result += &initializer_list;
            },
            _ => println!("You shouldn't be here (cpp field_initializer): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_reference_declarator(root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = ast::handle_identifier(&CTX, node, src);
                result += &identifier;
            },
            "qualified_identifier" => {
                let qualified_identifier = handle_qualified_identifier(node, src);
                result += &qualified_identifier;
            },
            "function_declarator" => {
                let function_declarator = ast::handle_function_declarator(&CTX, node, src);
                result += &function_declarator;
            },
            "pointer_declarator" => {
                let pointer_declarator = ast::handle_pointer_declarator(&CTX, node, src);
                result += &pointer_declarator;
            },
            "array_declarator" => {
                let array_declarator = ast::handle_array_declarator(&CTX, node, src);
                result += &array_declarator;
            },
            "parenthesized_declarator" => {
                let parenthesized_declarator = ast::handle_parenthesized_declarator(&CTX, node, src);
                result += &parenthesized_declarator;
            },
            "variadic_declarator" => {
                let variadic_declarator = handle_variadic_declarator(node, src);
                result += &variadic_declarator;
            },
            "structured_binding_declarator" => {
                let structured_binding_declarator = handle_structured_binding_declarator(node, src);
                result += &structured_binding_declarator;
            },
            "ERROR" => {
                let error = ast::handle_error(&CTX, node, src);
                result += &error;
            },
            "&" => result += "& ",
            "&&" => result += "&& ",
            _ => println!("You shouldn't be here (cpp reference_declarator): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_abstract_reference_declarator(root: Node, src: &str) -> String {
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

fn handle_class_specifier(root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = ast::handle_identifier(&CTX, node, src);
                result += format!(" {}", identifier).as_str();
            },
            "qualified_identifier" => {
                let qualified_identifier = handle_qualified_identifier(node, src);
                result += format!(" {}", qualified_identifier).as_str();
            },
            "template_type" => {
                let template_type = handle_template_type(node, src);
                result += format!(" {}", template_type).as_str();
            },
            "virtual_specifier" => {
                let virtual_specifier = handle_virtual_specifier(node, src);
                result += format!(" {}", virtual_specifier).as_str();
            },
            "base_class_clause" => {
                let base_class_clause = handle_base_class_clause(node, src);
                result += format!(" {}", base_class_clause).as_str();
            },
            "attribute_specifier" | "attribute_declaration" => {
                let attribute = ast::handle_attribute(node, src);
                result += format!(" {}", attribute).as_str();
            },
            "field_declaration_list" => {
                let field_declaration_list = ast::handle_field_declaration_list(&CTX, node, src);
                result = result.trim_end().to_string();
                result += format!("\n{}", field_declaration_list).as_str();
            },
            "class" => result += "class",
            _ => println!("You shouldn't be here (cpp class_specifier): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_template_declaration(root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "template_parameter_list" => {
                let template_parameter_list = handle_template_parameter_list(node, src);
                result += format!("{} ", template_parameter_list).as_str();
            },
            "function_definition" => {
                let function_definition = ast::handle_function_definition(&CTX, node, src);
                result += &function_definition;
            },
            "declaration" => {
                let declaration = ast::handle_declaration(&CTX, node, src);
                result += &declaration;
            },
            "field_declaration" => {
                let field_declaration = ast::handle_field_declaration(&CTX, node, src);
                result += &field_declaration;
            },
            "struct_specifier" => {
                let struct_specifier = ast::handle_struct_specifier(&CTX, node, src);
                result += format!("{};", struct_specifier.trim_end()).as_str();
            },
            "class_specifier" => {
                let class_specifier = handle_class_specifier(node, src);
                result += format!("{};", class_specifier.trim_end()).as_str();
            },
            "alias_declaration" => {
                let alias_declaration = handle_alias_declaration(node, src);
                result += &alias_declaration;
            },
            "type_definition" => {
                let type_definition = ast::handle_type_definition(&CTX, node, src);
                result += &type_definition;
            },
            "template_declaration" => {
                let template_declaration = handle_template_declaration(node, src);
                result += &template_declaration;
            },
            "friend_declaration" => {
                let friend_declaration = handle_friend_declaration(node, src);
                result += &friend_declaration;
            },
            "comment" => {
                let comment = ast::handle_comment(&CTX, node, src);
                utils::append_comment_to_result(&mut result, &comment, node.start_byte(), src, "", "\n", true);
            },
            "template" => result += "template ",
            ";" => (),
            _ => println!("You shouldn't be here (cpp template_declaration): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_template_parameter_list(root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "type_parameter_declaration" => {
                let type_parameter_declaration = handle_type_parameter_declaration(node, src);
                result += &type_parameter_declaration;
            },
            "parameter_declaration" => {
                let parameter_declaration = ast::handle_parameter_declaration(&CTX, node, src);
                result += &parameter_declaration;
            },
            "variadic_parameter_declaration" => {
                let variadic_parameter_declaration = handle_variadic_parameter_declaration(node, src);
                result += &variadic_parameter_declaration;
            },
            "variadic_type_parameter_declaration" => {
                let variadic_type_parameter_declaration = handle_variadic_type_parameter_declaration(node, src);
                result += &variadic_type_parameter_declaration;
            },
            "optional_type_parameter_declaration" => {
                let optional_type_parameter_declaration = handle_optional_type_parameter_declaration(node, src);
                result += &optional_type_parameter_declaration;
            },
            "optional_parameter_declaration" => {
                let optional_parameter_declaration = handle_optional_parameter_declaration(node, src);
                result += &optional_parameter_declaration;
            },
            "<" => result += "<",
            ">" => result += ">",
            "," => result += ", ",
            _ => println!("You shouldn't be here (cpp template_parameter_list): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_type_parameter_declaration(root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = ast::handle_identifier(&CTX, node, src);
                result += &identifier;
            },
            "typename" => result += "typename ",
            "class" => result += "class ",
            _ => println!("You shouldn't be here (type_parameter_declaration): {} : {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result.trim_end().to_string();
}

fn handle_access_specifier(root: Node, src: &str) -> String {
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

fn handle_try_statement(root: Node, src: &str) -> String {
    let mut parts = Vec::<Cow<'_, str>>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "try" => parts.push(Cow::Borrowed("try")),
            "compound_statement" => {
                let compound_statement = ast::handle_compound_statement(&CTX, node, src);
                parts.push(Cow::from(compound_statement));
            }
            "catch_clause" => {
                let catch_clause = handle_catch_clause(node, src);
                parts.push(Cow::from(catch_clause));
            }
            _ => println!("You shouldn't be here (cpp try_statement): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let result = parts.join(" ");
    return result;
}

fn handle_catch_clause(root: Node, src: &str) -> String {
    let mut parts = Vec::<Cow<'_, str>>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "catch" => parts.push(Cow::Borrowed("catch")),
            "parameter_list" => {
                let parameter_list = ast::handle_parameter_list(&CTX, node, src);
                parts.push(Cow::from(parameter_list));
            },
            "compound_statement" => {
                let compound_statement = ast::handle_compound_statement(&CTX, node, src);
                parts.push(Cow::from(compound_statement));
            },
            _ => println!("You shouldn't be here (cpp catch_clause): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let result = parts.join(" ");
    return result;
}

fn handle_lambda_expression(root: Node, src: &str) -> String {
    let mut parts = Vec::<Cow<'_, str>>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "lambda_capture_specifier" => {
                let lambda_capture_specifier = handle_lambda_capture_specifier(node, src);
                parts.push(Cow::from(lambda_capture_specifier));
            },
            "abstract_function_declarator" => {
                let abstract_function_declarator = ast::handle_abstract_function_declarator(&CTX, node, src);
                parts.push(Cow::from(abstract_function_declarator));
            },
            "compound_statement" => {
                let compound_statement = ast::handle_compound_statement(&CTX, node, src);
                parts.push(Cow::from(compound_statement));
            },
            "lambda_expression" => {
                let lambda_expression = handle_lambda_expression(node, src);
                parts.push(Cow::from(lambda_expression));
            },
            "template_parameter_list" => {
                let tpl = handle_template_parameter_list(node, src);
                parts.push(Cow::from(tpl));
            },
            "attribute_specifier" | "attribute_declaration" => {
                parts.push(Cow::Borrowed(ast::handle_attribute(node, src)));
            },
            "comment" => {
                let comment = ast::handle_comment(&CTX, node, src);
                utils::push_expr_comment(&mut parts, &comment, node.start_byte(), node.end_byte(), src);
            },
            "ERROR" => {
                let error = ast::handle_error(&CTX, node, src);
                parts.push(Cow::from(error));
            },
            ";" => (), // handled in compound_statement
            _ => println!("You shouldn't be here (cpp lambda_expression): {} : {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let mut result = parts.join(" ");
    // This function happens to work here too
    result = utils::remove_object_constructor_space(&result);
    return result;
}

fn handle_lambda_capture_specifier(root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = ast::handle_identifier(&CTX, node, src);
                result += &identifier;
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

fn handle_namespace_alias_definition(root: Node, src: &str) -> String {
    let mut parts = Vec::<Cow<'_, str>>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "namespace" => parts.push(Cow::Borrowed("namespace")),
            "=" => parts.push(Cow::Borrowed("=")),
            ";" => parts.push(Cow::Borrowed(";")),
            "identifier" => {
                let identifier = ast::handle_identifier(&CTX, node, src);
                parts.push(Cow::from(identifier));
            },
            "nested_namespace_specifier" => {
                let nested_namespace_specifier = handle_nested_namespace_specifier(node, src);
                parts.push(Cow::from(nested_namespace_specifier));
            },
            _ => println!("You shouldn't be here (cpp namespace_alias_definition): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let mut result = parts.join(" ");
    result = utils::remove_unnecessary_spaces(&result);
    return result;
}

fn handle_nested_namespace_specifier(root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = ast::handle_identifier(&CTX, node, src);
                result += &identifier;
            },
            "nested_namespace_specifier" => {
                let nested = handle_nested_namespace_specifier(node, src);
                result += &nested;
            },
            "::" => {
                if node.is_missing() {
                    if !result.is_empty() && !result.ends_with(' ') {
                        result += " ";
                    }
                } else {
                    result += "::";
                }
            },
            _ => println!("You shouldn't be here (cpp nested_namespace_specifier): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_alias_declaration(root: Node, src: &str) -> String {
    let mut parts = Vec::<Cow<'_, str>>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "using" => parts.push(Cow::Borrowed("using")),
            ";" => parts.push(Cow::Borrowed(";")),
            "=" => parts.push(Cow::Borrowed("=")),
            "identifier" => {
                let identifier = ast::handle_identifier(&CTX, node, src);
                parts.push(Cow::from(identifier));
            },
            "type_descriptor" => {
                let type_descriptor = ast::handle_type_descriptor(&CTX, node, src);
                parts.push(Cow::from(type_descriptor));
            },
            "dependent_type" => {
                let dependent_type = handle_dependent_type(node, src);
                parts.push(Cow::from(dependent_type));
            },
            "attribute_specifier" | "attribute_declaration" => {
                parts.push(Cow::Borrowed(ast::handle_attribute(node, src)));
            },
            _ => println!("You shouldn't be here (cpp alias_declaration): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let mut result = parts.join(" ");
    result = utils::remove_unnecessary_spaces(&result);
    return result;
}

fn handle_new_expression(root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "::" => result += "::",
            "new" => result += "new ",
            "identifier" => {
                let identifier = ast::handle_identifier(&CTX, node, src);
                result += &identifier;
            },
            "qualified_identifier" => {
                let qualified_identifier = handle_qualified_identifier(node, src);
                result += &qualified_identifier;
            },
            "template_type" => {
                let template_type = handle_template_type(node, src);
                result += &template_type;
            },
            "primitive_type" => {
                let primitive_type = ast::handle_primitive_type(node, src);
                result += &primitive_type;
            },
            "placement_argument_list" | "argument_list" => {
                let argument_list = ast::handle_argument_list(&CTX, node, src);
                result += &argument_list;
            },
            "new_declarator" => {
                result += node.utf8_text(src.as_bytes()).unwrap_or("");
            },
            "initializer_list" => {
                let initializer_list = ast::handle_initializer_list(&CTX, node, src);
                result += &initializer_list;
            },
            _ => println!("You shouldn't be here (cpp new_expression): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_namespace_definition(root: Node, src: &str) -> String {
    let mut parts = Vec::<Cow<'_, str>>::new();
    let mut temp = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "inline" => temp += "inline ",
            "namespace" => temp += "namespace ",
            "identifier" => {
                let identifier = ast::handle_identifier(&CTX, node, src);
                temp += &identifier;
            },
            "nested_namespace_specifier" => {
                let nested = handle_nested_namespace_specifier(node, src);
                temp += &nested;
            },
            "::" => temp += "::",
            "declaration_list" => {
                if !temp.is_empty() {
                    parts.push(Cow::from(temp.trim_end().to_string()));
                    temp = String::new();
                }
                let declaration_list = ast::handle_declaration_list(&CTX, node, src);
                parts.push(Cow::from(declaration_list));
            },
            _ => println!("You shouldn't be here (cpp namespace_definition): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let result = parts.join("\n");
    return result;
}

fn handle_structured_binding_declarator(root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "[" => result += "[",
            "]" => result += "]",
            "," => result += ", ",
            "identifier" => {
                let identifier = ast::handle_identifier(&CTX, node, src);
                result += &identifier;
            },
            _ => println!("You shouldn't be here (cpp structured_binding_declarator): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_static_assert_declaration(root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "static_assert" => result += "static_assert",
            "binary_expression" => {
                let binary_expression = ast::handle_binary_expression(&CTX, node, src);
                result += &binary_expression;
            },
            "unary_expression" => {
                let unary_expression = ast::handle_unary_expression(&CTX, node, src);
                result += &unary_expression;
            },
            "parenthesized_expression" => {
                let parenthesized_expression = ast::handle_parenthesized_expression(&CTX, node, src);
                result += &parenthesized_expression;
            },
            "call_expression" => {
                let call_expression = ast::handle_call_expression(&CTX, node, src);
                result += &call_expression;
            },
            "template_function" => {
                let template_function = handle_template_function(node, src);
                result += &template_function;
            },
            "qualified_identifier" => {
                let qualified_identifier = handle_qualified_identifier(node, src);
                result += &qualified_identifier;
            },
            "identifier" => {
                let identifier = ast::handle_identifier(&CTX, node, src);
                result += &identifier;
            },
            "string_literal" => {
                let string_literal = ast::handle_string_literal(&CTX, node, src);
                result += &string_literal;
            },
            "concatenated_string" => {
                let concatenated_string = ast::handle_concatenated_string(&CTX, node, src);
                result += &concatenated_string;
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

fn handle_static_assert<'a>(_root: Node, _src: &'a str) -> &'a str {
    "static_assert"
}

fn handle_placeholder_type_specifier(root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = ast::handle_identifier(&CTX, node, src);
                result += format!("{} ", identifier).as_str();
            },
            "qualified_identifier" => {
                let qualified_identifier = handle_qualified_identifier(node, src);
                result += format!("{} ", qualified_identifier).as_str();
            },
            "template_type" => {
                let template_type = handle_template_type(node, src);
                result += format!("{} ", template_type).as_str();
            },
            "decltype" => {
                let decltype = handle_decltype(node, src);
                result += &decltype;
            },
            "auto" => result += "auto ",
            _ => println!("You shouldn't be here (cpp placeholder_type_specifier): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_variadic_parameter_declaration(root: Node, src: &str) -> String {
    let mut parts = Vec::<Cow<'_, str>>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => parts.push(Cow::from(ast::handle_identifier(&CTX, node, src).to_string())),
            "qualified_identifier" => parts.push(Cow::from(handle_qualified_identifier(node, src))),
            "template_type" => parts.push(Cow::from(handle_template_type(node, src))),
            "type_qualifier" => parts.push(Cow::from(ast::handle_type_qualifier(&CTX, node, src).to_string())),
            "primitive_type" => parts.push(Cow::from(ast::handle_primitive_type(node, src).to_string())),
            "reference_declarator" => parts.push(Cow::from(handle_reference_declarator(node, src))),
            "pointer_declarator" => parts.push(Cow::from(ast::handle_pointer_declarator(&CTX, node, src))),
            "variadic_declarator" => {
                let vd = handle_variadic_declarator(node, src);
                if vd.starts_with("...") && !parts.is_empty() {
                    if let Some(last) = parts.last_mut() {
                        *last = Cow::Owned(format!("{}{}", last, vd));
                    }
                } else {
                    parts.push(Cow::from(vd));
                }
            },
            _ => println!("You shouldn't be here: (cpp handle_variadic_parameter_declaration): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return utils::remove_reference_spaces(&parts.join(" "));
}

fn handle_variadic_type_parameter_declaration(root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = ast::handle_identifier(&CTX, node, src);
                result += &identifier;
            },
            "reference_declarator" => {
                let reference_declarator = handle_reference_declarator(node, src);
                result += &reference_declarator;
            },
            "typename" => result += "typename",
            "class" => result += "class",
            "..." => result += "... ",
            _ => println!("You shouldn't be here: (cpp handle_variadic_type_parameter_declaration): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result.trim_end().to_string();
}

fn handle_variadic_declarator(root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = ast::handle_identifier(&CTX, node, src);
                result += &identifier;
            },
            "..." => result += "... ",
            _ => println!("You shouldn't be here (cpp variadic_declarator): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result.trim_end().to_string();
}

fn handle_base_class_clause(root: Node, src: &str) -> String {
    let mut parts = Vec::<Cow<'_, str>>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "access_specifier" => {
                parts.push(Cow::from(node.utf8_text(src.as_bytes()).unwrap_or("").trim().to_string()));
            },
            "public" | "private" | "protected" | "virtual" => {
                parts.push(Cow::from(node.grammar_name().to_string()));
            },
            "identifier" => {
                parts.push(Cow::from(ast::handle_identifier(&CTX, node, src).to_string()));
            },
            "template_type" => {
                parts.push(Cow::from(handle_template_type(node, src)));
            },
            "qualified_identifier" => {
                parts.push(Cow::from(handle_qualified_identifier(node, src)));
            },
            "decltype" => {
                parts.push(Cow::from(handle_decltype(node, src)));
            },
            "..." => parts.push(Cow::Borrowed("...")),
            ":" => parts.push(Cow::Borrowed(":")),
            "," => parts.push(Cow::Borrowed(",")),
            _ => println!("You shouldn't be here (cpp base_class_clause): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let mut result = parts.join(" ");
    if result.contains(",") {
        result = utils::remove_whitespace_before_commas(&result);
    }
    return result;
}

fn handle_parameter_pack_expansion(root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "type_descriptor" => {
                let type_descriptor = ast::handle_type_descriptor(&CTX, node, src);
                result += &type_descriptor;
            },
            "call_expression" => {
                let call_expression = ast::handle_call_expression(&CTX, node, src);
                result += &call_expression;
            },
            "identifier" => {
                let identifier = ast::handle_identifier(&CTX, node, src);
                result += &identifier;
            },
            "qualified_identifier" => {
                let qualified_identifier = handle_qualified_identifier(node, src);
                result += &qualified_identifier;
            },
            "binary_expression" => {
                let binary_expression = ast::handle_binary_expression(&CTX, node, src);
                result += &binary_expression;
            },
            "..." => result += "...",
            _ => println!("You shouldn't be here (cpp parameter_pack_expansion): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_optional_type_parameter_declaration(root: Node, src: &str) -> String {
    let mut parts = Vec::<Cow<'_, str>>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => parts.push(Cow::from(ast::handle_identifier(&CTX, node, src).to_string())),
            "qualified_identifier" => parts.push(Cow::from(handle_qualified_identifier(node, src))),
            "template_type" => parts.push(Cow::from(handle_template_type(node, src))),
            "type_descriptor" => parts.push(Cow::from(ast::handle_type_descriptor(&CTX, node, src))),
            "primitive_type" => parts.push(Cow::from(ast::handle_primitive_type(node, src).to_string())),
            "typename" => parts.push(Cow::Borrowed("typename")),
            "class" => parts.push(Cow::Borrowed("class")),
            "=" => parts.push(Cow::Borrowed("=")),
            _ => println!("You shouldn't be here (cpp optional_type_parameter_declaration): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return parts.join(" ");
}

fn handle_decltype(root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "sizeof_expression" => {
                let sizeof_expression = ast::handle_sizeof_expression(&CTX, node, src);
                result += &sizeof_expression;
            },
            "identifier" => result += ast::handle_identifier(&CTX, node, src),
            "qualified_identifier" => result += handle_qualified_identifier(node, src).as_str(),
            "call_expression" => result += ast::handle_call_expression(&CTX, node, src).as_str(),
            "field_expression" => result += ast::handle_field_expression(&CTX, node, src).as_str(),
            "binary_expression" => result += ast::handle_binary_expression(&CTX, node, src).as_str(),
            "unary_expression" => result += ast::handle_unary_expression(&CTX, node, src).as_str(),
            "decltype" => {
                if node.child_count() == 0 {
                    result += "decltype";
                } else {
                    result += handle_decltype(node, src).as_str();
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

fn handle_destructor_name(root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = ast::handle_identifier(&CTX, node, src);
                result += &identifier;
            },
            "template_type" => {
                let template_type = handle_template_type(node, src);
                result += &template_type;
            },
            "destructor_name" => {
                let destructor_name = handle_destructor_name(node, src);
                result += &destructor_name;
            },
            "::" => {
                if node.is_missing() {
                    if !result.is_empty() && !result.ends_with(' ') {
                        result += " ";
                    }
                } else {
                    result += "::";
                }
            },
            "~" => result += "~",
            _ => println!("You shouldn't be here (cpp destructor_name): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_friend_declaration(root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = ast::handle_identifier(&CTX, node, src);
                result += &identifier;
            },
            "qualified_identifier" => {
                let qualified_identifier = handle_qualified_identifier(node, src);
                result += &qualified_identifier;
            },
            "template_type" => {
                let template_type = handle_template_type(node, src);
                result += &template_type;
            },
            "declaration" => {
                let declaration = ast::handle_declaration(&CTX, node, src);
                result += &declaration;
            },
            "function_definition" => {
                let function_definition = ast::handle_function_definition(&CTX, node, src);
                result += &function_definition;
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

fn handle_pointer_type_declarator(root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = ast::handle_identifier(&CTX, node, src);
                result += format!(" {}", identifier).as_str();
            },
            "qualified_identifier" => {
                let qualified_identifier = handle_qualified_identifier(node, src);
                result += format!(" {}", qualified_identifier).as_str();
            },
            "pointer_type_declarator" => {
                let inner = handle_pointer_type_declarator(node, src);
                result += &inner;
            },
            "function_declarator" => {
                let function_declarator = ast::handle_function_declarator(&CTX, node, src);
                result += format!(" {}", function_declarator).as_str();
            },
            "parenthesized_declarator" => {
                let parenthesized_declarator = ast::handle_parenthesized_declarator(&CTX, node, src);
                result += format!(" {}", parenthesized_declarator).as_str();
            },
            "type_qualifier" => {
                let type_qualifier = ast::handle_type_qualifier(&CTX, node, src);
                result += format!(" {}", type_qualifier).as_str();
            },
            "ms_pointer_modifier" => {
                let modifier = node.utf8_text(src.as_bytes()).unwrap_or("").trim();
                result += format!(" {}", modifier).as_str();
            },
            "ERROR" => {
                let error = ast::handle_error(&CTX, node, src);
                result += format!(" {}", error).as_str();
            },
            "*" => result += "*",
            _ => println!("You shouldn't be here (cpp pointer_type_declarator): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_optional_parameter_declaration(root: Node, src: &str) -> String {
    let mut parts = Vec::<Cow<'_, str>>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => parts.push(Cow::from(ast::handle_identifier(&CTX, node, src).to_string())),
            "qualified_identifier" => parts.push(Cow::from(handle_qualified_identifier(node, src))),
            "primitive_type" => parts.push(Cow::from(ast::handle_primitive_type(node, src).to_string())),
            "sized_type_specifier" => parts.push(Cow::from(ast::handle_sized_type_specifier(&CTX, node, src))),
            "struct_specifier" => parts.push(Cow::from(ast::handle_struct_specifier(&CTX, node, src))),
            "enum_specifier" => parts.push(Cow::from(ast::handle_enum_specifier(&CTX, node, src))),
            "number_literal" => parts.push(Cow::from(ast::handle_number_literal(node, src).to_string())),
            "char_literal" => parts.push(Cow::from(ast::handle_char_literal(node, src).to_string())),
            "type_qualifier" => parts.push(Cow::from(ast::handle_type_qualifier(&CTX, node, src).to_string())),
            "template_type" => parts.push(Cow::from(handle_template_type(node, src))),
            "dependent_type" => parts.push(Cow::from(handle_dependent_type(node, src))),
            "reference_declarator" => parts.push(Cow::from(handle_reference_declarator(node, src))),
            "abstract_reference_declarator" => parts.push(Cow::from(handle_abstract_reference_declarator(node, src))),
            "abstract_pointer_declarator" => parts.push(Cow::from(ast::handle_abstract_pointer_declarator(&CTX, node, src))),
            "abstract_function_declarator" => parts.push(Cow::from(ast::handle_abstract_function_declarator(&CTX, node, src))),
            "abstract_parenthesized_declarator" => parts.push(Cow::from(ast::handle_abstract_parenthesized_declarator(&CTX, node, src))),
            "abstract_array_declarator" => parts.push(Cow::from(ast::handle_abstract_array_declarator(&CTX, node, src))),
            "function_declarator" => parts.push(Cow::from(ast::handle_function_declarator(&CTX, node, src))),
            "parenthesized_declarator" => parts.push(Cow::from(ast::handle_parenthesized_declarator(&CTX, node, src))),
            "array_declarator" => parts.push(Cow::from(ast::handle_array_declarator(&CTX, node, src))),
            "compound_literal_expression" => parts.push(Cow::from(ast::handle_compound_literal_expression(&CTX, node, src))),
            "initializer_list" => parts.push(Cow::from(ast::handle_initializer_list(&CTX, node, src))),
            "string_literal" => parts.push(Cow::from(ast::handle_string_literal(&CTX, node, src))),
            "concatenated_string" => parts.push(Cow::from(ast::handle_concatenated_string(&CTX, node, src))),
            "raw_string_literal" => parts.push(Cow::Borrowed(ast::handle_raw_string_literal(node, src))),
            "pointer_declarator" => parts.push(Cow::from(ast::handle_pointer_declarator(&CTX, node, src))),
            "call_expression" => parts.push(Cow::from(ast::handle_call_expression(&CTX, node, src))),
            "cast_expression" => parts.push(Cow::from(ast::handle_cast_expression(&CTX, node, src))),
            "binary_expression" => parts.push(Cow::from(ast::handle_binary_expression(&CTX, node, src))),
            "unary_expression" => parts.push(Cow::from(ast::handle_unary_expression(&CTX, node, src))),
            "field_expression" => parts.push(Cow::from(ast::handle_field_expression(&CTX, node, src))),
            "subscript_expression" => parts.push(Cow::from(ast::handle_subscript_expression(&CTX, node, src))),
            "sizeof_expression" => parts.push(Cow::from(ast::handle_sizeof_expression(&CTX, node, src))),
            "parenthesized_expression" => parts.push(Cow::from(ast::handle_parenthesized_expression(&CTX, node, src))),
            "null" => parts.push(Cow::from(handle_null(node, src).to_string())),
            "this" => parts.push(Cow::Borrowed("this")),
            "ERROR" => parts.push(Cow::from(ast::handle_error(&CTX, node, src))),
            "=" => parts.push(Cow::Borrowed("=")),
            "true" => parts.push(Cow::Borrowed("true")),
            "false" => parts.push(Cow::Borrowed("false")),
            _ => println!("You shouldn't be here (cpp optional_parameter_declaration): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let mut result = parts.join(" ");
    result = utils::remove_unnecessary_spaces(&result);
    result = utils::remove_reference_spaces(&result);
    return result;
}

fn handle_namespace<'a>(_root: Node, _src: &'a str) -> &'a str {
    "namespace"
}

fn handle_default_method_clause(root: Node, src: &str) -> String {
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

fn handle_delete_method_clause(root: Node, src: &str) -> String {
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

fn handle_explicit_function_specifier(root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "explicit" => result += "explicit",
            "(" => result += "(",
            ")" => result += ")",
            "binary_expression" => result += ast::handle_binary_expression(&CTX, node, src).as_str(),
            "identifier" => result += ast::handle_identifier(&CTX, node, src),
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

fn handle_virtual_specifier<'a>(root: Node, src: &'a str) -> &'a str {
    let mut result = "";
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "final" => result = "final",
            "override" => result = "override",
            _ => println!("You shouldn't be here (cpp virtual_specifier): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    if result.is_empty() {
        return root.utf8_text(src.as_bytes()).unwrap_or("");
    }
    result
}

fn handle_trailing_return_type(root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "->" => result += "-> ",
            "type_descriptor" => result += ast::handle_type_descriptor(&CTX, node, src).as_str(),
            _ => println!("You shouldn't be here (cpp trailing_return_type): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_operator_name(root: Node, src: &str) -> String {
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

fn handle_template_method(root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "template" => result += "template ",
            "identifier" => result += ast::handle_identifier(&CTX, node, src),
            "operator_name" => result += handle_operator_name(node, src).as_str(),
            "template_argument_list" => result += handle_template_argument_list(node, src).as_str(),
            _ => println!("You shouldn't be here (cpp template_method): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_template_instantiation(root: Node, src: &str) -> String {
    let mut parts = Vec::<Cow<'_, str>>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "extern" => parts.push(Cow::Borrowed("extern")),
            "template" => parts.push(Cow::Borrowed("template")),
            "class" => parts.push(Cow::Borrowed("class")),
            "struct" => parts.push(Cow::Borrowed("struct")),
            "class_specifier" => parts.push(Cow::from(handle_class_specifier(node, src))),
            "struct_specifier" => parts.push(Cow::from(ast::handle_struct_specifier(&CTX, node, src))),
            "declaration" => parts.push(Cow::from(ast::handle_declaration(&CTX, node, src))),
            "qualified_identifier" => parts.push(Cow::from(handle_qualified_identifier(node, src))),
            "template_type" => parts.push(Cow::from(handle_template_type(node, src))),
            "identifier" => parts.push(Cow::from(ast::handle_identifier(&CTX, node, src).to_string())),
            "type_qualifier" => parts.push(Cow::from(ast::handle_type_qualifier(&CTX, node, src).to_string())),
            "primitive_type" => parts.push(Cow::from(ast::handle_primitive_type(node, src).to_string())),
            "function_declarator" => parts.push(Cow::from(ast::handle_function_declarator(&CTX, node, src))),
            "pointer_declarator" => parts.push(Cow::from(ast::handle_pointer_declarator(&CTX, node, src))),
            "reference_declarator" => parts.push(Cow::from(handle_reference_declarator(node, src))),
            ";" => parts.push(Cow::Borrowed(";")),
            _ => println!("You shouldn't be here (cpp template_instantiation): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let result = parts.join(" ");
    return utils::remove_unnecessary_spaces(&result);
}

fn handle_linkage_specification(root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "extern" => result += "extern ",
            "string_literal" => result += format!("{} ", ast::handle_string_literal(&CTX, node, src)).as_str(),
            "declaration_list" => {
                let declaration_list = ast::handle_declaration_list(&CTX, node, src);
                result += &declaration_list;
            },
            "declaration" => result += ast::handle_declaration(&CTX, node, src).as_str(),
            "function_definition" => result += ast::handle_function_definition(&CTX, node, src).as_str(),
            "type_definition" => result += ast::handle_type_definition(&CTX, node, src).as_str(),
            _ => println!("You shouldn't be here (cpp linkage_specification): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result.trim_end().to_string();
}

fn handle_delete_expression(root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "::" => result += "::",
            "delete" => result += "delete",
            "[" => result += "[",
            "]" => result += "]",
            "identifier" => result += format!(" {}", ast::handle_identifier(&CTX, node, src)).as_str(),
            "qualified_identifier" => result += format!(" {}", handle_qualified_identifier(node, src)).as_str(),
            "field_expression" => result += format!(" {}", ast::handle_field_expression(&CTX, node, src)).as_str(),
            "subscript_expression" => result += format!(" {}", ast::handle_subscript_expression(&CTX, node, src)).as_str(),
            "pointer_expression" => result += format!(" {}", ast::handle_pointer_expression(&CTX, node, src)).as_str(),
            "call_expression" => result += format!(" {}", ast::handle_call_expression(&CTX, node, src)).as_str(),
            "parenthesized_expression" => result += format!(" {}", ast::handle_parenthesized_expression(&CTX, node, src)).as_str(),
            "this" => result += " this",
            _ => println!("You shouldn't be here (cpp delete_expression): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_throw_statement(root: Node, src: &str) -> String {
    let mut parts = Vec::<Cow<'_, str>>::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "throw" => parts.push(Cow::Borrowed("throw")),
            "call_expression" => parts.push(Cow::from(ast::handle_call_expression(&CTX, node, src))),
            "identifier" => parts.push(Cow::from(ast::handle_identifier(&CTX, node, src).to_string())),
            "qualified_identifier" => parts.push(Cow::from(handle_qualified_identifier(node, src))),
            "string_literal" => parts.push(Cow::from(ast::handle_string_literal(&CTX, node, src))),
            "number_literal" => parts.push(Cow::from(ast::handle_number_literal(node, src).to_string())),
            "new_expression" => parts.push(Cow::from(handle_new_expression(node, src))),
            "binary_expression" => parts.push(Cow::from(ast::handle_binary_expression(&CTX, node, src))),
            ";" => parts.push(Cow::Borrowed(";")),
            _ => println!("You shouldn't be here (cpp throw_statement): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    let result = parts.join(" ");
    return utils::remove_unnecessary_spaces(&result);
}

fn handle_operator_cast(root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "identifier" => {
                let identifier = ast::handle_identifier(&CTX, node, src);
                result += format!(" {}", identifier).as_str();
            },
            "qualified_identifier" => {
                let qualified_identifier = handle_qualified_identifier(node, src);
                result += format!(" {}", qualified_identifier).as_str();
            },
            "primitive_type" => {
                let primitive_type = ast::handle_primitive_type(node, src);
                result += format!(" {}", primitive_type).as_str();
            },
            "template_type" => {
                let template_type = handle_template_type(node, src);
                result += format!(" {}", template_type).as_str();
            },
            "type_qualifier" => {
                let type_qualifier = ast::handle_type_qualifier(&CTX, node, src);
                result += format!(" {}", type_qualifier).as_str();
            },
            "abstract_pointer_declarator" => {
                let abstract_pointer_declarator = ast::handle_abstract_pointer_declarator(&CTX, node, src);
                result += &abstract_pointer_declarator;
            },
            "abstract_reference_declarator" => {
                let abstract_reference_declarator = handle_abstract_reference_declarator(node, src);
                result += &abstract_reference_declarator;
            },
            "abstract_function_declarator" => {
                let abstract_function_declarator = ast::handle_abstract_function_declarator(&CTX, node, src);
                if abstract_function_declarator.starts_with('(') {
                    result += &abstract_function_declarator;
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

fn handle_init_statement(root: Node, src: &str) -> String {
    for node in root.named_children(&mut root.walk()) {
        match node.grammar_name() {
            "declaration" => return ast::handle_declaration(&CTX, node, src),
            "expression_statement" => return ast::handle_expression_statement(&CTX, node, src),
            "alias_declaration" => return handle_alias_declaration(node, src),
            "assignment_expression"
            | "call_expression"
            | "update_expression"
            | "binary_expression"
            | "comma_expression"
            | "unary_expression"
            | "pointer_expression"
            | "field_expression"
            | "subscript_expression" => return ast::handle_expression_statement(&CTX, root, src),
            _ => break,
        }
    }
    ast::handle_declaration(&CTX, root, src)
}

fn handle_dependent_type(root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "typename" => result += "typename ",
            "template" => result += "template ",
            "identifier" | "type_identifier" => result += ast::handle_identifier(&CTX, node, src),
            "qualified_identifier" => result += handle_qualified_identifier(node, src).as_str(),
            "template_type" => result += handle_template_type(node, src).as_str(),
            "dependent_type" => result += handle_dependent_type(node, src).as_str(),
            _ => println!("You shouldn't be here (cpp dependent_type): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}

fn handle_pure_virtual_clause(root: Node, src: &str) -> String {
    let mut result = String::new();
    for node in root.children(&mut root.walk()) {
        match node.grammar_name() {
            "=" => result += " = ",
            "0" | "number_literal" => result += "0",
            ";" => result += ";",
            _ => println!("You shouldn't be here (cpp pure_virtual_clause): {}: {}\n", node.grammar_name(), node.utf8_text(src.as_bytes()).unwrap_or("")),
        }
    }
    return result;
}
