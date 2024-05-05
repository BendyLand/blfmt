use std::env;

/* 
Usage:

icfmt [extension] [opt-flags] [path]
*/

fn main() {
    parse_args();
}

fn parse_args() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 { 
        println!("USAGE: icfmt [target file-extension] [opt-flags] [path]");
        return; 
    }
    for i in 0..args.len() {
        if i > 0 {
            match args[i].as_str() {
                "txt" => format_txt_files(),
                _ => format_txt_files(),
            }
        }
    }
}

fn format_txt_files() {
    let path_buf = env::current_dir();
    let path = match path_buf {
        Ok(p) => {
            p
            .to_str()
            .map(|s| s.to_string())
            .unwrap_or(String::from("FOUND A BUG LINE 28"))
        },
        Err(e) => {
            println!("There was a problem getting the path: {}", e);
            String::from("FOUND A BUG LINE 32")
        },
    };
    dbg!(path);
}