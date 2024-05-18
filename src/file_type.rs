use std::process::Command;
use crate::text_file;

#[allow(unused_variables)]
pub fn process_txt_file(path: String) {
    let opts = text_file::TxtOpts{columns: 80, spacing: 1};
    let maybe_text_files = text_file::find_txt_files(path);
    let text_files = match maybe_text_files {
        Some(files) => files,
        None => Vec::<String>::new(),
    };
    for file in text_files {
        ()
    }
}

pub fn process_go_file(path: String) {
    let command = format!("gofmt -w {}", path);
    let res = Command::new("sh").arg("-c").arg(command).output().unwrap();
    let err = String::from_utf8(res.stderr).unwrap_or_default();
    if err.as_str() != "" {
        eprintln!("Error running gofmt:\n{}", err);
    }
    else {
        println!("Go files formatted successfully!");
    }
}