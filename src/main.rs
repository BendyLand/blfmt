mod parser;
mod text_file;
mod processing;
mod groups;
mod utils;

fn main() {
    let maybe_args = parser::parse_args();
    let (path, ext, opts) = {
        match maybe_args {
            Some((a, b, c)) => (a, b, c),
            None => (String::new(), String::new(), Vec::<String>::new()),
        }
    };
    process_file_type(path, ext, opts);
    // let test1 = vec!["this is the first line", "this is the second line", "this is the third line"];
    // let test2 = test1.join("\n");
    // let test3 = test1.join("\n\n");
    // let test4 = test1.join("\n\n\n");
    // let test5 = test1.join("\n\n\n\n");
    // println!("{:?} {} {} {} {}", test1, test2, test3, test4, test5);
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
                println!("Columns: {}, Spacing: {}", &cols, &spaces);
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
