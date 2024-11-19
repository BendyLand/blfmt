use std::{fs, fs::File, io::Write, io::Error};
use regex::Regex;
use crate::options::{self, TxtOpts};
use std::ops::{Bound, RangeBounds};
use std::cmp::Ordering;

pub trait StringUtils {
    fn substring(&self, start: usize, len: usize) -> &str;
    fn slice(&self, range: impl RangeBounds<usize>) -> &str;
    fn peek_next(&self, current: usize) -> Option<char>;
    fn starts_with_any(&self, chars: Vec<char>) -> bool;
}

impl StringUtils for str {
    fn substring(&self, start: usize, len: usize) -> &str {
        let mut char_pos = 0;
        let mut byte_start = 0;
        let mut it = self.chars();
        loop {
            if char_pos == start { break; }
            if let Some(c) = it.next() {
                char_pos += 1;
                byte_start += c.len_utf8();
            }
            else { break; }
        }
        char_pos = 0;
        let mut byte_end = byte_start;
        loop {
            if char_pos == len { break; }
            if let Some(c) = it.next() {
                char_pos += 1;
                byte_end += c.len_utf8();
            }
            else { break; }
        }
        &self[byte_start..byte_end]
    }

    fn slice(&self, range: impl RangeBounds<usize>) -> &str {
        let start = match range.start_bound() {
            Bound::Included(bound) | Bound::Excluded(bound) => *bound,
            Bound::Unbounded => 0,
        };
        let len = match range.end_bound() {
            Bound::Included(bound) => *bound + 1,
            Bound::Excluded(bound) => *bound,
            Bound::Unbounded => self.len(),
        } - start;
        self.substring(start, len)
    }

    fn peek_next(&self, current: usize) -> Option<char> {
        if current < self.len()-1 {
            return self.chars().nth(current+1);
        }
        return None;
    }

    fn starts_with_any(&self, chars: Vec<char>) -> bool {
        for c in chars {
            if self.starts_with(c) {
                return true;
            }
        }
        return false;
    }
}

pub enum Style {
    Allman,
    KnR,
    Stroustrup,
}

pub fn format_else_lines(file: &mut String, style: Style) {
    match style {
        Style::KnR => return,
        Style::Allman => format_to_allman(file),
        Style::Stroustrup => format_to_stroustrup(file),
    }
}

fn format_to_allman(file: &mut String) {
    //todo: implement
}

fn format_to_stroustrup(file: &mut String) {
    let mut lines: Vec<String> = file.lines().into_iter().map(|x| x.to_string()).collect();
    for i in 0..lines.len() {
        if lines[i].contains("else") && lines[i].contains("}") {
            let idx = lines[i].find("else").unwrap();
            let indents = lines[i].chars().filter(|x| *x == '\t').count();
            lines[i].insert(idx-1, '\n');
            lines[i].remove(idx);
            for _ in 0..indents { lines[i].insert(idx, '\t'); }
        }
    }
    let result = lines.join("\n");
    *file = result;
}

pub fn close_empty_curly_brace_blocks(file: &mut String) {
    let mut lines: Vec<String> = file.lines().into_iter().map(|x| x.to_string()).collect();
    let mut lines_to_remove = Vec::<usize>::new();
    for i in 0..lines.len()-1 {
        if lines[i].ends_with("{") && lines[i+1].starts_with("}") {
            lines[i] = format!("{}}}", lines[i]);
            lines_to_remove.push(i+1);
        }
    }
    for n in lines_to_remove {
        lines.remove(n);
    }
    *file =  lines.join("\n");
}

pub fn sort_include_groups(file: String) -> String {
    let mut lines: Vec<String> = file.lines().map(|line| line.to_string()).collect();
    let mut result = Vec::new();
    let mut temp_group = Vec::new();
    for line in lines.into_iter() {
        if line.trim_start().starts_with("#include") {
            temp_group.push(line);
        }
        else {
            if !temp_group.is_empty() {
                sort_includes(&mut temp_group);
                // Add sorted group to the result
                result.extend(temp_group.drain(..));
            }
            result.push(line);
        }
    }
    if !temp_group.is_empty() {
        sort_includes(&mut temp_group);
        result.extend(temp_group.drain(..));
    }
    result.join("\n")
}

fn sort_includes(includes: &mut Vec<String>) {
    includes.sort_by(|a, b| {
        if a.contains("\"") && b.contains("\"") {
            if a < b { Ordering::Less }
            else if a > b { Ordering::Greater }
            else { Ordering::Equal }
        }
        else if a.contains(">") && b.contains(">") {
            if a < b { Ordering::Less }
            else if a > b { Ordering::Greater }
            else { Ordering::Equal }
        }
        else {
            if a.contains("\"") && !b.contains("\"") { Ordering::Greater }
            else if !a.contains("\"") && b.contains("\"") { Ordering::Less }
            else { Ordering::Equal }
        }
    });
}

