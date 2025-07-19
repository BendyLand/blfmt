use std::{fs, fs::File, io::Write, io::Error};
use regex::Regex;
use crate::options::{self, TxtOpts};
use std::ops::{Bound, RangeBounds};
use std::cmp::Ordering;
use std::collections::HashMap;

pub trait StringUtils {
    fn substring(&self, start: usize, len: usize) -> &str;
    fn at(&self, current: usize) -> Option<char>;
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

    fn at(&self, current: usize) -> Option<char> {
        if current < self.len() {
            return self.chars().nth(current);
        }
        return None;
    }
}

pub enum Style {
    Allman,
    KnR,
    Stroustrup,
}

pub fn format_else_lines(file: &mut String, style: &Style) {
    match style {
        Style::KnR => return,
        Style::Allman => format_to_allman(file),
        Style::Stroustrup => format_to_stroustrup(file),
    }
}

fn format_to_allman(file: &mut String) {
    let mut lines: Vec<String> = file.lines().map(|x| x.to_string()).collect();
    let pattern = Regex::new("^.*[^\\\"]\\belse\\b[^\\\"].*$").unwrap();
    let catch_pattern = Regex::new("^.*[^\\\"]\\bcatch\\b[^\\\"].*$").unwrap();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i].trim_start();
        // Skip comments
        if line.starts_with("//") || line.starts_with("/*") {
            i += 1;
            continue;
        }
        // Handle `else`
        if pattern.is_match(&lines[i]) {
            if let Some(idx) = lines[i].find("else") {
                let indents = lines[i].chars().take_while(|&c| c == '\t').count();
                lines[i].insert(idx, '\n');
                for _ in 0..indents {
                    lines[i].insert(idx + 1, '\t');
                }
            }
        }
        // Handle `catch`
        else if catch_pattern.is_match(&lines[i]) {
            if let Some(idx) = lines[i].find("catch") {
                let indents = lines[i].chars().take_while(|&c| c == '\t').count();
                lines[i].insert(idx, '\n');
                for _ in 0..indents {
                    lines[i].insert(idx + 1, '\t');
                }
            }
        }
        // Handle opening braces `{` on the same line as code (Allman conversion)
        if let Some(pos) = lines[i].find('{') {
            // Make sure it's not the only thing on the line already
            if lines[i].trim() != "{" {
                let indent = lines[i].chars().take_while(|&c| c == '\t').collect::<String>();
                let before = lines[i][..pos].trim_end().to_string();
                let after = &lines[i][pos + 1..].to_string().clone(); // Skip over the brace
                lines[i] = before;
                lines.insert(i + 1, format!("{}{{{}", indent, after));
                i += 1; // skip over the inserted line
            }
        }
        i += 1;
    }
    let result = lines.join("\n");
    *file = result;
}

fn format_to_stroustrup(file: &mut String) {
    let mut lines: Vec<String> = file.lines().into_iter().map(|x| x.to_string()).collect();
    let pattern = Regex::new("^.*[^\\\"]\\belse\\b[^\\\"].*$").unwrap();
    let catch_pattern = Regex::new("^.*[^\\\"]\\bcatch\\b[^\\\"].*$").unwrap();
    for i in 0..lines.len() {
        if lines[i].trim_start().starts_with("//") || lines[i].trim_start().starts_with("/*") { continue; }
        if pattern.is_match(&lines[i]) {
            let idx = lines[i].find("else").unwrap();
            let indents = lines[i].chars().filter(|x| *x == '\t').count();
            lines[i].insert(idx-1, '\n');
            lines[i].remove(idx);
            for _ in 0..indents { lines[i].insert(idx, '\t'); }
        }
        else if catch_pattern.is_match(&lines[i]) {
            let idx = lines[i].find("catch").unwrap();
            let indents = lines[i].chars().filter(|x| *x == '\t').count();
            lines[i].insert(idx-1, '\n');
            lines[i].remove(idx);
            for _ in 0..indents { lines[i].insert(idx, '\t'); }
        }
    }
    let result = lines.join("\n");
    *file = result;
}

