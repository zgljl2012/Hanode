use std::path::Path;

pub fn exists(s: &String) -> bool {
    return Path::new(s).exists()
}
