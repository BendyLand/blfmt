use crate::{group, options, utils, utils::StringUtils};
use regex::Regex;
use std::fs::{self, File};
use std::io::Write;
use std::process::Command;

pub fn format_rs_file(path: String) {
    let contents = fs::read_to_string(path.clone()).unwrap();
    let lines = contents.split("\n").collect::<Vec<&str>>();
    let mut sections = group::group_rs_file_into_sections(lines);
    for i in 0..sections.len() {
        if sections[i].is_empty() {
            continue;
        }
        let original = &sections[i].clone().to_string();
        sections[i] = format_rs_file_group(original.to_owned());
    }
}

fn format_rs_file_group(section: String) -> String {
    let top_lines = {
        vec!["#", "use", "mod", "//"]
            .into_iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
    };
    let mid_levels = {
        vec!["trait", "struct", "enum", "pub"]
            .into_iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
    };
    if utils::starts_with_any(&section, &top_lines) {
        // todo: order: use, mod, comments; then join by single space between them.
        println!("Top level statement!");
        println!("{}", &section);
    }
    else if utils::starts_with_any(&section, &mid_levels) {
        // todo: order: traits, enums, structs, public functions; format funcs; then join by single space between them.
        println!("Mid level statement!");
        println!("{}", &section);
    }
    else {
        // todo: format private functions.
        println!("Bottom level statement!");
        println!("{}", &section);
    }
    return section;
}

//todo: create format_inner_cpp_function()
pub fn format_cpp_file(path: String) {
    let contents = fs::read_to_string(path.clone()).unwrap();
    let lines = contents.split("\n").collect::<Vec<&str>>();
    let mut sections = group::group_cpp_file_into_sections(lines);
    for i in 0..sections.len() {
        if sections[i].is_empty() {
            continue;
        }
        let original = &sections[i].clone().to_string();
        sections[i] = format_cpp_file_group(original.to_owned());
    }
    // c_file_groups should still work on cpp files
    let result = join_c_file_groups(sections).trim_start().to_string(); 
    let ok = utils::write_file(path.clone(), result.as_bytes());
    match ok {
        Ok(_) => println!("Successfully wrote: {}", path),
        Err(e) => println!("Error during `format_c_file()`: {}", e),
    };
}

fn format_cpp_top_level_group(group: String) -> String {
    let mut result;
    let mut lines = group.split("\n").map(|x| x.to_string()).collect::<Vec<String>>();
    lines = lines.into_iter().filter(|x| !x.is_empty()).collect();
    lines.sort();
    result = lines.join("\n").to_string();
    result = swap_include_kind_locations(result);
    if result.contains("#") && result.contains("using") {
        lines = result.split("\n").map(|x| x.to_string()).collect::<Vec<String>>();
        result = "".to_string();
        for line in lines {
            if line.starts_with("using") && !result.contains("using"){
                result += "\n";
            }
            result += (line + "\n").as_str();
        }
    }
    return result;
}

fn format_cpp_file_group(group: String) -> String {
    let is_comment_group = group.starts_with("//") || group.starts_with("/*");
    if is_comment_group { return group; }
    let mut result: String;
    if group.starts_with("#") || group.starts_with("using") {
        result = format_cpp_top_level_group(group);
    }
    else {
        result = format_cpp_non_top_level_group(group);
    }
    return result;
}

