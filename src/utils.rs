use std::path::PathBuf;

pub fn get_canonical_path(path: PathBuf) -> String {
    return std::fs::canonicalize(path)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
}
