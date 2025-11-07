use crate::options;

pub fn format_paragraph(paragraph: String, opts: options::TxtOpts) -> String {
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
