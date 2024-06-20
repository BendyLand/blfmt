mod parser;
mod text_file;
mod processing;
mod grouping;
mod utils;

// Usage:
// cargo run [path] [filetype] [filetype opts] 
// cargo run ../storage txt 80 1 
// 
// To reset text files:
// cp ../storage/safe-dir-in-storage/*.txt ../storage

fn main() {
    let maybe_args = parser::parse_args();
    let (path, ext, opts) = {
        match maybe_args {
            Some((a, b, c)) => (a, b, c),
            None => (String::new(), String::new(), Vec::<String>::new()),
        }
    };
    process_file_type(path, ext, opts);
    utils::restore_test_files();
}

fn process_file_type(path: String, file_type: String, opts: Vec<String>) {
    match file_type.as_str() {
        "txt" => {
            if opts.len() >= 2 {
                let cols = {
                    match opts[1].parse::<u8>() {
                        Ok(num) => num,
                        Err(e) => {
                            println!("Invalid argument provided: {}", e);
                            80
                        },
                    }
                };
                let spaces = {
                    match opts[2].parse::<u8>() {
                        Ok(num) => num,
                        Err(e) => {
                            println!("Invalid argument provided: {}", e);
                            1
                        },
                    }
                };
                let opts = text_file::TxtOpts { columns: (cols), spacing: (spaces) };
                processing::begin_processing_txt_files(path, opts); 
            }
            else {
                println!("Invalid arguments.");
            }
        },
        "go" => {
            processing::begin_processing_go_files(path);
        }
        _ => println!("File type not supported."),
    }
}
