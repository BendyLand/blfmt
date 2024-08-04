use crate::{c_format, utils, utils::StringUtils};

pub fn format_cpp_file_group(group: String) -> String {
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

fn format_cpp_top_level_group(group: String) -> String {
    let mut result;
    let mut lines = group.split("\n").map(|x| x.to_string()).collect::<Vec<String>>();
    lines = lines.into_iter().filter(|x| !x.is_empty()).collect();
    lines.sort();
    result = lines.join("\n").to_string();
    result = c_format::swap_include_kind_locations(result);
    if result.contains("#") && ((result.contains("using") || result.contains(";"))) {
        lines = result.split("\n").map(|x| x.to_string()).collect::<Vec<String>>();
        result = "".to_string();
        for line in lines {
            if line.starts_with("using") && !result.contains("using"){
                result += "\n";
            }
            else if line.chars().nth(0).unwrap_or(' ') != ' ' && line.ends_with(";"){
                result += "\n"
            }
            result += (line + "\n").as_str();
        }
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
        if line_clone.trim_end().ends_with("}") { open_braces -= 1; }
        let is_one_liner = {
            utils::starts_with_any(&line_clone, &one_liners) &&
            line_clone.trim_end().ends_with(";")
        };
        match i {
            0 => {
                let brk = handle_fn_def_line(&line, &mut result);
                if brk { break; }
            },
            1 => {
                let (result, rtn) = handle_fn_second_line(&line, &mut result, &one_liners, &lines, &i, &mut skip, &mut in_function);
                if rtn { return result; }
            },
            x if x == lines.len()-1 => result += "}",
            _ => {
                handle_fn_inside(&line, &mut result, &one_liners, &lines, &i, &is_one_liner, &mut skip, &mut open_braces, &in_function);
            }
        }
        if line_clone.trim_end().ends_with("{") { open_braces += 1; }
    }
    result = result.trim_end().to_string();
    return result;
}

fn handle_fn_inside(line: &String, result: &mut String, one_liners: &Vec<String>, lines: &Vec<String>, i: &usize, is_one_liner: &bool, skip: &mut bool, open_braces: &mut i32, in_function: &bool) {
    let mut temp = line.trim().to_string();
    if utils::starts_with_any(&temp.to_string(), &one_liners) && !is_one_liner {
        if !line.contains("{") && lines[i+1].contains("{") {
            temp += " {";
            *skip = true;
        }
    }
    else if temp.starts_with("}") && temp.len() > 1 {
        let temp2 = temp.clone();
        let suffix = temp2.substring(1, temp.len()+1).trim();
        temp = "}\n".to_string();
        *open_braces -= 1;
        for _ in 0..*open_braces { temp += "    "; }
        temp += suffix;
    }
    let mut prefix = String::new();
    let is_case = {
        line.trim_start().starts_with("case") || 
        line.trim_start().starts_with("default")
    };
    if is_case { *open_braces -= 1; }
    for _ in 0..*open_braces { prefix += "    "; }
    if is_case { *open_braces += 1; }
    if *in_function && prefix.is_empty() { prefix += "    "; }
    let temp_str = prefix + temp.as_str();
    *result += (temp_str + "\n").as_str();
}

fn handle_fn_def_line(line: &String, result: &mut String) -> bool {
    if !line.ends_with(")") {
        let idx = line.rfind(")").unwrap_or(line.len()-1);
        let temp = line.substring(0, idx+1);
        if line.ends_with("{}") {
            *result += temp;
            *result += "\n{}";
            return true;
        }
        *result += (temp.to_string() + "\n").as_str();
    }
    else {
        *result += (line.to_owned() + "\n").as_str();
    }
    return false;
}

fn handle_fn_second_line(line: &String, result: &mut String, one_liners: &Vec<String>, lines: &Vec<String>, i: &usize, skip: &mut bool, in_function: &mut bool) -> (String, bool) {
    if line.trim_end().len() > 1 || !line.contains("{") {
        if line.trim() == "{}" {
            *result += "{}\n";
            return (result.to_owned(), true);
        }
        *result += "{\n";
        let mut temp = line.clone().trim_end().to_string();
        if utils::starts_with_any(&line, one_liners) {
            if !line.contains("{") && lines[i+1].contains("{") {
                temp += " {";
                *skip = true;
            }
        }
        *result += (temp + "\n").as_str();
    }
    else {
        *result += (line.to_owned() + "\n").as_str();
    }
    *in_function = true;
    return (result.clone().to_string(), false);
}

