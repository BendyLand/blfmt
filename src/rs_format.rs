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
    let result; 
    if utils::starts_with_any(&section, &top_lines) {
        result = order_top_level(&section);
    }
    else if utils::starts_with_any(&section, &mid_levels) {
        // todo: order: traits, enums, structs, public functions; format funcs; then join by single space between them.
        // println!("Mid level statement!");
        result = format_rs_function(&section);
    }
    else {
        // todo: format private functions.
        // println!("Bottom level statement!");
        result = format_rs_function(&section);
    }
    return result;
}

fn order_top_level(section: &String) -> String {
    let lines = section.split("\n").collect::<Vec<&str>>();
    let mut result = String::new();
    let mut attr_temp = String::new();
    let mut use_temp = String::new();
    let mut mod_temp = String::new();
    let mut remainder = String::new();
    let mut in_comment = false;
    for line in lines {
        in_comment = line.starts_with("/*") || in_comment;
        if in_comment && line.contains("*/") { in_comment = false; }
        if line.starts_with("#") && !in_comment {
            attr_temp += (line.to_string() + "\n").as_str();
        }
        else if line.starts_with("use") && !in_comment {
            use_temp += (line.to_string() + "\n").as_str();
        }
        else if line.starts_with("mod") && !in_comment {
            mod_temp += (line.to_string() + "\n").as_str();
        }
        else {
            remainder += (line.to_string() + "\n").as_str();
        }
    }
    let temp = {
        vec![attr_temp, use_temp, mod_temp, remainder]
            .into_iter()
            .map(|x| order_top_level_group(x))
            .collect::<Vec<String>>()
    };
    return temp.join("\n");
}

fn order_top_level_group(group: String) -> String {
    let mut lines = group.split("\n").filter(|x| !x.is_empty()).collect::<Vec<&str>>();
    lines.sort();
    let result = lines.join("\n").trim().to_string();
    return result;
}

fn format_rs_function(section: &String) -> String {
    let lines = section.split("\n").map(|x| x.to_string()).filter(|x| !x.is_empty()).collect::<Vec<String>>();
    let mut result = Vec::<String>::new();
    let mut result_str = "".to_string();
    result.push(format_top_brace(&lines));
    let mut lines = normalize(&lines);
    lines = format_inner_curly_braces(&lines);
    println!("");
    println!("Line: {}", result[0]);
    for line in lines {
        println!("Line: {}", line);
    }
    println!("");
    /*  
    indent_inside_fn()
    format_long_statements()
    */
    return result_str;
}

fn format_top_brace(lines: &Vec<String>) -> String {
    let mut result = String::new();
    for (i, line) in lines.into_iter().enumerate() {
        if i == 0 {
            let mut temp_line; 
            if line.trim_end().ends_with(",") {
                temp_line = format!("{} {}", line, &lines[i+1].trim()); // won't handle > 2 line fn defs
                if !temp_line.contains("{") {
                    temp_line = temp_line.trim_end().to_string() + " {";
                }
                else {
                    temp_line = temp_line;
                }
            }
            else {
                if !line.contains("{") {
                    temp_line = line.trim_end().to_string() + " {";
                }
                else {
                    temp_line = line.to_string(); 
                }
            }
            result = temp_line;
            break;
        }
    }
    return result;
}

fn normalize(lines: &Vec<String>) -> Vec<String> {
    let mut result = Vec::<String>::new();
    for (i, line) in lines.into_iter().enumerate() {
        if i == 0 { continue; }
        result.push(line.trim().to_string());
    }
    return result;
}

fn format_inner_curly_braces(lines: &Vec<String>) -> Vec<String> {
    let mut result = Vec::<String>::new();
    let brace_lines = {
        vec!["if", "else", "for", "while"]
            .into_iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
    }; 
    let mut temp = String::new();
    let mut skip = false;
    for (i, line) in lines.into_iter().enumerate() {
        if skip { 
            skip = false;
            continue; 
        }
        if i == 0 {
            let is_remaining_fn_def = {
                line.contains(")") &&
                line.contains("{") &&
                !utils::starts_with_any(line, &brace_lines)
            };
            if is_remaining_fn_def { continue; }
        }
        if utils::starts_with_any(line, &brace_lines) {
            if !line.contains("{") {
                temp += (line.trim_end().to_string() + " {").as_str();
                skip = true;
            }
            else {
                temp += line;
            }
            result.push(temp);
            temp = "".to_string();
        }
        else {
            if line.contains("else") && line.contains("}") {
                temp = line[1..].trim().to_string();
                if !temp.contains("{") {
                    temp += " {";
                    skip = true;
                }
                result.push("}".to_string());
            }
            else {
                temp = line.to_string();
            }
            result.push(temp);
            temp  = "".to_string();
        }
    }
    return result;
}