fn format_cpp_non_top_level_group(group: String) -> String {
    let mut result = String::new();
    let mut in_function = false;
    let one_liners = {
        vec!["if", "else if", "else", "for", "while"]
            .into_iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
    };
    let mut lines = group.split("\n").map(|x| x.to_string()).collect::<Vec<String>>();
    let mut open_braces = 0;
    let mut skip = false;
    for (i, line) in lines.clone().into_iter().enumerate() {
        if skip { 
            skip = false;
            open_braces += 1;
            continue;
        }
        let line_clone = line.clone();
        if line_clone.contains("}") { open_braces -= 1; }
        let is_one_liner = {
            utils::starts_with_any(&line_clone, &one_liners) &&
            line_clone.trim_end().ends_with(";")
        };
        match i {
            0 => {
                if !line.ends_with(")") {
                    let idx = line.rfind(")").unwrap_or(line.len()-1);
                    let temp = line.substring(0, idx+1);
                    result += (temp.to_string() + "\n").as_str();
                }
                else {
                    result += (line + "\n").as_str();
                }
            },
            1 => {
                if line.trim_end().len() > 1 || !line.contains("{") {
                    result += "{\n";
                    let mut temp = line.clone().trim_end().to_string();
                    if utils::starts_with_any(&line, &one_liners) {
                        if !line.contains("{") && lines[i+1].contains("{") {
                            temp += " {";
                            skip = true;
                        }
                    }
                    result += (temp + "\n").as_str();
                }
                else {
                    result += (line + "\n").as_str();
                }
                in_function = true;
            },
            x if x == lines.len()-1 => result += "}",
            _ => {
                let mut temp = line.trim().to_string();
                if utils::starts_with_any(&temp.to_string(), &one_liners) && !is_one_liner {
                    if !line.contains("{") && lines[i+1].contains("{") {
                        temp += " {";
                        skip = true;
                    }
                }
                else if temp.starts_with("}") && temp.len() > 1 {
                    let temp2 = temp.clone();
                    let suffix = temp2.substring(1, temp.len()+1).trim();
                    temp = "}\n".to_string();
                    for _ in 0..open_braces { temp += "    "; }
                    temp += suffix;
                }
                let mut prefix = String::new();
                for _ in 0..open_braces { prefix += "    "; }
                if in_function && prefix.is_empty() { prefix += "    "; }
                let following_one_liner = {
                    utils::starts_with_any(&lines[i-1], &one_liners) &&
                    lines[i-1].ends_with(";")
                };
                // if following_one_liner { prefix += "    "; }
                let temp_str = prefix + temp.as_str();
                result += (temp_str + "\n").as_str();
            }
        }
        if is_one_liner { open_braces -= 1; }
        if line_clone.contains("{") { open_braces += 1; }
    }
    result = result.trim_end().to_string();
    return result;
}

fn swap_include_kind_locations(group: String) -> String {
    let lines = group.split("\n").collect::<Vec<&str>>();
    let mut lang_lines = String::new();
    let mut custom_lines = String::new();
    for line in lines {
        if line.contains("<") && !line.contains("//") {
            lang_lines += (line.to_string() + "\n").as_str();
        }
        else {
            custom_lines += (line.to_string() + "\n").as_str();
        }
    }
    let mut result = lang_lines + &custom_lines;
    result = result.trim_end().to_string();
    return result;
}

fn format_preprocessor_group(group: String) -> String {
    let mut result = String::new();
    let mut names = {
        group
            .split("\n")
            .map(|line| line.trim_start_matches("#include "))
            .collect::<Vec<&str>>()
    };
    names = names
        .into_iter()
        .filter(|x| !x.is_empty())
        .collect::<Vec<&str>>();
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
    return result;
}

pub fn format_c_file(path: String) {
    let contents = fs::read_to_string(path.clone()).unwrap();
    let lines = contents.split("\n").collect::<Vec<&str>>();
    let mut sections = group::group_c_file_into_sections(lines);
    for i in 0..sections.len() {
        if sections[i].is_empty() {
            continue;
        }
        let original = &sections[i].clone().to_string();
        sections[i] = format_c_file_group(original.to_owned());
    }
    let result = join_c_file_groups(sections).trim_start().to_string();
    let ok = utils::write_file(path.clone(), result.as_bytes());
    match ok {
        Ok(_) => println!("Successfully wrote: {}", path),
        Err(e) => println!("Error during `format_c_file()`: {}", e),
    };
}

fn format_c_file_group(group: String) -> String {
    let is_preprocessor = group.trim_start().starts_with("#include");
    let is_function_hoist = utils::check_is_function_hoist(&group);
    let mut result = String::new();
    if is_preprocessor {
        result = format_preprocessor_group(group);
    }
    else if is_function_hoist {
        result += (group + "\n").as_str();
    }
    else {
        result = normalize_c_function_group(group);
    }
    result = format_inner_curly_braces(result);
    result = indent_c_function_group(result);
    return result;
}

fn normalize_c_function_group(group: String) -> String {
    let temp_result = group.clone();
    let mut result = String::new();
    let mut lines = group
        .split("\n")
        .map(|x| x.to_string())
        .collect::<Vec<String>>();
    let header = utils::extract_c_function_header(&group);
    result += (header + "\n" + "{" + "\n").as_str();
    lines = lines[1..].into_iter().map(|x| x.to_owned()).collect::<Vec<String>>();
    for line in lines {
        if line == "{" { continue; }
        let temp = line.trim_start();
        result += (temp.to_string() + "\n").as_str();
    }
    if result.ends_with("\n{\n") {
        result = result.strip_suffix("\n{\n").unwrap_or(&result).to_string();
    }
    return result;
}

