use std::env;
use std::fs;

// const TXT_FILE_COLUMNS: u8 = 80;
// const LINE_BREAKS: u8 = 1;

pub fn parse_args() -> (Option<String>, Option<String>) {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        return (None, None);
    }
    else if args.len() == 2 {
        if &args[1] == "help" || &args[1] == "-h" {
            println!("Welcome to the icfmt help menu!\n");
            print_usage();
            println!("Valid file extensions:");
            display_file_extensions();
        }
        else if check_valid_file_extension(&args[1].to_string()) {
            println!("Please provide a valid directory path.");
            print_usage();
            return (None, None);
        }
        else {
            println!("Please provide a valid file extension.");
            print_usage();
            return (None, None);
        }
    }
    let path = args[args.len()-1].clone();
    let valid_file_ext: bool = check_valid_file_extension(&args[1]);
    if valid_file_ext {
        let file_ext = &args[1];
        return (Some(path), Some(file_ext.to_owned()));
    }
    else {
        return (None, None)
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
    let file_result = fs::read_to_string("src/file-extensions.txt");
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
    println!("USAGE:\nicfmt [file-extension] [opt-flags] [dir-path]");
}