pub fn tidy_up_loose_ends(file: &mut String, style: Style) {
    let mut lines: Vec<String> = file.lines().into_iter().map(|x| x.to_string()).collect();
    let mut lines_clone: Vec<String> = lines.clone();
    for (i, line) in lines_clone.into_iter().enumerate() {
        if line.contains(",") {
            lines[i] = ensure_space_after_char(&line, ',');
        }
    }
    *file = lines.join("\n");
    // format_long_lines(file, 100);
    // add_blank_lines_back(file, lines_before_blank_lines);
    remove_blank_lines_from_blocks(file);
    ensure_no_consecutive_blank_lines(file);
    ensure_proper_doc_comment_spacing(file);
    fix_indentation_levels(file);
    shift_back_preproc_lines(file);
    join_single_line_constructs(file, style);
}

pub fn detect_indentation(line: &String) -> usize {
    let mut result = 0;
    for c in line.chars() {
        if c != '\t' { break; }
        result += 1;
    }
    return result;
}

fn fix_indentation(line: &String, current: usize, target: usize) -> String {
    let mut prefix = String::new();
    let result: String;
    if current < target {
        for _ in current..target {
            prefix += "\t";
        }        
        result = format!("{}{}", prefix, line);
    }
    else {
        let mut temp = line.clone();
        for _ in target..current {
            temp = remove_single_tab(line);
        }
        result = temp;
    }
    return result;
}

fn fix_indentation_levels(file: &mut String) {
    let lines: Vec<String> = file.lines().map(|x| x.to_string()).collect();
    let mut result = Vec::<String>::new();
    let mut indent_level = 0;
    for (i, line) in lines.clone().into_iter().enumerate() {
        if line == "}" || line.trim_start().starts_with("}") {
            indent_level -= 1;
        }
        let current_level = detect_indentation(&line);
        if current_level != indent_level {
            let temp = fix_indentation(&line, current_level, indent_level);
            result.push(temp);
        }
        else {
            let temp = line.clone();
            result.push(temp);
        }
        if line == "{" || line.ends_with("{") {
            indent_level += 1;
        }
    }
    *file = result.join("\n");
}

fn line_is_function_def(line: &String, pattern: &regex::Regex) -> bool {
    return pattern.is_match(line)
}

fn line_is_doc_comment(line: &String) -> bool {
    return line.trim_start().starts_with("*") && line.contains("*/");
}

fn find_next_function_def(remaining_lines: &[String], pattern: &regex::Regex) -> i32 {
    for (i, line) in remaining_lines.into_iter().enumerate() {
        if line_is_function_def(line, &pattern) {
            return i as i32;
        }
    }
    return -1;
}

fn find_next_doc_comment(remaining_lines: &[String]) -> i32 {
    for (i, line) in remaining_lines.into_iter().enumerate() {
        if line.trim_start().starts_with("*") && line.contains("*/") {
            return i as i32;
        }
    }
    return -1;
}

fn ensure_proper_doc_comment_spacing(file: &mut String) {
    let mut lines: Vec<String> = file.trim().lines().map(|x| x.to_string()).collect();
    let mut result = Vec::<String>::new();
    let pattern = Regex::new("^(\\w*\\s*)*\\w+\\s+\\w+\\(.*\\)\\s*;?\\s*$").unwrap();
    let mut lines_to_remove = Vec::<usize>::new();
    for (i, line) in lines.clone().into_iter().enumerate() {
        if i > 0 && i < lines.len()-1 {
            if line.trim().is_empty() {
                if line_is_doc_comment(lines.iter().nth(i-1).unwrap()) &&
                   line_is_function_def(lines.iter().nth(i+1).unwrap(), &pattern) {
                   continue;
               }
            }
        }
        result.push(line);
    }
    *file = result.join("\n").trim_end().to_string() + "\n";
}

