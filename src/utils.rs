use std::{path::Path, fs};

pub fn exists(s: &String) -> bool {
    return Path::new(s).exists()
}

pub fn read_pid(s: &String) -> Option<i32> {
    fs::read_to_string(Path::new(s))
        .expect(format!("Should have been able to read the file: {}", s).as_str())
        .parse::<i32>()
        .ok()
}
