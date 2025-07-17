#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_mut)]

use std::io::stdin;
use tree_sitter::{Parser, Language};
extern "C" { pub fn tree_sitter_c() -> Language; }

mod restore;
mod parser;
mod format;
mod group;
mod utils;
mod options;
mod c_format;
mod cpp_format;
mod c_ast;
mod cpp_ast;
mod txt_format;

fn main() {
    let maybe_args = parser::parse_args();
    let (filepath, args) = {
        match maybe_args {
            Some((a, b)) => (a, b),
            None => (String::new(), Vec::<String>::new()),
        }
    };
    let help_arg = options::check_help_arg(&args); if help_arg == 1 { return; }

    let res_arg = options::check_restore_arg(&args);
    if res_arg == 1 { return; }

    let write_arg = options::check_write_arg(&args);
    let file_type = utils::infer_file_type(&filepath);
    match file_type.as_str() {
        ".txt" => {
            let opts: options::TxtOpts = options::get_txt_opts(&args);
            format::format_txt_file(filepath, opts, &args, write_arg);
        },
        ".go" => {
            format::format_go_file(filepath);
        },
        ".cpp" | ".cc" | ".C" | ".hpp" | ".hh" | ".H" => {
            let style = options::get_c_style(&args);
            format::format_cpp_file(filepath, style, write_arg);
        },
        ".c" | ".h" => {
            let style = options::get_c_style(&args);
            format::format_c_file(filepath, style, write_arg);
        },
        ".py" => {
            format::format_py_file(filepath);
        },
        _ => {
            println!("Unknown file type");
            println!("Valid file types are:");
            utils::display_file_extensions();
        },
    };
}
