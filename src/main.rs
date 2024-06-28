#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

mod parser;
mod processing;
mod grouping;
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
    if utils::check_valid_file_extension(&filepath) {
        //todo: main parsing logic
        println!("I'm in the main parsing logic!");
    }
    else {
        println!("Invalid file extension provided.");
    }
    // dbg!(std::env::current_dir());
}
