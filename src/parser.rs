use std::env;
use std::fs;

pub fn parse_args() -> Option<(String, String, Vec<String>)> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        return None;
    }
    else if args.len() == 2 {
        if &args[1] == "help" || &args[1] == "-h" {
            println!("Welcome to the blfmt help menu!\n");
            print_usage();
            println!("Valid file extensions:");
            display_file_extensions();
        }
        else if check_valid_file_extension(&args[2]) {
            println!("Please provide a valid directory path.");
            print_usage();
            return None;
        }
        else {
            println!("Please provide a valid file extension.");
            print_usage();
            return None;
        }
    }
    let path = args[1].clone();
    let valid_file_ext: bool = check_valid_file_extension(&args[2]);
    if valid_file_ext {
        let file_ext = &args[2];
        let opts = &args[2..].into_iter().map(|x| x.to_owned()).collect::<Vec<String>>();
        return Some((path, file_ext.to_owned(), opts.to_owned()));
    }
    else {
        return None;
    }
}

pub fn expand_arg(arg: &str) -> String {
    let result;
    match arg {
        "." => {
            match env::current_dir() {
                Ok(path) => {
                    result = path.to_str().unwrap_or_default().to_string()
                },
                Err(e) => {
                    eprintln!("Failed to get current directory: {}", e);
                    result = String::from("ERROR");
                },
            }
        },
        _ => result = arg.to_string(),
    }
    return result;
}


fn check_valid_file_extension(arg: &String) -> bool {
    let extensions = get_file_extensions_list();
    for extension in extensions {
        if arg == &extension {
            return true;
        }
    }
    return false;
}

fn get_file_extensions_list() -> Vec<String> {
    let file_result = fs::read_to_string("src/non-code/list-of-file-ext.txt");
    let file = match file_result {
        Ok(s) => s,
        Err(e) => {
            println!("Error reading file: {}", e);
            String::from("ERROR")
        }
    };
    let lines = {
        file
            .lines()
            .map(|line| line.to_string())
            .collect::<Vec<String>>()
    };
    return lines;
}

fn display_file_extensions() {
    let lines = get_file_extensions_list();
    for line in lines {
        println!("{}", line);
    }
}

fn print_usage() {
    println!("USAGE:\nblfmt [file-extension] [opt-flags] [dir-path]");
}