pub fn ensure_space_after_char(line: &String, target: char) -> String {
    let mut result = String::new();
    for (i, c) in line.char_indices() {
        result += c.to_string().as_str();
        if i < line.len()-1 {
            if c == target && line.chars().nth(i+1) != Some(' ') && line.chars().nth(i+1) != Some(target) {
                result += " ";
            }
        }
    }
    return result;
}


fn remove_blank_lines_from_blocks(file: &mut String) {
    let mut lines: Vec<String> = file.trim().lines().map(|x| x.to_string()).collect();
    let mut result = Vec::<String>::new();
    let mut in_block = false;
    for (i, line) in lines.clone().into_iter().enumerate() {
        let at_block_end = in_block && (line == "}".to_string() || line == "};".to_string());
        if !in_block && line == "{".to_string() { in_block = true; }
        else if at_block_end { in_block = false; }
        if in_block && !check_line_is_blank(&line) { result.push(line.clone()); }
        else if !in_block { result.push(line.clone()); }
    }
    *file = result.join("\n").trim_end().to_string() + "\n";
}

fn ensure_no_consecutive_blank_lines(file: &mut String) {
    let mut lines: Vec<String> = file.trim().lines().map(|x| x.to_string()).collect();
    let mut result = Vec::<String>::new();
    for (i, line) in lines.clone().into_iter().enumerate() {
        if i < lines.len()-1 {
            let line_empty = check_line_is_blank(&line);
            if line_empty {
                if !check_line_is_blank(&result[result.len()-1]) {
                    result.push(line.clone());
                }
                continue;
            }
        }
        result.push(line);
    }
    *file = result.join("\n").trim_end().to_string() + "\n";
}

//TODO: make a function that scans for the specific constructs that get aggressively formatted (e.g., maps)
pub fn scan_for_lines_before_blank_lines(file: String) -> Vec<(usize, String)> {
    let mut result = Vec::<(usize, String)>::new();
    let mut lines: Vec<String> = file.lines().map(|x| x.to_string()).collect();
    let mut open_braces = 0;
    for (i, line) in lines.clone().into_iter().enumerate() {
        // only add lines that are inside of blocks
        if line.contains("{") { open_braces += 1; }
        if line.contains("}") { open_braces -= 1; }
        if open_braces > 0 {
            if i < lines.len()-1  {
                let has_no_visible_chars = {
                    lines[i+1]
                        .chars()
                        .all(|x| x.is_whitespace() || x == '\t')
                };
                if lines[i+1].is_empty() || has_no_visible_chars {
                    result.push((i+1, line));
                }
            }
        }
    }
    return result;
}

fn spaces_to_tabs(line: &String) -> String {
    return line.replace("    ", "\t");
}

fn format_long_lines(file: &mut String, target_len: usize) {
    let mut lines: Vec<String> = file.lines().map(|x| x.to_string()).collect();
    let mut temp = String::new();
    for (i, entry) in lines.clone().iter().enumerate() {
        if entry.len() > target_len {
            if entry.chars().filter(|x| *x == '\"').count() == 2 && !entry.contains("{") {
                lines[i] = format_long_string_line(&entry, target_len);
            }
        }
    }
    *file = lines.join("\n");
}

//TODO: create functions/algorithms to format specific kinds of long lines.
//this kind of almost works for certain cases
fn format_long_string_line(line: &String, target_len: usize) -> String {
    let mut result = line.clone();
    if let Some(idx) = line.find("\"") {
        let indents = count_leading_chars(&line, '\t');
        let mut s = "\n".to_string();
        for _ in 0..indents+1 { s.push('\t'); }
        result.insert_str(idx-1, s.as_str());
        let r_idx = result.rfind("\"").unwrap();
        if r_idx != idx && line.chars().nth(r_idx) == Some(')') {
            s = s[..s.len()-1].to_string();
            result.insert_str(r_idx+1, s.as_str());
        }
    }
    return result;
}
// fn format_long_array_line(line: String, target_len: usize) -> String {
    
