use std::{fs, fs::File, io::Write, io::Error};
use regex::Regex;
use crate::options::{self, TxtOpts};
use std::ops::{Bound, RangeBounds};

pub trait StringUtils {
    fn substring(&self, start: usize, len: usize) -> &str;
    fn slice(&self, range: impl RangeBounds<usize>) -> &str;
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

pub fn remove_empty_lines(lines: Vec<&str>) -> String {
    let mut result = String::new();
    for line in lines {
        if !line.is_empty() {
            result += (line.to_string() + "\n").as_str();
        }
    }
    return result;
}

pub fn write_file(path: String, contents: &[u8]) -> Result<(), Error> {
    let mut dest = File::create(path).unwrap();
    let ok = dest.write_all(contents);
    return ok;
}

pub fn restore_example_rs_file() {
    let temp1 = fs::read_to_string("/Users/benlandrette/ccode/serious-projects/bendyland/blfmt/safe_rs_ex1.rs").unwrap().to_owned();
    let good1 = temp1.as_bytes();
    let mut file1 = fs::File::create("/Users/benlandrette/ccode/serious-projects/bendyland/blfmt/rs_ex1.rs").expect("Unable to get rs_ex1.rs");
    let res1 = file1.write_all(good1);
    match res1 {
        Err(e) => println!("{}", e),
        _ => (),
    };
}

pub fn restore_example_cpp_file() {
    let temp1 = fs::read_to_string("/Users/benlandrette/ccode/serious-projects/bendyland/blfmt/safe_cpp_example.cpp").unwrap().to_owned();
    let good1 = temp1.as_bytes();
    let mut file1 = fs::File::create("/Users/benlandrette/ccode/serious-projects/bendyland/blfmt/cpp_ex.cpp").expect("Unable to get example_file.cpp");
    let res1 = file1.write_all(good1);
    match res1 {
        Err(e) => println!("{}", e),
        _ => (),
    };
}

pub fn restore_example_c_file() {
    let temp1 = fs::read_to_string("/Users/benlandrette/ccode/serious-projects/bendyland/blfmt/safe_example_file.c").unwrap().to_owned();
    let good1 = temp1.as_bytes();
    let mut file1 = fs::File::create("/Users/benlandrette/ccode/serious-projects/bendyland/blfmt/ex1.c").expect("Unable to get example_file.c");
    let res1 = file1.write_all(good1);
    match res1 {
        Err(e) => println!("{}", e),
        _ => (),
    };
}

pub fn restore_example_txt_files() {
    let temp1 = fs::read_to_string("/Users/benlandrette/ccode/serious-projects/bendyland/blfmt/storage/safe-dir-in-storage/one.txt").unwrap().to_owned();
    let good1 = temp1.as_bytes();
    let temp2 = fs::read_to_string("/Users/benlandrette/ccode/serious-projects/bendyland/blfmt/storage/safe-dir-in-storage/two.txt").unwrap();
    let good2 = temp2.as_bytes();
    let temp3 = fs::read_to_string("/Users/benlandrette/ccode/serious-projects/bendyland/blfmt/storage/safe-dir-in-storage/three.txt").unwrap();
    let good3 = temp3.as_bytes();
    let mut file1 = fs::File::create("/Users/benlandrette/ccode/serious-projects/bendyland/blfmt/storage/one.txt").expect("Unable to get one.txt");
    let mut file2 = fs::File::create("/Users/benlandrette/ccode/serious-projects/bendyland/blfmt/storage/two.txt").expect("Unable to get two.txt");
    let mut file3 = fs::File::create("/Users/benlandrette/ccode/serious-projects/bendyland/blfmt/storage/three.txt").expect("Unable to get three.txt");
    let res1 = file1.write_all(good1);
    let res2 = file2.write_all(good2);
    let res3 = file3.write_all(good3);
    match res1 {
        Err(e) => println!("{}", e),
        _ => (),
    }
    match res2 {
        Err(e) => println!("{}", e),
        _ => (),
    }
    match res3 {
        Err(e) => println!("{}", e),
        _ => (),
    }
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

pub fn line_ends_with_curly_brace(line: &String) -> bool {
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
