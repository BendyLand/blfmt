use regex::Regex;
use crate::utils;

fn separate_c_file_sections(lines: Vec<String>) -> String {
    let mut result = String::new();
    let mut inner_brackets = 0;
    let mut is_function_line: bool;
    for line in lines {
        is_function_line = Regex::new(r"^\w|\*+\s\w+\s*\(.*\).*").unwrap().is_match(&line);
        if is_function_line {
            result += "\n";
            result += (line.to_string() + "\n").as_str();
            continue;
        }
        result += (line.to_string() + "\n").as_str();
    }
    return result;
}

pub fn group_c_file_into_sections(lines: Vec<&str>) -> Vec<String> {
    let text = utils::remove_empty_lines(lines);
    let lines = text.split("\n").map(|x| x.to_string()).collect::<Vec<String>>();
    let new_text = separate_c_file_sections(lines.clone());
    let result = new_text.split("\n\n").map(|x| x.to_string()).collect::<Vec<String>>();
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