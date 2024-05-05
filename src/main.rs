use std::env;
use std::fs;

/* 
Usage:

icfmt [extension] [opt-flags] [path]
*/

fn do_something() {} // placeholder fn

fn main() {
    parse_args();
}

fn parse_args() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 { 
        println!("USAGE:\nicfmt [target file-extension] [opt-flags] [path]");
        return; 
    }
    else if args.len() == 2 {
        if &args[1] == "help" || &args[1] == "-h" {
            println!("USAGE:\nicfmt [target file-extension] [opt-flags] [path]\n");
            println!("Valid file extensions:");
            display_file_extensions();
        }
        else if check_valid_file_extension(&args[1].to_string()) {
            println!("Please provide a valid directory path.");
            return;
        }
    }
    for i in 0..args.len() {
        if i > 0 {
            match args[i].as_str() {
                "txt" => format_txt_files(),
                _ => do_something(), 
            }
        }
    }
}

fn get_current_dir() -> String {
    let path_buf = env::current_dir();
    let path = match path_buf {
        Ok(ok_path) => {
            ok_path
                .to_str()
                .map(|s| s.to_string())
                .unwrap()
        },
        Err(e) => {
            println!("There was a problem getting the path: {}", e);
            String::from("PATH-ERROR")
        },
    };
    return path;
}

fn format_txt_files() {
    let current_dir = get_current_dir(); // match with PATH-ERROR to check for success
    dbg!(current_dir);
}

fn check_valid_file_extension(arg: &String) -> bool {
    let extensions = get_valid_file_extensions();
    for extension in extensions {
        if arg == &extension {
            return true;
        }
    }
    return false;
}

fn get_valid_file_extensions() -> Vec<String> {
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
    let lines = get_valid_file_extensions();
    for line in lines {
        println!("{}", line);
    }
}