pub fn add_all_leading_tabs(text: String) -> String {
    let lines: Vec<&str> = text.split("\n").collect();
    let mut temp_vec = Vec::<String>::new();
    for line in lines {
        let temp_str = format!("\t{}", line);
        temp_vec.push(temp_str);
    }
    return temp_vec.join("\n");
}

pub fn remove_dereference_spaces(line: String) -> String {
    let mut result = String::new();
    let mut skip = false;
    for (i, c) in line.char_indices() {
        if skip {
            skip = false;
            continue;
        }
        if i <= line.len()-2 {
            if c == '*' && line.chars().nth(i+1) == Some(' ') {
                skip = true;
            }
        }
        result += c.to_string().as_str();
    }
    return result;
}

pub fn remove_pointer_spaces(line: String) -> String {
    let mut result = String::new();
    for (i, c) in line.char_indices() {
        if i <= line.len()-2 && i > 0 {
            let skip = {
                c == ' ' &&
                line.chars().nth(i+1) == Some('*') &&
                (line.chars().nth(i-1).unwrap().is_alphanumeric() || line.chars().nth(i-1) == Some('*'))
            };
            if skip {
                continue;
            }
        }
        result += c.to_string().as_str();
    }
    return result;
}

pub fn remove_all_spaces(line: String) -> String {
    let chars: Vec<char> = line.chars().filter(|c| *c != ' ').collect();
    let mut result = String::new();
    for c in chars {
        result += c.to_string().as_str();
    }
    return result;
}

pub fn remove_object_constructor_space(line: String) -> String {
    let mut result = line.clone();
    let start = line.find("(").unwrap_or(line.len());
    if line.chars().nth(start-1) == Some(' ') {
        result.remove(start-1);
    }
    return result;
}

pub fn remove_unnecessary_spaces(line: String) -> String {
    let leading_tokens = vec!['(', '[', ' ', '!'];
    let ending_tokens = vec![')', '[', ']', ' ', ',', ';'];
    let mut result = String::new();
    let mut skip = false;
    for (i, c) in line.char_indices() {
        if skip {
            skip = false;
            continue;
        }
        if let Some(next) = line.peek_next(i) {
            if ending_tokens.contains(&next) {
                if c == ' ' { continue; }
            }
            if leading_tokens.contains(&c) {
                if next == ' ' { skip = true; }
            }
        }
        result += c.to_string().as_str();
    }
    return result.trim().to_string();
}

pub fn remove_whitespace_before_commas(line: String) -> String {
    let mut result = "".to_string();
    for (i, c) in line.chars().enumerate() {
        if i < line.len()-1 {
            if c == ' ' && line.chars().nth(i+1) == Some(',') { continue; }
        }
        result += c.to_string().as_str();
    }
    return result;
}

pub fn add_leading_whitespace(src: String, amount: usize) -> String {
    let mut prefix: String = "".to_string();
    for _ in 0..amount { prefix += " " };
    return format!("{}{}", prefix, src);
}

pub fn count_leading_whitespace(src: String, target: char) -> usize {
    let mut count = 0;
    for c in src.chars() {
        if c != target {
            break;
        }
        count += 1;
    }
    return count;
}

pub fn sanitize(input: String) -> String {
    let allowed_chars = Regex::new(r"[a-zA-Z0-9.\-_\+\=/\s]+").unwrap();
    let captures = allowed_chars.captures(&input).unwrap();
    let result = captures.get(0).unwrap().as_str().to_string();
    return result;
}

pub fn check_is_function_hoist(group: &String) -> bool {
    let re = Regex::new(r"^\s*\w+.*\)\s*;\s*(\n\s*\w+.*\)\s*;\s*)*$").unwrap();
    return re.is_match(&group);
}

pub fn starts_with_any(line: &String, opts: &Vec<String>) -> bool {
    for opt in opts {
        let words = &line.split(" ").collect::<Vec<&str>>();
        if line.trim().to_string().starts_with(opt) && (opt.len() == words[0].len()) {
            return true;
        }
    }
    return false;
}

pub fn extract_inner_header(line: String) -> String {
    let re = Regex::new(r"^.*\(.*\)").unwrap();
    let mut result = line.strip_suffix("{").unwrap_or(&line).to_string();
    let matches = re.find_iter(&result).map(|x| x.as_str().to_string()).collect::<Vec<String>>();
    if matches.len() > 0 {
        result = matches[0].clone();
    }
    return result;
}

fn handle_one_liner(line: String) -> String {
    let mut result = String::new();
    let idx1 = line.chars().position(|x| x == '{').unwrap();
    let idx2 = line.chars().position(|x| x == '}').unwrap();
    result += (line[0..idx1].to_string()+ "\n").as_str();
    result += ("{".to_string() + "\n").as_str();
    result += (line[idx1+1..idx2].trim_start().to_string() + "\n").as_str();
    result += ("}".to_string() + "\n").as_str();
    return result;
}

