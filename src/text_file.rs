use crate::parser;
use std::fs;

#[derive(Debug)]
pub struct TxtOpts {
    pub columns: u8,
    pub spacing: u8,
}

pub fn process_txt_file(contents: &str, _options: &TxtOpts) {
    println!("Processing text file...");
    let paragraphs = contents.split("\n\n").collect::<Vec<&str>>();
    for paragraph in paragraphs {
        dbg!(paragraph);
        /* 
        split each paragraph into words.
        rejoin into a single line.
        if the length is > max_columns:
            call split_at_final_whitespace() (not written yet).
        else:
            done; return the string and a placeholder.
        
        here I will have (&str, &str)...
        either the first and new lines, or the original string and "".

        add (at least) the first string to a result container.
        
        repeat for the new line if it is still too long (recursion?).
        */
    }
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