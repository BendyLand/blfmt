use std::io::Read;

use crate::utils;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TxtOpts {
    pub columns: usize,
    pub spacing: usize,
}

impl Default for TxtOpts {
    fn default() -> Self {
        Self {
            columns: 80,
            spacing: 1,
        }
    }
}

pub fn parse_c_style_value(val: &str) -> Option<utils::Style> {
    match val.trim().to_ascii_lowercase().as_str() {
        "allman" => Some(utils::Style::Allman),
        "knr" | "k&r" => Some(utils::Style::KnR),
        "stroustrup" => Some(utils::Style::Stroustrup),
        _ => None,
    }
}

pub fn read_stdin_text() -> String {
    let mut text = String::new();
    let _ = std::io::stdin().read_to_string(&mut text);
    text
}

pub fn print_help_menu() {
    println!("Welcome to the blfmt help menu!");
    utils::print_usage();
    println!("The available file types are:");
    utils::display_file_extensions();
}
