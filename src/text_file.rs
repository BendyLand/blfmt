use crate::parser;
use std::fs;

#[derive(Debug, Clone, Copy)]
pub struct TxtOpts {
    pub columns: u8,
    pub spacing: u8,
}

pub fn process_txt_file(path: &String, opts: TxtOpts) -> String {
    let file = fs::read_to_string(path).unwrap_or_default();
    let lines = file.lines().collect::<Vec<&str>>();
    let mut new_file = String::new();
    for (i, line) in lines.iter().enumerate() {
        if line == &"" && i < lines.len() {
            new_file += "\n\n";
            continue;
        }
        let mut new_line = String::new();
        let words = line.split(" ").collect::<Vec<&str>>();
        for word in words {
            if new_line.len() + word.len() <= opts.columns as usize {
                new_line += (word.to_owned() + " ").as_str();
            }
            else {
                let newline = new_line + "\n";
                new_file += newline.as_str();
                new_line = "".to_string();
            }
        }
        let newline = new_line + "\n";
        new_file += newline.as_str();
    }
    return new_file;
}

pub fn find_txt_file_paths(mut dir: String) -> Option<Vec<String>> {
    println!("Finding txt files...");
    let mut result = Vec::new();
    if dir == "." { 
        dir = parser::expand_arg(".");
    }
    let read_dir_result = match fs::read_dir(&dir) {
        Ok(dir) => dir,
        Err(e) => {
            eprintln!("Failed to read directory: {}", e);
            return None;
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
                    let path = path.to_str().unwrap_or_default().to_string();
                    result.push(path);
                }
            },
            Err(e) => {
                eprintln!("Error processing txt file: {}", e);
            }
        }
    }
    return Some(result);
}