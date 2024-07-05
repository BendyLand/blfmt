use std::process::Command;
use std::fs::{self, File};
use std::io::{Write};
use crate::{options, utils, group};

pub fn format_c_file(path: String) {
    let path_clone = path.clone();
    let contents = fs::read_to_string(path).unwrap();
    let lines = contents.split("\n").collect::<Vec<&str>>();
    let mut sections = group::group_c_file_into_sections(lines);
    for i in 0..sections.len() {
        if sections[i].is_empty() {
            continue;
        }
        let original = &sections[i].clone().to_string();
        sections[i] = format_c_file_group(original.to_owned());
    }
    for section in sections {
        print!("{}", section);
    }
    // let result = join_c_file_groups(sections);

    // let ok = utils::write_file(path, result.as_bytes());
    // match ok {
    //     Ok(_) => println!("Successfully wrote: {}", path_clone),
    //     Err(e) => println!("Error writing file: {}", e),
    // };
}

fn join_c_file_groups(groups: Vec<String>) -> String {
    String::new()
}

fn swap_include_kind_locations(group: String) -> String {
    let lines = group.split("\n").collect::<Vec<&str>>();
    let mut lang_lines = String::new();
    let mut custom_lines = String::new();
    for line in lines {
        if line.contains("<") {
            lang_lines += (line.to_string() + "\n").as_str();
        }
        else {
            custom_lines += (line.to_string() + "\n").as_str();
        }
    }
    let mut result = lang_lines + &custom_lines;
    result = result.trim_end().to_string() + "\n";
    return result;
}

fn normalize_c_function_group(group: String) -> String {
    let temp_result = group.clone();
    let mut result = String::new();
    let mut lines = group.split("\n").map(|x| x.to_string()).collect::<Vec<String>>();
    let header = utils::extract_c_function_header(&group);
    result += (header + "\n" + "{" + "\n").as_str();
    lines = lines[1..].into_iter().map(|x| x.to_owned()).collect::<Vec<String>>();
    for line in lines {
        if line == "{" {
            continue;
        }
        let temp = line.trim_start();
        result += (temp.to_string() + "\n").as_str(); 
    }
    return result;
}

fn indent_c_function_group(group: String) -> String {
    if group.contains("#include") { return group; }
    let res = group.clone();
    let mut result = String::new();
    let lines = group.split("\n").map(|x| x.to_string()).collect::<Vec<String>>();
    let lines_clone = lines.clone();
    let mut inner_group = false;
    let mut indent = 0;
    for (i, line) in lines.into_iter().enumerate() {
        if i > 1 && i < lines_clone.len()-2 {
            if i > 2 && i < lines_clone.len()-1 && lines_clone[i-1].contains("{") {
                inner_group = true;
                indent += 1;
            }
            if line.contains("}") {
                indent -= 1;
                if indent == 0 {
                    inner_group = false;
                }
            }
            if inner_group {
                for _ in 0..indent {
                    result += "    ";
                }
            }
            result += ("    ".to_string() + &line + "\n").as_str();
        }
        else {
            result += (line.to_string() + "\n").as_str();
        }
    }
    return result;
}

fn format_curly_braces(group: String) -> String {
    group
}

fn format_c_file_group(group: String) -> String {
    let is_preprocessor = group.starts_with("#include");
    let mut result = String::new();
    if is_preprocessor {
        let mut names = {
            group
                .split("\n")
                .map(|line| line.trim_start_matches("#include "))
                .collect::<Vec<&str>>()
        };
        names.sort_by(|a, b| {
            let a_slice = &a[1..];
            let b_slice = &b[1..];
            a_slice.cmp(&b_slice)
        });
        for name in names {
            let temp = "#include ".to_string() + name;
            result += (temp.to_string() + "\n").as_str();
        }
        result = swap_include_kind_locations(result);
    }
    else {
        result = normalize_c_function_group(group);
    }
    result = indent_c_function_group(result);
    result = format_curly_braces(result);
    return result;
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
    let ok = utils::write_file(path, paragraphs.as_bytes());
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