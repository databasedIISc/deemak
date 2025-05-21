use super::whereami::display_relative_path;
use std::path::Path;

pub const HELP_TXT: &str = r#"
Usage: ls

Lists the objects and places you can go to in the current directory.
"#;

pub fn ls(args: &[&str], current_dir: &Path, root_dir: &Path) -> String {
    // Determine target path with security checks
    let target_path = if args.is_empty() {
        current_dir.to_path_buf()
    } else {
        let joined = current_dir.join(args[0]);
        if joined.starts_with(root_dir) {
            joined
        } else {
            return "ls: Access denied outside root directory".to_string();
        }
    };

    // Read directory entries
    let entries = match std::fs::read_dir(&target_path) {
        Ok(entries) => entries,
        Err(e) => {
            return format!(
                "ls: cannot access '{}': {}",
                display_relative_path(&target_path, root_dir),
                e
            );
        }
    };

    let mut files = String::new();
    let mut directories = String::new();

    for entry in entries {
        match entry {
            Ok(entry) => {
                let path = entry.path();
                let name = entry.file_name().to_string_lossy().into_owned();

                if name == "info.json" {
                    continue; // Skip info.json
                }

                if path.is_dir() {
                    directories.push_str(&format!("   {}/\n", name));
                } else {
                    files.push_str(&format!("   {}\n", name));
                }
            }
            Err(e) => {
                return format!("Error reading entry: {}\n", e);
            }
        }
    }
    if directories.is_empty() {
        directories = "   (none)\n".to_string();
    }

    // Format output with relative path header
    format!(
        "\nObjects:\n{files}\nYou can go to:\n{directories}",
        files = files,
        directories = directories
    )
}
