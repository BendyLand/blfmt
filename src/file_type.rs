use std::process::Command;
use std::fs;
use crate::text_file;

#[allow(unused_variables)]
pub fn begin_processing_txt_files(path: String) {
    let opts = text_file::TxtOpts{columns: 80, spacing: 1};
    let maybe_text_files = text_file::find_txt_file_paths(path);
    let text_file_paths = match maybe_text_files {
        Some(file_paths) => file_paths,
        None => Vec::<String>::new(),
    };
    for file_path in &text_file_paths {
        println!("Processing: {}", &file_path);
        let new_file = text_file::process_txt_file(file_path, opts);
        fs::write(file_path, new_file).expect("Unable to write file.");
        println!("File processed successfully!");
    }
}

pub fn begin_processing_go_files(path: String) {
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