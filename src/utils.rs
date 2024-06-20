use std::{fs, io::Write};


pub fn find_paths(file_ext: &str, root_dir: &str) -> Option<Vec<String>> {
    let maybe_files = fs::read_dir(root_dir);
    let files = match maybe_files {
        Ok(f) => f,
        Err(e) => {
            println!("Error reading directory: {}", e);
            return None;
        },
    };
    let mut result = Vec::<String>::new();
    for file in files {
        let file = match file {
            Ok(f) => f,
            Err(e) => {
                println!("There is some kind of error on this file: {}", e);
                continue;
            }
        };
        let file_path = file.path().as_os_str().to_string_lossy().into_owned();
        if file_path.ends_with(file_ext) {
            result.push(file_path);
        }
    }
    return Some(result);
}

pub fn restore_test_files() {
    let temp1 = fs::read_to_string("../storage/safe-dir-in-storage/one.txt").unwrap_or_default().to_owned();
    let good1 = temp1.as_bytes();
    let temp2 = fs::read_to_string("../storage/safe-dir-in-storage/two.txt").unwrap_or_default();
    let good2 = temp2.as_bytes();
    let temp3 = fs::read_to_string("../storage/safe-dir-in-storage/three.txt").unwrap_or_default();
    let good3 = temp3.as_bytes();
    let mut file1 = fs::File::create("../storage/one.txt").expect("Unable to get one.txt");
    let mut file2 = fs::File::create("../storage/two.txt").expect("Unable to get two.txt");
    let mut file3 = fs::File::create("../storage/three.txt").expect("Unable to get three.txt");
    let res1 = file1.write_all(good1);
    let res2 = file2.write_all(good2);
    let res3 = file3.write_all(good3);
    match res1 {
        Err(e) => println!("{}", e),
        _ => (),
    }
    match res2 {
        Err(e) => println!("{}", e),
        _ => (),
    }
    match res3 {
        Err(e) => println!("{}", e),
        _ => (),
    }
}

pub fn intersperse<T: Clone>(vec: Vec<T>, value: T) -> Vec<T> {
    if vec.is_empty() {
        return vec;
    }
    let mut result = Vec::<T>::new();
    for item in vec {
        result.push(item);
        result.push(value.clone());
    }
    result = result[..result.len()-1].into_iter().map(|x| x.to_owned()).collect::<Vec<T>>();
    return result;
}

pub fn median(lengths: &Vec<usize>) -> usize {
    let mut sorted_list = lengths.clone();
    sorted_list.sort();
    let idx = sorted_list.len() / 2;
    return sorted_list[idx];
}

pub fn check_for_even_line_length(lines: &Vec<&str>) -> bool {
    let line_lens = lines.iter().map(|x| x.len()).collect::<Vec<usize>>();
    let med = median(&line_lens);
    let check_val = (med as f32 * 0.8).ceil();
    let mut count = 0;
    for i in 0..line_lens.len() {
        if (line_lens[i] as f32) < check_val && i != 0 {
            count += 1;
        }
    }
    return count == 0;
}