// }

// fn format_long_function_definition_line(line: String, target_len: usize) -> String {
    
// }

// This function works unless the code jumps by more lines than the following block contains.
pub fn add_blank_lines_back(file: &mut String, target_lines: Vec<(usize, String)>) {
    let mut lines: Vec<String> = file.lines().map(|x| x.to_string()).collect();
    // prinln!("{:#?}", &target_lines);
    for mut entry in target_lines.clone() {
        if lines.len() > entry.0+1 {
            let line = spaces_to_tabs(&entry.1);
            if lines[entry.0] == line {
                lines.insert(entry.0+1, "".to_string());
            }
            else {
                let mut idx = entry.0;
                'Inner: for i in (0..entry.0).rev() {
                    if &i < &lines.len() {
                        if &lines[i] == &line {
                            idx = i+1;
                            break 'Inner;
                        }
                    }
                }
                lines.insert(idx, "".to_string());
            }
        }
        else {
            let line = spaces_to_tabs(&entry.1);
            let mut idx = entry.0;
            'Inner: for i in (0..entry.0).rev() {
                if &i < &lines.len() {
                    if &lines[i] == &line {
                        idx = i+1;
                        break 'Inner;
                    }
                }
            }
            lines.insert(idx, "".to_string());
        }
    }
    *file = lines.join("\n");
}

fn join_single_line_constructs(file: &mut String, style: Style) {
    let mut allman = false;
    match style {
        Style::Allman => allman = true, 
        _ => (),
    };
    let lines: Vec<String> = file.split("\n").map(|x| x.to_string()).collect();
    let mut parts = Vec::<String>::new();
    let mut temp = String::new();
    let mut end = true;
    let mut indent = 0;
    for (i, line) in lines.iter().enumerate() {
        if allman && i < lines.len()-1 {
            let check = line.trim();
            let check2 = lines.iter().nth(i+1).unwrap().trim();
            if check.starts_with("switch") && (!check2.ends_with("{") && !check.ends_with(";")) {
                // must be added here to preserve indentation
                temp += format!("{} ", line.trim_end()).as_str();
                end = false;
                continue;
            }
        }
        else {
            let check = line.trim();
            if check.starts_with("switch") && (!check.ends_with("{") && !check.ends_with(";")) {
                temp += format!("{} ", line.trim_end()).as_str();
                end = false;
                continue;
            }
        }
        if !end && line.trim_end().ends_with(";") {
            end = true;
            temp += line.trim();
            parts.push(temp);
            temp = "".to_string();
            continue;
        }
        else if !end {
            temp += format!("{} ", line.trim()).as_str();
            continue;
        }
        parts.push(line.clone());
    }
    *file = parts.join("\n");
}

fn shift_back_preproc_lines(file: &mut String) {
    let lines: Vec<String> = file.split("\n").map(|x| x.to_string()).collect();
    let mut parts = Vec::<String>::new();
    for line in lines {
        // Labels are also shifted back here because it just made sense
        if line.trim_start().starts_with("#") || line.trim_end().ends_with(":") {
            let temp = remove_single_tab(&line);
            parts.push(temp);
        }
        else {
            parts.push(line);
        }
    }
    *file = parts.join("\n");
}

