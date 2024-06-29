use std::process::Command;
use std::fs::{self, File};
use std::io::{Write};
use crate::{options, utils};


pub fn format_txt_file(path: String, opts: options::TxtOpts) {
    let path_clone = path.clone();
    let file_contents = fs::read_to_string(path).unwrap();
    let result = String::new();
    let (cols, spacing) = (opts.columns, opts.spacing);
    dbg!(path_clone);
    /* 
    todo: 

        ? make group_paragraphs(lines, opt_titles)
        ? make format_paragraph(paragraph, opts)
        ? double check the write_file logic commented out below.

    */

    // let mut dest = File::create(path).unwrap();
    // let ok = dest.write_all(new_file.as_bytes()); 
    // match ok {
    //     Ok(_) => println!("Successfully wrote: {}", path_clone),
    //     Err(e) => println!("Error writing file: {}", e),
    // };
}

pub fn format_go_file(path: String) {
    let command = format!("gofmt -w {}", path);
    let res = Command::new("sh").arg("-c").arg(command).output().unwrap();
    let err = String::from_utf8(res.stderr).unwrap_or_default();
    if err.as_str() != "" {
        eprintln!("Error running gofmt:\n{}", err);
    }
    else {
        println!("Go file formatted successfully!");
    }
}