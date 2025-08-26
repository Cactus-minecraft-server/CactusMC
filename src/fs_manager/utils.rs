use log::info;
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Write},
    path::Path,
};

/// Appends `content` to a file located at `path`.
pub fn append_file(path: &Path, content: &str) -> io::Result<()> {
    OpenOptions::new()
        .append(true)
        .open(path)?
        .write_all(content.as_bytes())
}

/// Creates a file if it does not already exist.
/// If the file already exists, it doesn't modify its content.
pub fn create_file(path: &Path, content: Option<&str>) -> io::Result<()> {
    match OpenOptions::new().write(true).create_new(true).open(path) {
        Ok(mut file) => content.map_or_else(
            || {
                info!("Created an empty file: '{}'", path.display());
                Ok(())
            },
            |s| {
                file.write_all(s.as_bytes())
                    .inspect(|_| info!("File '{}' created.", path.display()))
            },
        ),
        Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
            info!("File '{}' already exists. Not altering it.", path.display());
            Ok(())
        }
        Err(e) => Err(e),
    }
}

/// Creates a directory given its path.
pub fn create_dir(path: &Path) -> io::Result<()> {
    if path.exists() {
        info!(
            "Directory '{}' already exists. Not altering it.",
            path.display()
        );
        return Ok(());
    }
    fs::create_dir(path).inspect(|_| info!("Created directory: '{}'", path.display()))
}

/// Opens an already existing file and overwrites all content with `content`.
pub fn overwrite_file(path: &Path, content: &str) -> std::io::Result<()> {
    let mut file = File::create(path)?;
    file.write_all(content.as_bytes())
        .inspect(|_| info!("Overwrote file: '{}'", path.display()))
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
        create_file(&file_path, Some(content))?;
        assert!(file_path.exists());
        assert_eq!(fs::read_to_string(&file_path)?, content);

        // Test 2: Attempt to create an existing file (should not modify)
        let existing_content = fs::read_to_string(&file_path)?;
        create_file(&file_path, Some("New content"))?;
        assert_eq!(fs::read_to_string(&file_path)?, existing_content);

        // Test 3: Create file with empty content
        let empty_file_path = temp_dir.path().join("empty.txt");
        create_file(&empty_file_path, Some(""))?;
        assert!(empty_file_path.exists());
        assert_eq!(fs::read_to_string(&empty_file_path)?, "");

        Ok(())
    }
}
