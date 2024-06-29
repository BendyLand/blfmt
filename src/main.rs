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
// cargo run ../storage/one.txt -o 80 1 -t Cyber Digital Tech

fn main() {
    let maybe_args = parser::parse_args();
    let (filepath, args) = {
        match maybe_args {
            Some((a, b)) => (a, b),
            None => (String::new(), Vec::<String>::new()),
        }
    };
    let contains_restore_arg = {
        args.contains(&"-r".to_string()) || 
        args.contains(&"--restore".to_string())
    };
    if contains_restore_arg {
        utils::restore_test_files();
        println!("Test files restored.");
        return;
    }
    let contains_help_arg = {
        args.contains(&"-h".to_string()) || 
        args.contains(&"--help".to_string())
    };
    if contains_help_arg {
        utils::print_usage();
        return;
    }
    if &args.len() < &2 {
        println!("Invalid arguments provided.");
        utils::print_usage();
        return;
    }
    let file_type = utils::infer_file_type(&filepath);
    match file_type.as_str() {
        "txt" => {
            let opts: TxtOpts;
            if args.contains(&"-o".to_string()) || args.contains(&"--opts".to_string()) {
                let args = args.iter().map(|x| x.as_str()).collect::<Vec<&str>>();
                opts = utils::extract_txt_opts(args);
            }
            else {
                opts = TxtOpts{columns: 80, spacing: 1};
            }
            format::format_txt_file(filepath, opts, &args);
        },
        _ => println!("Unknown file type."),
    };
}
