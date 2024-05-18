mod parser;
mod text_file;
use std::process::Command;

/*
Usage:
icfmt [file-extension] [opt-flags] [dir-path]
*/

fn main() {
    let (file_path, file_type) = parser::parse_args();
    let path = match file_path {
        Some(fp) => fp,
        None => String::new(),
    };
    let ext = match file_type {
        Some(t) => t,
        None => String::new(),
    };
    process_file_type(path, ext);
}

fn process_file_type(path: String, file_type: String) {
    match file_type.as_str() {
        "txt" => {
            let opts = text_file::TxtOpts{columns: 80, spacing: 1};
            let maybe_text_files = text_file::find_txt_files(path);
            let text_files = match maybe_text_files {
                Some(files) => files,
                None => Vec::<String>::new(),
            };
            for file in text_files {
                text_file::process_txt_file(&file, &opts);
            }
        },
        "go" => {
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
        _ => println!("File type not supported."),
    }
}
