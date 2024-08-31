use log::{debug, info};
use std::fs::metadata;
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Write},
    path::Path,
};

/// Creates a file if it does not already exist.
/// If the file already exists, it doesn't modify its content.
pub fn create_file(path: &Path, content: &str) -> io::Result<()> {
    match OpenOptions::new().write(true).create_new(true).open(path) {
        Ok(mut file) => file.write_all(content.as_bytes()),
        Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
            debug!(
                "File '{}' already exists. Not altering it.",
                path.to_string_lossy()
            );
            Ok(())
        }
        Err(e) => Err(e),
    }
}
///Create_file with no content...
pub fn create_file_nn(path: &Path) -> io::Result<()> {
    // Verify if the file does not already exist.
    if metadata(path).is_ok() {
        println!(
            "File '{}' already exists. Not altering it.",
            path.to_string_lossy()
        );
        return Ok(());
    }

    // Create the file.
    match std::fs::File::create(path) {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

/// Creates a file given its path.
pub fn create_dir(path: &Path) -> io::Result<()> {
    fs::create_dir(path)?;
    info!("Created directory: '{}'", path.to_string_lossy());
    Ok(())
}

/// Opens an already existing file and overwrites all content with `content`.
pub fn overwrite_file(path: &Path, content: &str) -> std::io::Result<()> {
    let mut file = File::create(path)?;
    debug!("Overwrote file '{}'", path.to_string_lossy());
    file.write_all(content.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;
    use tempfile::TempDir;

    #[test]
    fn test_overwrite_file() {
        // Create a temporary file
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let file_path = temp_file.path();

        // Test 1: Write to the file
        let content = "Hello, World!";
        overwrite_file(file_path, content).expect("Failed to write to file");
        assert_eq!(fs::read_to_string(file_path).unwrap(), content);

        // Test 2: Overwrite existing content
        let new_content = "New content";
        overwrite_file(file_path, new_content).expect("Failed to overwrite file");
        assert_eq!(fs::read_to_string(file_path).unwrap(), new_content);

        // Test 3: Write empty string
        overwrite_file(file_path, "").expect("Failed to write empty string");
        assert_eq!(fs::read_to_string(file_path).unwrap(), "");

        // Test 4: Write unicode content
        let unicode_content = "こんにちは世界";
        overwrite_file(file_path, unicode_content).expect("Failed to write unicode content");
        assert_eq!(fs::read_to_string(file_path).unwrap(), unicode_content);
    }

    #[test]
    fn test_create_file() -> io::Result<()> {
        let temp_dir = TempDir::new()?;

        // Test 1: Create a new file
        let file_path = temp_dir.path().join("test1.txt");
        let content = "Hello, World!";
        create_file(&file_path, content)?;
        assert!(file_path.exists());
        assert_eq!(fs::read_to_string(&file_path)?, content);

        // Test 2: Attempt to create an existing file (should not modify)
        let existing_content = fs::read_to_string(&file_path)?;
        create_file(&file_path, "New content")?;
        assert_eq!(fs::read_to_string(&file_path)?, existing_content);

        // Test 3: Create file with empty content
        let empty_file_path = temp_dir.path().join("empty.txt");
        create_file(&empty_file_path, "")?;
        assert!(empty_file_path.exists());
        assert_eq!(fs::read_to_string(&empty_file_path)?, "");

        Ok(())
    }
}
