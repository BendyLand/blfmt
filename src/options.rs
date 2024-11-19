use crate::restore;
use crate::utils;

#[derive(Clone, Copy, Debug)]
pub struct TxtOpts {
    pub columns: usize,
    pub spacing: usize,
}

pub fn get_c_style(args: &Vec<String>) -> utils::Style {
    let args: Vec<String> = args.into_iter().map(|x| x.to_lowercase()).collect();
    let result: utils::Style;
    if args.contains(&"-s".to_string()) || args.contains(&"--style".to_string()) {
        if args.contains(&"allman".to_string()) { 
            result = utils::Style::Allman; 
        }
        else if args.contains(&"k&r".to_string()) || args.contains(&"knr".to_string()) { 
            result = utils::Style::KnR; 
        }
        else if args.contains(&"stroustrup".to_string()) { 
            result = utils::Style::Stroustrup; 
        }
        else { result = utils::Style::Stroustrup; }
    }
    else { result = utils::Style::Stroustrup; }
    return result;
}

pub fn get_txt_opts(args: &Vec<String>) -> TxtOpts {
    let opts: TxtOpts;
    if args.contains(&"-o".to_string()) || args.contains(&"--opts".to_string()) {
        let args = args.iter().map(|x| x.as_str()).collect::<Vec<&str>>();
        opts = utils::extract_txt_opts(args);
    }
    else {
        opts = TxtOpts{columns: 80, spacing: 1};
    }
    return opts;
}

pub fn check_restore_arg(args: &Vec<String>) -> usize {
    let contains_restore_arg = {
        args.contains(&"-r".to_string()) || 
        args.contains(&"--restore".to_string())
    };
    if contains_restore_arg {
        // utils::restore_example_txt_files();
        restore::restore_example_c_file();
        // utils::restore_example_cpp_file();
        // utils::restore_example_rs_file();
        println!("Example file restored.");
        return 1; 
    }
    return 0;
}

pub fn check_help_arg(args: &Vec<String>) -> usize {
    let contains_help_arg = {
        args.contains(&"-h".to_string()) || 
        args.contains(&"--help".to_string())
    };
    if contains_help_arg {
        println!("Welcome to the blfmt help menu!");
        utils::print_usage();
        println!("Here are the available file types:");
        utils::display_file_extensions();
        return 1;
    }
    return 0;
}