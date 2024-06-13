use crate::{groups, parser};
use std::fs;

#[derive(Debug, Clone, Copy)]
pub struct TxtOpts {
    pub columns: u8,
    pub spacing: u8,
}

pub fn process_txt_file(path: &String, opts: TxtOpts) -> String {
    let file = fs::read_to_string(path).unwrap_or_default();
    let lines = file.split("\n").collect::<Vec<&str>>();
    let mut temp = String::new();
    let mut result = Vec::<String>::new();
    for line in lines {
        let words = line.split_whitespace().collect::<Vec<&str>>();
        for word in words {
            if temp.len() + word.len() > opts.columns as usize {
                result.push(temp.trim_end().to_owned());
                temp.clear();
            }
            temp += (word.to_owned() + " ").as_str();
        }
        if !temp.is_empty() {
            result.push(temp.clone());
            temp = "".to_string();
        }
    }
    result.push(temp);
    result = result.into_iter().filter(|item| !item.is_empty()).collect::<Vec<String>>();
    let mut result_str = result.join("\n").to_string();
    let options = &[opts.columns.to_string(), opts.spacing.to_string()];
    let paragraphs = groups::group_paragraphs(&result_str, &options[..]);
    let mut sep = String::from("\n");
    for _ in 0..opts.spacing {
        sep.push_str("\n");
    }
    result_str = paragraphs.join(&sep.as_str());
    return result_str;
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