use crate::parser;
use std::fs;

#[derive(Debug)]
pub struct TxtOpts {
    pub columns: u8,
    pub spacing: u8,
}

pub fn find_txt_files(mut dir: String) -> Option<Vec<String>> {
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
                    let path = path.to_str().unwrap_or_default();
                    let contents = fs::read_to_string(&path).unwrap();
                    result.push(contents);
                }
            },
            Err(e) => {
                eprintln!("Error processing txt file: {}", e);
            }
        }
    }
    return Some(result);
}