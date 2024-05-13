use std::env;
use std::fs;

const TXT_FILE_COLUMNS: u8 = 80;
// const LINE_BREAKS: u8 = 1;

pub fn parse_args() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("USAGE:\nicfmt [file-extension] [opt-flags] [dir-path]");
        return;
    }
    else if args.len() == 2 {
        if &args[1] == "help" || &args[1] == "-h" {
            println!("Welcome to the icfmt help menu!\n");
            println!("USAGE:\nicfmt [file-extension] [opt-flags] [dir-path]\n");
            println!("Valid file extensions:");
            display_file_extensions();
        }
        else if check_valid_file_extension(&args[1].to_string()) {
            println!("Please provide a valid directory path.");
            println!("USAGE:\nicfmt [file-extension] [opt-flags] [dir-path]");
            return;
        }
        else {
            println!("Please provide a valid file extension.");
            println!("USAGE:\nicfmt [file-extension] [opt-flags] [dir-path]");
            return;
        }
    }
    let path = args[args.len()-1].clone();
    if args[1].as_str() == "txt" {
        find_txt_files(TXT_FILE_COLUMNS, path);
    }
}

fn expand_arg(arg: &str) -> String {
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

fn find_txt_files(columns: u8, mut dir: String) {
    if dir == "." { 
        dir = expand_arg(".");
    }
    let read_dir_result = match fs::read_dir(&dir) {
        Ok(dir) => dir,
        Err(e) => {
            eprintln!("Failed to read directory: {}", e);
            return;
        },
    };
    for entry in read_dir_result {
        match entry {
            Ok(entry) => {
                let path = entry.path();
                let is_txt_file: bool = {
                    path
                        .extension()
                        .map(|s| s.to_str().unwrap_or_default()) == Some("txt")
                };
                if path.is_file() && is_txt_file {
                    let path = path.to_str().unwrap_or_default();
                    process_txt_file(columns, path);
                }
            },
            Err(e) => {
                eprintln!("Error processing txt file: {}", e);
            }
        }
    }
}

fn process_txt_file(columns: u8, path: &str) {
    println!("Processing txt file '{}' into {} columns...", path, columns);
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