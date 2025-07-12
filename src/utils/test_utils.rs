use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use walkdir::WalkDir;

/// Creates file at the specified path with the given content.
pub fn create_file<P: AsRef<Path>>(path: P, content: &str) {
    let mut file = File::create(path).unwrap();
    writeln!(file, "{content}").unwrap();
}

/// Removes the file at the specified path.
pub fn remove_file<P: AsRef<Path>>(path: P) {
    fs::remove_file(path).unwrap();
}

/// Helper to create a test directory wi/// Creates a temporary directory with the following structure:
/// Creates a temporary directory with the following structure:
/// temp_dir/
/// ├── file1.txt
/// ├── subdir1/
/// │   ├── file2.txt
/// │   ├── file3.txt
/// │   └── nested1/
/// │       └── file4.txt
/// └── subdir2/
///     ├── file5.txt
///     └── nested2/
///         ├── file6.txt
///         └── file7.txt
pub fn setup_test_dir() -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().unwrap();
    let root_path = temp_dir.path().to_path_buf();

    create_file(root_path.join("file1.txt"), "hello from file1");

    let subdir1_path = root_path.join("subdir1");
    fs::create_dir(&subdir1_path).unwrap();
    create_file(subdir1_path.join("file2.txt"), "hello from file2");
    create_file(subdir1_path.join("file3.txt"), "hello from file3");

    let nested1_path = subdir1_path.join("nested1");
    fs::create_dir(&nested1_path).unwrap();
    create_file(nested1_path.join("file4.txt"), "hello from file4");

    let subdir2_path = root_path.join("subdir2");
    fs::create_dir(&subdir2_path).unwrap();
    create_file(subdir2_path.join("file5.txt"), "hello from file5");

    let nested2_path = subdir2_path.join("nested2");
    fs::create_dir(&nested2_path).unwrap();
    create_file(nested2_path.join("file6.txt"), "hello from file6");
    create_file(nested2_path.join("file7.txt"), "hello from file7");

    (temp_dir, root_path)
}

fn get_text_files_contents(path: &Path) -> HashMap<PathBuf, String> {
    let mut contents = HashMap::new();
    for entry in WalkDir::new(path) {
        let entry = entry.unwrap();
        let entry_path = entry.path();
        if entry_path.is_file() {
            let mut file = fs::File::open(entry_path).unwrap();
            let mut buffer = Vec::new();
            // Try to read to end, and then check if it's valid utf8
            if file.read_to_end(&mut buffer).is_ok() {
                if let Ok(content) = String::from_utf8(buffer) {
                    let relative_path = entry_path.strip_prefix(path).unwrap().to_path_buf();
                    contents.insert(relative_path, content.trim().to_string());
                }
            }
        }
    }
    contents
}

/// Helper to get directory contents as a HashMap for comparison.
pub fn get_dir_contents(path: &PathBuf, ignore_dir_info: bool) -> HashMap<PathBuf, String> {
    let mut contents = HashMap::new();
    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        let entry_path = entry.path();
        if entry_path.is_file() {
            if ignore_dir_info && entry_path.to_string_lossy().contains(".dir_info") {
                continue;
            }
            let mut file = fs::File::open(entry_path).unwrap();
            let mut buffer = Vec::new();
            // Try to read to end, and then check if it's valid utf8
            if file.read_to_end(&mut buffer).is_ok() {
                if let Ok(content) = String::from_utf8(buffer) {
                    let relative_path = entry_path.strip_prefix(path).unwrap().to_path_buf();
                    contents.insert(relative_path, content.trim().to_string());
                }
            } else {
                // If reading fails, we can still insert the path with an empty string
                let relative_path = entry_path.strip_prefix(path).unwrap().to_path_buf();
                contents.insert(relative_path, String::new());
            }
        }
    }
    contents
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashMap;
    use std::path::PathBuf;

    #[test]
    fn test_get_dir_contents() {
        let (_temp_dir, root_path) = setup_test_dir();
        let contents_got = get_dir_contents(&root_path, true);

        let mut expected = HashMap::new();
        expected.insert(PathBuf::from("file1.txt"), "hello from file1".to_string());
        expected.insert(
            PathBuf::from("subdir1/file2.txt"),
            "hello from file2".to_string(),
        );
        expected.insert(
            PathBuf::from("subdir1/file3.txt"),
            "hello from file3".to_string(),
        );
        expected.insert(
            PathBuf::from("subdir1/nested1/file4.txt"),
            "hello from file4".to_string(),
        );
        expected.insert(
            PathBuf::from("subdir2/file5.txt"),
            "hello from file5".to_string(),
        );
        expected.insert(
            PathBuf::from("subdir2/nested2/file6.txt"),
            "hello from file6".to_string(),
        );
        expected.insert(
            PathBuf::from("subdir2/nested2/file7.txt"),
            "hello from file7".to_string(),
        );

        assert_eq!(expected, contents_got);
    }
}
