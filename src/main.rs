mod parser;

/*
Usage:

icfmt [file-extension] [opt-flags] [dir-path]
*/

fn main() {
    parser::parse_args();
}
