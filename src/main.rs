#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_mut)]

use std::io::stdin;

mod parser;
mod format;
mod group;
mod utils;
mod options;
mod c_format;
mod cpp_format;
mod rs_format;
mod txt_format;

fn main() {
    let maybe_args = parser::parse_args();
    let (filepath, args) = {
        match maybe_args {
            Some((a, b)) => (a, b),
            None => (String::new(), Vec::<String>::new()),
        }
    };
    let help_arg = options::check_help_arg(&args);
    if help_arg == 1 { return; }

    let res_arg = options::check_restore_arg(&args);
    if res_arg == 1 { return; }

    let file_type = utils::infer_file_type(&filepath);
    match file_type.as_str() {
        ".txt" => {
            let opts: options::TxtOpts = options::get_txt_opts(&args);
            format::format_txt_file(filepath, opts, &args);
        },
        ".go" => {
            format::format_go_file(filepath);
        },
        ".cpp" => {
            format::format_cpp_file(filepath); 
        },
        ".c" => {
            format::format_c_file(filepath);
        },
        ".rs" => {
            format::format_rs_file(filepath);
        },
        _ => {
            if utils::check_valid_file_ext(&filepath) {
                format::basic_format(filepath);
            }
            else {
                println!("Unknown file type");
                println!("Valid file types are:");
                utils::display_file_extensions();
            }
        },
    };
}
