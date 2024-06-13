use std::process::Command;
use std::fs::{File};
use std::io::{Write};
use crate::text_file;

pub fn begin_processing_txt_files(path: String, opts: text_file::TxtOpts) {
    let file_paths = text_file::find_txt_file_paths(path);
    let paths = match file_paths {
        Some(paths) => paths,
        None => vec![],
    };
    for current_path in paths {
        let path_clone = current_path.clone();
        let new_file = text_file::process_txt_file(&current_path, opts);
        let mut dest = File::create(current_path).unwrap();
        let ok = dest.write_all(new_file.as_bytes()); 
        match ok {
            Err(e) => println!("There was a problem writing to the file: {}", e),
            _ => println!("Successfully formatted file at: {}", path_clone)
        }
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