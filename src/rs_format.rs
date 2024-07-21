use crate::utils;

pub fn format_rs_file_group(section: String) -> String {
    let top_lines = {
        vec!["#", "use", "mod", "//"]
            .into_iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
    };
    let mid_levels = {
        vec!["trait", "struct", "enum", "pub"]
            .into_iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
    };
    if utils::starts_with_any(&section, &top_lines) {
        // todo: order: use, mod, comments; then join by single space between them.
        println!("Top level statement!");
        println!("{}", &section);
    }
    else if utils::starts_with_any(&section, &mid_levels) {
        // todo: order: traits, enums, structs, public functions; format funcs; then join by single space between them.
        println!("Mid level statement!");
        println!("{}", &section);
    }
    else {
        // todo: format private functions.
        println!("Bottom level statement!");
        println!("{}", &section);
    }
    return section;
}