use std::env;
use std::fs;
use crate::utils;

pub fn parse_args() -> Option<(String, Vec<String>)> {
    let args: Vec<String> = env::args().collect();
    // restore and help are special commands
    if args.contains(&"-r".to_string()) || args.contains(&"--restore".to_string()) {
        return Some((String::new(), vec!["-r".to_string()]));
    }
    if args.contains(&"-h".to_string()) || args.contains(&"--help".to_string()) {
        return Some((String::new(), vec!["-h".to_string()]));
    }
    if args.len() < 2 {
        utils::print_usage();
        return None;
    }
    else if args.len() == 2 {
        let arg = &args[1];
        return Some((arg.to_owned(), vec![]));
    }
    else {
        let path = &args[1].to_string();
        let rest = &args[2..].into_iter().map(|x| x.to_owned()).collect::<Vec<String>>();
        return Some((path.to_owned(), rest.to_owned()));
    }
}
