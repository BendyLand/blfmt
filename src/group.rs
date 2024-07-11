use regex::Regex;
use crate::utils;

pub fn group_rs_file_into_sections(lines: Vec<&str>) -> Vec<String> {
    let text = utils::remove_empty_lines(lines);
    let lines = text.split("\n").map(|x| x.to_string()).collect::<Vec<String>>();
    let sections = separate_rs_file_sections(lines);
    return sections;
}

fn separate_rs_file_sections(lines: Vec<String>) -> Vec<String> {
    let top_lines = {
        vec!["use", "mod", "#", "//"]
            .into_iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
    };
    let starts = {
        vec!["pub", "fn", "trait", "struct", "enum", "{", "}", "use", "mod", "#", "//"]
            .into_iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
    };
    let mut result = Vec::<String>::new();
    let mut is_group = false;
    let mut temp = String::new();
    for line in lines {
        if utils::starts_with_any(&line, &starts) {
            is_group = true;
            if line.starts_with("}") {
                temp += "}\n";
                result.push(temp.clone());
                temp = "".to_string();
                is_group = false;
                continue;
            }
        }
        if is_group {
            if !utils::starts_with_any(&line, &top_lines) {
                if utils::starts_with_any(&temp, &top_lines) {
                    result.push(temp.clone());
                    temp = "".to_string();
                }
            }
            temp += (line + "\n").as_str();
        }
    }
    let (one, two) = separate_attrs_from_libs(&result[0]);
    match two.clone() {
        s if s.len() > 0 => {
            result[0] = one;
            result.insert(1, two);
        },
        _ => (),
    };
    return result;
}

fn separate_attrs_from_libs(section: &String) -> (String, String) {
    let no_header = {
        !( section.starts_with("#")   || 
           section.starts_with("use") || 
           section.starts_with("mod") )
    };
    let mut result = Vec::<String>::new();
    if no_header { return (section.clone(), "".to_string()); }
    let lines = section.split("\n").map(|x| x.to_string()).collect::<Vec<String>>();
    let mut temp1 = String::new();
    let mut temp2 = String::new();
    let mut next_section = false;
    for line in lines {
        if (line.starts_with("use") || line.starts_with("mod")) && temp1.starts_with("#") {
            next_section = true;
        }
        if next_section {
            temp2 += (line + "\n").as_str();
        }
        else {
            temp1 += (line + "\n").as_str();
        }
    }
    return (temp1, temp2);
}

pub fn group_c_file_into_sections(lines: Vec<&str>) -> Vec<String> {
    let text = utils::remove_empty_lines(lines);
    let lines = text.split("\n").map(|x| x.to_string()).collect::<Vec<String>>();
    let result = separate_c_file_sections(lines.clone());
    return result;
}

fn separate_c_file_sections(lines: Vec<String>) -> Vec<String> {
    let mut temp = String::new();
    let mut result = Vec::<String>::new();
    let mut is_function_line: bool;
    let mut is_preprocessor_section: bool;
    let mut is_function_hoist_section: bool;
    let mut function_hoist_lines = 0;
    let mut preprocessor_lines = 0;
    for line in lines {
        is_function_line = Regex::new(r"^\w|\*+\s\w+\s*\(.*\).*").unwrap().is_match(&line);
        is_preprocessor_section = line.starts_with("#");
        is_function_hoist_section = is_function_line && line.trim_end().ends_with(";");
        if is_function_hoist_section {
            if function_hoist_lines == 0 {
                result.push(temp);
                temp = "".to_string();
            }
            function_hoist_lines += 1;
        }
        else if is_function_line { 
            result.push(temp);
            temp = "".to_string();
            temp += (line.to_string() + "\n").as_str();
            continue;
        }
        else if is_preprocessor_section {
            if preprocessor_lines == 0 {
                result.push(temp);
                temp = "".to_string();
            }
            preprocessor_lines += 1;
        }
        temp += (line.to_string() + "\n").as_str();
    }
    return result;
}

fn group_paragraph_by_titles(lines: Vec<&str>, titles: &[String]) -> Vec<String> {
    let mut result = Vec::<String>::new();
    let mut temp = String::new();
    for line in lines {
        let words = line.split_whitespace().collect::<Vec<&str>>();
        if words.len() > 0 {
            if titles.contains(&words[0].to_string()) {
                result.push(temp);
                temp = "".to_string();
            }
            temp += (line.to_owned() + "\n").as_str();
        }
    }
    result.push(temp);
    return result;
}

fn group_paragraphs_by_line_length(lines: Vec<&str>) -> Vec<String> {
    let line_lens = lines.iter().map(|line| line.len()).collect::<Vec<usize>>();
    let med = utils::median(&line_lens);
    let check_val = (med as f32 * 0.8).ceil();
    let mut result = Vec::<String>::new();
    let mut temp = String::new();
    for line in lines {
        if (line.len() as f32) < check_val {
            temp += line;
            result.push(temp.clone());
            temp = "".to_string();
        }
        else {
            temp += (line.to_owned() + "\n").as_str();
        }
    }
    result.push(temp);
    let final_res = result.into_iter().filter(|item| !item.is_empty()).collect::<Vec<String>>();
    return final_res;
}

pub fn group_paragraphs(text: &String, args: &[String]) -> Vec<String> {
    let result;
    let lines = text.split("\n").collect::<Vec<&str>>();
    if args.contains(&"-t".to_string()) || args.contains(&"--titles".to_string()) {
        result = group_paragraph_by_titles(lines, args);
    }
    else if utils::check_for_even_line_length(&lines) {
        result = lines.into_iter().map(|line| line.to_string()).collect::<Vec<String>>();
    }
    else {
        result = group_paragraphs_by_line_length(lines)
    }
    return result;
}