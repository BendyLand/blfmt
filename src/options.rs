use crate::utils;

#[derive(Clone, Copy, Debug)]
pub struct TxtOpts {
    pub columns: usize,
    pub spacing: usize,
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

pub fn check_minimum_args(args: &Vec<String>) -> usize {
    if &args.len() < &2 {
        println!("Invalid arguments provided.");
        utils::print_usage();
        return 1;
    }
    return 0;
}

pub fn check_restore_arg(args: &Vec<String>) -> usize {
    let contains_restore_arg = {
        args.contains(&"-r".to_string()) || 
        args.contains(&"--restore".to_string())
    };
    if contains_restore_arg {
        utils::restore_test_files();
        println!("Test files restored.");
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