use std::{path::PathBuf, process::Command};

pub fn get_canonical_path(path: PathBuf) -> String {
    return std::fs::canonicalize(path)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
}

// Opens vscode for the give path
pub fn open_code_editor(path: &str) {
    Command::new("code")
        .args([format!("{}", path)])
        .output()
        .expect("Failed to open directory");
}
