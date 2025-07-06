use crate::{group, options, utils, utils::StringUtils, c_format, cpp_format, txt_format, c_ast, cpp_ast};
use regex::Regex;
use std::fs::{self, File};
use std::io::Write;
use std::process::Command;

pub fn format_c_file(path: String, style: utils::Style, write_arg: bool) {
    let ast = c_format::parse_c_file(path.clone());
    let contents = std::fs::read_to_string(&path).unwrap_or("NOFILE".to_string());
    if contents == "NOFILE".to_string() {
        println!("'{}' not found.", path);
        return;
    }
    let result = c_ast::traverse_c_ast(ast, contents, style);
    if write_arg { utils::write_results(&path, result); }
    else { utils::print_results(&result); }
}

pub fn format_cpp_file(path: String, style: utils::Style, write_arg: bool) {
    let ast = cpp_format::parse_cpp_file(path.clone());
    let contents = std::fs::read_to_string(&path).unwrap_or("NOFILE".to_string());
    if contents == "NOFILE".to_string() {
        println!("'{}' not found.", path);
        return;
    }
    let result = cpp_ast::traverse_cpp_ast(ast, contents, style);
    if write_arg { utils::write_results(&path, result); }
    else { utils::print_results(&result); }
}

pub fn format_py_file(path: String) {
    let command = utils::sanitize(format!("black {}", path));
    let res = Command::new("sh").arg("-c").arg(command).output().unwrap();
    let res = String::from_utf8(res.stderr).unwrap_or_default();
    println!("{}", res);
}

pub fn format_go_file(path: String) {
    let command = utils::sanitize(format!("gofmt -w {}", path));
    let res = Command::new("sh").arg("-c").arg(command).output().unwrap();
    let err = String::from_utf8(res.stderr).unwrap_or_default();
    if err.as_str() != "" {
        eprintln!("Error running gofmt:\n{}", err);
    }
    else {
        println!("Go file formatted successfully!");
    }
}

pub fn format_txt_file(path: String, opts: options::TxtOpts, opt_titles: &[String], write_arg: bool) {
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
    if write_arg { utils::write_results(&path, paragraphs); }
    else { utils::print_results(&paragraphs); }
}
