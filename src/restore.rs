use std::{fs, fs::File, io::Write, io::Error};
use crate::utils;

//* THESE PATHS ARE PROBABLY ALL WRONG

pub fn restore_example_rs_file() {
    let temp1 = fs::read_to_string("/Users/benlandrette/ccode/serious-projects/bendyland/blfmt/safe_rs_ex1.rs").unwrap().to_owned();
    let good1 = temp1.as_bytes();
    let mut file1 = fs::File::create("/Users/benlandrette/ccode/serious-projects/bendyland/blfmt/rs_ex1.rs").expect("Unable to get rs_ex1.rs");
    let res1 = file1.write_all(good1);
    match res1 {
        Err(e) => println!("{}", e),
        _ => (),
    };
}

pub fn restore_example_cpp_file() {
    let temp1 = fs::read_to_string("/Users/benlandrette/ccode/serious-projects/bendyland/blfmt/safe_cpp_example.cpp").unwrap().to_owned();
    let good1 = temp1.as_bytes();
    let mut file1 = fs::File::create("/Users/benlandrette/ccode/serious-projects/bendyland/blfmt/cpp_ex.cpp").expect("Unable to get example_file.cpp");
    let res1 = file1.write_all(good1);
    match res1 {
        Err(e) => println!("{}", e),
        _ => (),
    };
}

pub fn restore_example_c_file() {
    let temp1 = fs::read_to_string("/Users/benlandrette/ccode/serious-projects/bendyland/blfmt/storage/safe_ex1.c").unwrap().to_owned();
    let good1 = temp1.as_bytes();
    let mut file1 = fs::File::create("/Users/benlandrette/ccode/serious-projects/bendyland/blfmt/ex1.c").expect("Unable to get example_file.c");
    let res1 = file1.write_all(good1);
    match res1 {
        Err(e) => println!("{}", e),
        _ => (),
    };
}

pub fn restore_example_txt_files() {
    let temp1 = fs::read_to_string("/Users/benlandrette/ccode/serious-projects/bendyland/blfmt/storage/safe-dir-in-storage/one.txt").unwrap().to_owned();
    let good1 = temp1.as_bytes();
    let temp2 = fs::read_to_string("/Users/benlandrette/ccode/serious-projects/bendyland/blfmt/storage/safe-dir-in-storage/two.txt").unwrap();
    let good2 = temp2.as_bytes();
    let temp3 = fs::read_to_string("/Users/benlandrette/ccode/serious-projects/bendyland/blfmt/storage/safe-dir-in-storage/three.txt").unwrap();
    let good3 = temp3.as_bytes();
    let mut file1 = fs::File::create("/Users/benlandrette/ccode/serious-projects/bendyland/blfmt/storage/one.txt").expect("Unable to get one.txt");
    let mut file2 = fs::File::create("/Users/benlandrette/ccode/serious-projects/bendyland/blfmt/storage/two.txt").expect("Unable to get two.txt");
    let mut file3 = fs::File::create("/Users/benlandrette/ccode/serious-projects/bendyland/blfmt/storage/three.txt").expect("Unable to get three.txt");
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
