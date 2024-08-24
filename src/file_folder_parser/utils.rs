use colored::Colorize;
use log::{debug, info};
use std::{
    fs::File,
    io::{self, Write},
    path::Path,
};

/// Checks the existence of a directory given its path, and logs it (debug).
pub fn check_dir(dir_path: &Path) -> bool {
    let dirpath_str: &str = &dir_path.to_string_lossy();

    if dir_path.exists() {
        debug!(
            "Folder '{}' already exists, using this one.",
            dirpath_str.red()
        );
        true
    } else {
        debug!("Folder '{}' has been created.", dirpath_str.blue());
        false
    }
}

/// Checks the existence of a file given its path, and logs it (debug).
pub fn check_file(file_path: &Path) -> bool {
    let filepath_str: &str = &file_path.to_string_lossy();

    if file_path.exists() {
        debug!(
            "File '{}' already exists, using this one.",
            filepath_str.red()
        );
        true
    } else {
        debug!("File '{}' has been created.", filepath_str.blue());
        false
    }
}

/// Creates a file given its path and its content.
pub fn create_file(path: &Path, content: &str) -> Result<(), io::Error> {
    let mut file = File::create(path)?;
    file.write_all(content.as_bytes())?;
    debug!("File '{}' has been created.", path.to_string_lossy().blue());
    Ok(())
}
