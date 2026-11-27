#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_mut)]

use std::io::stdin;
use tree_sitter::{Parser, Language};
extern "C" { pub fn tree_sitter_c() -> Language; }

mod parser;
mod format;
mod group;
mod utils;
mod options;
mod c_format;
mod cpp_format;
mod ast;
mod c_ast;
mod cpp_ast;
mod txt_format;

fn main() {
    let Some(cli) = parser::parse_args() else {
        return;
    };
    if cli.help {
        options::print_help_menu();
        return;
    }
    let stdin_text = if cli.is_stdin {
        options::read_stdin_text()
    } else {
        String::new()
    };
    let file_type: &str = if cli.is_stdin {
        utils::infer_file_type(&cli.filepath)
    } else {
        utils::infer_file_type(&cli.filepath)
    };
    match file_type {
        ".txt" => {
            format::format_txt_file(&cli.filepath, cli.txt_opts, &cli.titles, cli.write, stdin_text);
        },
        ".go" => {
            format::format_go_file(&cli.filepath);
        },
        ".cpp" | ".cc" | ".C" | ".h" | ".hpp" | ".hh" | ".H" => {
            format::format_cpp_file(&cli.filepath, cli.style, cli.write, stdin_text);
        },
        ".c" => {
            format::format_c_file(&cli.filepath, cli.style, cli.write, stdin_text);
        },
        ".py" => {
            format::format_py_file(&cli.filepath);
        },
        _ => {
            println!("Unsupported file type.");
            utils::print_usage();
            println!("The available file types are:");
            utils::display_file_extensions();
        },
    };
}