pub fn extract_c_function_header(group: &String) -> String {
    let lines = group.split("\n").map(|x| x.to_string()).collect::<Vec<String>>();
    let result;
    if lines[0].ends_with(")") {
        lines[0].clone()
    }
    else {
        let end = lines[0].chars().position(|x| x == ')').unwrap_or_else(|| {
            let length = lines[0].len();
            if length > 0 {
                length - 1
            }
            else {
                length
            }
        });
        let one_liner = lines[0].contains("{") && lines[0].contains("}");
        if one_liner {
            result = handle_one_liner(lines[0].to_string());
            return result;
        }
        if lines[0].len() > 0 {
            let temp = &lines[0][..end+1];
            temp.to_owned().to_string()
        }
        else {
            String::new()
        }
    }
}

fn line_has_no_alphanumeric(line: &str) -> bool {
    for c in line.chars() {
        if c.is_alphanumeric() {
            return false;
        }
    }
    return true;
}

pub fn remove_blank_lines(lines: Vec<&str>) -> String {
    let result: Vec<&str> = lines.into_iter().filter(|line| !line_has_no_alphanumeric(line) || line.contains("}")).collect();
    return result.join("\n");
}

pub fn write_file(path: &String, contents: &[u8]) -> Result<(), Error> {
    let mut dest = File::create(path).unwrap();
    let ok = dest.write_all(contents);
    return ok;
}

pub fn extract_txt_opts(args: Vec<&str>) -> options::TxtOpts {
    let start = args.iter().position(|x| x == &"-o" || x == &"--opts").unwrap() + 1;
    let remainder = args[start..].into_iter().map(|x| x.to_owned()).collect::<Vec<&str>>();
    let pattern = Regex::new(r"\s-\w").unwrap();
    let remainder_string = &remainder.join(" ");
    let matches = &pattern.find_iter(&remainder_string).map(|x| x.as_str()).collect::<Vec<&str>>();
    let end: usize;
    if matches.len() > 0 {
        let flag = remainder.iter().filter(|x| x.contains("-")).collect::<Vec<&&str>>()[0].to_owned();
        let start = &start.clone();
        end = args[*start..].iter().position(|x| x == &flag).unwrap() + start;
    }
    else {
        end = args.len();
    }
    let opts = args[start..end].into_iter().map(|s| s.parse::<usize>().expect("Error parsing number.")).collect::<Vec<usize>>();
    let result: TxtOpts;
    let cols = opts[0];
    let spacing = opts[1];
    result = TxtOpts{
        columns: cols,
        spacing: spacing
    };
    return result;
}

pub fn intersperse<T: Clone>(vec: Vec<T>, value: T) -> Vec<T> {
    if vec.is_empty() {
        return vec;
    }
    let mut result = Vec::<T>::new();
    for item in vec {
        result.push(item);
        result.push(value.clone());
    }
    result = result[..result.len()-1].into_iter().map(|x| x.to_owned()).collect::<Vec<T>>();
    return result;
}

pub fn median(lengths: &Vec<usize>) -> usize {
    let mut sorted_list = lengths.clone();
    sorted_list.sort();
    let idx = sorted_list.len() / 2;
    return sorted_list[idx];
}

pub fn check_for_even_line_length(lines: &Vec<&str>) -> bool {
    let line_lens = lines.iter().map(|x| x.len()).collect::<Vec<usize>>();
    let med = median(&line_lens);
    let check_val = (med as f32 * 0.8).ceil();
    let mut count = 0;
    for i in 0..line_lens.len() {
        if (line_lens[i] as f32) < check_val && i != 0 {
            count += 1;
        }
    }
    return count == 0;
}

pub fn ends_with_brace(line: &String) -> bool {
    return line.trim_end().ends_with("{");
}

pub fn infer_file_type(filepath: &String) -> String {
    let supported_types = get_file_extensions_list();
    for item in supported_types {
        if filepath.contains(&item) {
            return item.to_string();
        }
    }
    return "unknown".to_string();
}

pub fn get_file_extensions_list() -> Vec<String> {
    let exts = {
        vec![".cpp", ".c", ".go", ".rs", ".swift", ".txt", ".py"]
            .into_iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
    };
    return exts;
}

pub fn display_file_extensions() {
    let lines = get_file_extensions_list();
    for line in lines {
        println!("{}", line);
    }
}

pub fn check_valid_file_ext(path: &String) -> bool {
    let exts = get_file_extensions_list();
    for ext in exts {
        if path.contains(&ext) { return true; }
    }
    return false;
}

pub fn print_usage() {
    println!("USAGE:\nblfmt <file-path> <flags + options>");
}
