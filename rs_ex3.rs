extern crate image;

use crate::utils;
use std::fmt;
use std::error::Error;

#[derive(Debug)]
struct UnsupportedFormatError {
    format: String,
}

impl fmt::Display for UnsupportedFormatError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        return write!(f, "Unsupported target format: {}", self.format);
    }
}

impl Error for UnsupportedFormatError {}

pub fn convert_image(file: String, target: String) -> Result<(), Box<dyn std::error::Error>> {
    let img = image::open(&file)?;
    let new_path = utils::extract_file_name(&file);
    let new_file = format!("{}.{}", &new_path, target);
    match target.to_lowercase().as_str() {
        "jpg" | "jpeg" => {
            img.save_with_format(new_file.clone(), image::ImageFormat::Jpeg)?;
            println!("'{}' successfully converted to '{}'!", &file, &new_file);
        },
        "png" => {
            img.save_with_format(new_file.clone(), image::ImageFormat::Png)?;
            println!("'{}' successfully converted to '{}'!", &file, &new_file);
        },
        _ => return Err(Box::new(UnsupportedFormatError { format: target })),
    };
    return Ok(());
}