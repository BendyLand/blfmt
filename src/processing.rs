use std::process::Command;
use std::fs::{self, File};
use std::io::{Write};
use crate::{options, utils};


fn process_txt_file(path: &str, opts: &options::TxtOpts) -> String {
    let file_contents = fs::read_to_string(path).unwrap();
    let mut result = String::new();
    let (cols, spacing) = (opts.columns, opts.spacing);
    
    
    
    
    
    let safe_temp = &file_contents.clone();
    return safe_temp.to_owned();
}

pub fn begin_processing_txt_files(path: String, opts: options::TxtOpts) {
    let file_paths = utils::find_paths("txt", &path).unwrap_or_default();
    for path in file_paths {
        let path_clone = path.clone();
        let new_file = process_txt_file(&path, &opts);
        let mut dest = File::create(path).unwrap();
        let ok = dest.write_all(new_file.as_bytes()); 
        match ok {
            Ok(_) => println!("Successfully wrote: {}", path_clone),
            Err(e) => println!("Error writing file: {}", e),
        };
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