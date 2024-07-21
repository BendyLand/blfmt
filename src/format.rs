use crate::{group, options, utils, utils::StringUtils, c_format, cpp_format, rs_format, txt_format};
use regex::Regex;
use std::fs::{self, File};
use std::io::Write;
use std::process::Command;

pub fn format_c_file(path: String) {
    let contents = fs::read_to_string(path.clone()).unwrap();
    let lines = contents.split("\n").collect::<Vec<&str>>();
    let mut sections = group::group_c_file_into_sections(lines);
    for i in 0..sections.len() {
        if sections[i].is_empty() {
            continue;
        }
        let original = &sections[i].clone().to_string();
        sections[i] = c_format::format_c_file_group(original.to_owned());
    }
    let result = c_format::join_c_file_groups(sections).trim_start().to_string();
    let ok = utils::write_file(path.clone(), result.as_bytes());
    match ok {
        Ok(_) => println!("Successfully wrote: {}", path),
        Err(e) => println!("Error during `format_c_file()`: {}", e),
    };
}

pub fn format_cpp_file(path: String) {
    let contents = fs::read_to_string(path.clone()).unwrap();
    let lines = contents.split("\n").collect::<Vec<&str>>();
    let mut sections = group::group_cpp_file_into_sections(lines);
    for i in 0..sections.len() {
        if sections[i].is_empty() {
            continue;
        }
        let original = &sections[i].clone().to_string();
        sections[i] = cpp_format::format_cpp_file_group(original.to_owned());
    }
    // c_file_groups should still work on cpp files
    let result = c_format::join_c_file_groups(sections).trim_start().to_string(); 
    let ok = utils::write_file(path.clone(), result.as_bytes());
    match ok {
        Ok(_) => println!("Successfully wrote: {}", path),
        Err(e) => println!("Error during `format_c_file()`: {}", e),
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

pub fn format_rs_file(path: String) {
    let contents = fs::read_to_string(path.clone()).unwrap();
    let lines = contents.split("\n").collect::<Vec<&str>>();
    let mut sections = group::group_rs_file_into_sections(lines);
    for i in 0..sections.len() {
        if sections[i].is_empty() {
            continue;
        }
        let original = &sections[i].clone().to_string();
        sections[i] = rs_format::format_rs_file_group(original.to_owned());
    }
}

pub fn format_txt_file(path: String, opts: options::TxtOpts, opt_titles: &[String]) {
    let path_clone = path.clone();
    let file_contents = fs::read_to_string(&path).unwrap();
    let result = String::new();
    let (cols, spacing) = (opts.columns, opts.spacing);
    let paragraphs = group::group_paragraphs(&file_contents, opt_titles);
    let mut result = Vec::<String>::new();
    for paragraph in paragraphs {
        let temp_para = txt_format::format_paragraph(paragraph, opts);
        result.push(temp_para);
    }
    let mut sep = "\n".to_string();
    for _ in 0..opts.spacing {
        sep += "\n";
    }
    let paragraphs = result.join(sep.as_str());
    let ok = utils::write_file(path, paragraphs.as_bytes());
    match ok {
        Ok(_) => println!("Successfully wrote: {}", path_clone),
        Err(e) => println!("Error during `format_txt_file()`: {}", e),
    };
}