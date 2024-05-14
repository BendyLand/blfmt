mod parser;
mod text_file;

/*
Usage:

icfmt [file-extension] [opt-flags] [dir-path]
*/

fn main() {
    let dir_contents = parser::parse_args();
    let default_txt_opts = text_file::TxtOpts {
        columns: 80,
        spacing: 1,
    };
    let path = match dir_contents {
        Some(contents) => contents,
        None => String::new(),
    };
    let maybe_files = text_file::get_txt_files(path);
    let files = match maybe_files {
        Some(files) => files,
        None => Vec::new(),
    };
    for file in files {
        text_file::process_txt_file(&file, &default_txt_opts);
    }
}