fn format_keyword_line(dest: &mut String, no_brace_layers: &mut usize,
                       line: &mut String, lines: &Vec<String>, i: usize) {
    *line = line.trim().to_string();
    let line_clone = line.clone();
    let header = utils::extract_inner_header(line.clone());
    if header.len() == line_clone.len() {
        if lines[i + 1].contains("{") { // trailing brace
            *dest += (line_clone.to_string() + " {\n").as_str();
        }
        else { // no brace
            *dest += (line_clone.to_string() + " {\n").as_str();
            *no_brace_layers += 1;
        }
    }
    else {
        if line_clone.contains("{") { // same-line brace
            *dest += (line_clone + "\n").as_str();
        }
        else { // one-liner
            *dest += (line_clone + "\n").as_str();
        }
    }
}

fn format_non_keyword_line(dest: &mut String, no_brace_layers: &mut usize,
                           line: &mut String, lines: &Vec<String>, i: usize) {
    if line.contains("}") && line.contains("{") {
        *dest += "}\n";
        let pos = &line.chars().position(|x| x == 'e').unwrap_or_default();
        let slc = &line.as_str()[pos.to_owned()..];
        *line = slc.to_owned().clone();
    }
    *dest += (line.to_string() + "\n").as_str();
    if *no_brace_layers > 0 {
        for layer in 0..*no_brace_layers {
            *dest += "}\n"
        }
    }
    *no_brace_layers = 0;
}

fn format_inner_curly_braces(group: String) -> String {
    let names = {
        vec!["for", "while", "if", "else if", "else", "switch"]
            .into_iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
    };
    let lines = group.split("\n").map(|x| x.to_string()).collect::<Vec<String>>();
    let lines_clone = lines.clone();
    let mut result = String::new();
    let mut no_brace_layers: usize = 0;
    for (i, mut line) in lines_clone.into_iter().enumerate() {
        if i > 1 && &line == &"{" {
            continue;
        }
        if utils::starts_with_any(&line, &names.clone()) {
            format_keyword_line(&mut result, &mut no_brace_layers, &mut line, &lines, i);
        }
        else {
            format_non_keyword_line(&mut result, &mut no_brace_layers, &mut line, &lines, i);
        }
    }
    return result;
}

fn format_function_group(dest: &mut String, lines: &Vec<String>, i: usize,
                         inner_group: &mut bool, indent: &mut i8, line: &String) {
    if i > 2 && i < lines.len() - 1 && lines[i - 1].contains("{") {
        *inner_group = true;
        *indent += 1;
    }
    if line.contains("}") {
        *indent -= 1;
        if *indent == 0 {
            *inner_group = false;
        }
    }
    if *inner_group {
        for _ in 0..*indent {
            *dest += "    ";
        }
    }
    *dest += (line.to_string() + "\n").as_str();
}

fn indent_c_function_group(group: String) -> String {
    if group.contains("#include") { return group; }
    let is_function_hoist_group = {
        let lines = group.split("\n").collect::<Vec<&str>>();
        let re = Regex::new(r"^.*\(.*\);").unwrap();
        if re.is_match(&lines[0]) {
            let result = (group + "\n").to_string();
            return result;
        }
    };
    let mut result = String::new();
    let lines = group.split("\n").map(|x| x.to_string()).collect::<Vec<String>>();
    if lines[0].ends_with(";") { return group; }
    let lines_clone = lines.clone();
    let mut inner_group = false;
    let mut indent = 0;
    for (i, line) in lines.into_iter().enumerate() {
        if i > 1 && i < lines_clone.len() - 1 {
            format_function_group(&mut result,&lines_clone, i, &mut inner_group, &mut indent, &line);
        }
        else {
            result += (line.to_string() + "\n").as_str();
        }
    }
    result = indent_c_function(result);
    return result;
}

fn indent_c_function(group: String) -> String {
    let mut result = String::new();
    let is_function_hoist = &group.trim_end().ends_with(";");
    if *is_function_hoist {
        return group.trim().to_string();
    }
    let lines = group.split("\n").collect::<Vec<&str>>();
    let lines = utils::remove_empty_lines(lines).split("\n").map(|x| x.to_string()).collect::<Vec<String>>();
    for i in 0..2 {
        result += (lines[i].clone() + "\n").as_str();
    }
    let length = lines.len();
    for i in 2..length - 2 {
        result += ("    ".to_string() + lines[i].clone().as_str() + "\n").as_str();
    }
    result += "}\n";
    return result;
}

fn join_c_file_groups(groups: Vec<String>) -> String {
    let mut result;
    let mut temp = Vec::<String>::new();
    for group in groups {
        let temp_group = group.trim_end();
        temp.push(temp_group.to_string() + "\n");
    }
    result = temp.join("\n");
    result = (result.trim_end().to_string() + "\n").to_string();
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
        Err(e) => println!("Error during `format_txt_file()`: {}", e),
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
