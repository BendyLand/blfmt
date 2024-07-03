#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use crate::options::TxtOpts;

mod parser;
mod format;
mod group;
mod utils;
mod options;

// Usage:
// cargo run <filepath> <flags + opts> 
// cargo run storage/one.txt -o 80 1 -t Cyber Digital Tech-Infused
// cargo run -r

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

    let min_args = options::check_minimum_args(&args);
    if min_args == 1 { return; }

    let file_type = utils::infer_file_type(&filepath);
    match file_type.as_str() {
        "txt" => {
            let opts: TxtOpts = options::get_txt_opts(&args);
            format::format_txt_file(filepath, opts, &args);
        },
        _ => println!("Unknown file type."),
    };
}