pub fn close_empty_curly_brace_blocks(file: &mut String) {
    let mut lines: Vec<String> = file.lines().into_iter().map(|x| x.to_string()).collect();
    let mut lines_to_remove = Vec::<usize>::new();
    for i in 0..lines.len()-1 {
        if lines[i].trim_end().ends_with("{") &&
           lines[i+1].trim_start().starts_with("}") &&
           lines[i+1].trim().len() == 1 {
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
                for _ in 0..temp_group.len() {
                    sort_includes(&mut temp_group);
                }
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
    return result.join("\n");
}

fn remove_comment(line: String) -> String {
    let idx = line.find("//").unwrap_or(line.len());
    let mut result: String;
    if idx == line.len() { result = line; }
    else { result = line[..idx].to_string(); }
    return result;
}

fn sort_includes(includes: &mut Vec<String>) {
    includes.sort_by(|a, b| {
        let mut a2 = a.clone();
        let mut b2 = b.clone();
        a2 = remove_comment(a2);
        b2 = remove_comment(b2);
        let a_has_angle = a2.contains('<') && a2.contains('>');
        let b_has_angle = b2.contains('<') && b2.contains('>');
        match (a_has_angle, b_has_angle) {
            (true, false) => Ordering::Less,    // Angle brackets come first
            (false, true) => Ordering::Greater, // Double quotes come after
            _ => a.cmp(b), // If both are the same type, sort normally
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

pub fn fix_stars(file: String) -> String {
    let lines: Vec<String> = file.lines().map(|x| x.to_string()).collect();
    let mut parts = Vec::<String>::new();
    for line in lines {
        let mut temp_line = String::new();
        for (i, c) in line.char_indices() {
            if line.len() > 1 {
                if i <= line.len()-2 && i > 0 {
                    // dereferences *must* be in parenthesis or at the start of the line
                    let deref = {
                        c == ' ' &&
                        line.at(i-1).unwrap() == '*' &&
                        line.at(i+1).unwrap().is_alphanumeric() &&
                        (
                            line.at(i-2).unwrap() == '(' ||
                            line.at(i-2).unwrap() == '\t'
                        )
                    };
                    if deref { continue; }
                    let ok_mult = {
                        c == '*' &&
                        line.at(i-1).unwrap() == ' ' &&
                        line.at(i+1).unwrap() == ' '
                    };
                    if ok_mult {
                        temp_line.push(c);
                        continue;
                    }
                    let ok_deref = {
                        c == '*' &&
                        line.at(i+1).unwrap().is_alphabetic() &&
                        line.at(i-1).unwrap() == ' ' &&
                        (
                            line.at(i-2).unwrap() == ' ' || 
                            line.at(i-2).unwrap() == ';' || 
                            line.at(i-3).unwrap() == ';'  
                        )
                    };
                    if ok_deref {
                        temp_line.push(c);
                        continue;
                    }
                    let ptr = {
                        c == '*' &&
                        line.at(i+1).unwrap().is_alphanumeric() &&
                        (
                            line.at(i-1).unwrap().is_alphanumeric() ||
                            line.at(i-1).unwrap() == '*' 
                        )
                    };
                    if ptr {
                        temp_line += format!("{} ", c).as_str();
                        continue;
                    }
                    let fix = {
                        c == '*' &&
                        line.at(i-1).unwrap().is_alphanumeric() &&
                        line.at(i+1).unwrap().is_whitespace()
                    };
                    if fix {
                        temp_line += format!(" {}", c).as_str();
                        continue;
                    }
                    let swap_ptr = {
                        c == '*' &&
                        line.at(i-1).unwrap().is_whitespace() &&
                        line.at(i+1).unwrap().is_alphanumeric()
                    };
                    if swap_ptr {
                        temp_line.pop();
                        temp_line += format!("{} ", c).as_str();
                        continue
                    }
                }
            }
            temp_line.push(c);
        }
        parts.push(temp_line);
    }
    return parts.join("\n");
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
                line.at(i+1) == Some('*') &&
                (
                    line.at(i-1).unwrap().is_alphanumeric() ||
                    line.at(i-1) == Some('*')               ||
                    line.at(i-1) == Some('>')
                )
            };
            if skip {
                continue;
            }
        }
        result += c.to_string().as_str();
    }
    return result;
}

pub fn switch_pointer_spaces(line: String) -> String {
    let mut result = String::new();
    for (i, c) in line.char_indices() {
        if i <= line.len()-2 && i > 0 {
            let skip = {
                c == ' ' &&
                line.at(i+1) == Some('*') &&
                (line.at(i-1).unwrap().is_alphanumeric() || line.at(i-1) == Some('*'))
            };
            if skip {
                continue;
            }
        }
        result += c.to_string().as_str();
    }
    for (i, c) in line.char_indices() {
        if i < line.len()-2 {
            let add_space = {
                c == '*' &&
                line.at(i+1) != Some('*') &&
                line.at(i+1) != Some(' ')
            };
            if add_space {
                result.insert(i, ' ');
            }
        }
    }
    return result;
}

pub fn remove_reference_spaces(line: String) -> String {
    let mut result = String::new();
    for (i, c) in line.char_indices() {
        if i <= line.len()-3 && i > 0 {
            let skip = {
                c == ' ' &&
                line.at(i+1) == Some('&') &&
                line.at(i+2) != Some('&') &&
                (
                    line.at(i-1).unwrap().is_alphanumeric() ||
                    line.at(i-1) == Some('>')               ||
                    line.at(i-1) == Some('*')
                )
            };
            if skip { continue; }
        }
        result += c.to_string().as_str();
    }
    return result;
}

fn remove_single_tab(line: &String) -> String {
    let mut line = line.clone();
    if line.starts_with("\t") { line.remove(0); }
    return line;
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
    if line.at(start-1) == Some(' ') {
        result.remove(start-1);
    }
    return result;
}

pub fn remove_unnecessary_spaces(line: &String) -> String {
    let leading_tokens = vec!['(', '[', ' ', '!'];
    let ending_tokens = vec![')', '[', ']', ' ', ',', ';'];
    let mut result = String::new();
    let mut skip = false;
    let mut in_string = false;
    for (i, c) in line.char_indices() {
        if skip {
            skip = false;
            continue;
        }
        if let Some(next) = line.at(i+1) {
            if next == '"' && c != '\\'{
                if in_string { in_string = false; }
                else { in_string = true; }
            }
            if ending_tokens.contains(&next) && !in_string {
                if c == ' ' { continue; }
            }
            if leading_tokens.contains(&c) && !in_string {
                if next == ' ' { skip = true; }
            }
        }
        result += c.to_string().as_str();
    }
    return result.trim().to_string();
}

pub fn remove_double_spaces(line: &String) -> String {
    let mut result = "".to_string();
    for (i, c) in line.chars().enumerate() {
        if i < line.len()-1 {
            if c == ' ' && line.at(i+1) == Some(' ') { continue; }
        }
        result += c.to_string().as_str();
    }
    return result;
}

pub fn remove_whitespace_before_commas(line: &String) -> String {
    let mut result = "".to_string();
    for (i, c) in line.chars().enumerate() {
        if i < line.len()-1 {
            if c == ' ' && line.at(i+1) == Some(',') { continue; }
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

pub fn count_leading_chars(src: &String, target: char) -> usize {
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

fn check_line_is_blank(line: &String) -> bool {
    return line.is_empty() || line.chars().all(|x| x.is_whitespace() || x == '\t');
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
    // .cpp has to stay before .c, otherwise it breaks
    let exts = {
        vec![".cpp", ".cc", ".C", ".c", ".hpp", ".hh", ".H", ".h", ".go", ".py", ".txt"]
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
    println!("USAGE:\nblfmt <file-path> <flags + opts>");
}

pub fn write_results(path: &String, results: String) {
    let ok = write_file(path, results.as_bytes());
    match ok {
        Ok(_) => println!("Successfully wrote: '{}'", path),
        Err(e) => println!("Error during `format_txt_file()`: {}", e),
    };
}

pub fn print_results(results: &String) {
    println!("{}", results);
}

