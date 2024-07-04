use std::process::Command;
use std::fs::{self, File};
use std::io::{Write};
use crate::{options, utils, group};

pub fn format_c_file(path: String) {
    let path_clone = path.clone();
    let lines = path.split("\n").collect:: <Vec<&str>>();
    let mut sections = group::group_c_file_into_sections(lines);
    for i in 0..sections.len() {
        let original = &sections[i].clone().to_string();
        sections[i] = format_c_file_group(original.to_owned());
    }
    let result = join_c_file_groups(sections);

    // let mut dest = File::create(path).unwrap();
    // let ok = dest.write_all(result); 
    // match ok {
    //     Ok(_) => println!("Successfully wrote: {}", path_clone),
    //     Err(e) => println!("Error writing file: {}", e),
    // }; 
}

fn join_c_file_groups(groups: Vec<String>) -> String {
    String::new()
}

fn format_c_file_group(group: String) -> String {
    String::new()
}

fn format_paragraph(paragraph: String, opts: options::TxtOpts) -> String {
    let mut result = String::new();
    let cols = opts.columns.clone();
    let words = paragraph.split_whitespace();
    let mut line = String::new();
    for word in words {
        if &line.len() + &word.len() > cols {
            result += (line.to_owned() + "\n").as_str();
            line = "".to_string();
        }
        line += (word.trim().to_string() + " ").as_str();
    }
    result += &line;
    return result;
}

pub fn format_txt_file(path: String, opts: options::TxtOpts, opt_titles: &[String]) {
    let path_clone = path.clone();
    let file_contents = fs::read_to_string(&path).unwrap();
    let result = String::new();
    let (cols, spacing) = (opts.columns, opts.spacing);
    let paragraphs = group::group_paragraphs(&file_contents, opt_titles);
    let mut result = Vec::<String>::new();
    for paragraph in paragraphs {
        let temp_para = format_paragraph(paragraph, opts);
        result.push(temp_para);
    }
    let mut sep = "\n".to_string();
    for _ in 0..opts.spacing {
        sep += "\n";
    }
    let paragraphs = result.join(sep.as_str());
    let mut dest = File::create(path).unwrap();
    let ok = dest.write_all(paragraphs.as_bytes()); 
    match ok {
        Ok(_) => println!("Successfully wrote: {}", path_clone),
        Err(e) => println!("Error writing file: {}", e),
    };
}

pub fn format_go_file(path: String) {
    let command = format!("gofmt -w {}", path);
    let res = Command::new("sh").arg("-c").arg(command).output().unwrap();
    let err = String::from_utf8(res.stderr).unwrap_or_default();
    if err.as_str() != "" {
        eprintln!("Error running gofmt:\n{}", err);
    }
    else {
        println!("Go file formatted successfully!");
    }
}