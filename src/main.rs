mod parser;
mod text_file;
mod file_type;

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
            file_type::process_txt_file(path);
        },
        "go" => {
            file_type::process_go_file(path);
        }
        _ => println!("File type not supported."),
    }
}
