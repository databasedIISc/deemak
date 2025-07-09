use super::cmds::check_dir_info;
use super::whereami::display_relative_path;
use std::fs;
use std::path::Path;

pub const HELP_TXT: &str = r#"
Usage: read <object_name>

Read the contents of a file and display it.
"#;

/// Read and display file contents (similar to 'cat' command)
pub fn read(args: &[&str], current_dir: &Path, root_dir: &Path) -> String {
    if args.is_empty() {
        return "read: missing file operand".to_string();
    }

    let file_path = current_dir.join(args[0]);

    // Check if file is doesn't refer a restricted one
    if check_dir_info(&file_path) {
        return format!(
            "read: Attempted to read/refer restricted files: {} Operation Not Permitted",
            display_relative_path(&file_path, root_dir)
        );
    }

    // Security check - must stay within root directory
    if !file_path.starts_with(root_dir) {
        return "read: Access denied outside root directory".to_string();
    }

    // Check if path is a directory
    if file_path.is_dir() {
        return format!(
            "read: {}: Is a directory",
            display_relative_path(&file_path, root_dir)
        );
    }

    match fs::read_to_string(&file_path) {
        Ok(content) => content,
        Err(e) => format!(
            "read: {}: {}",
            display_relative_path(&file_path, root_dir),
            match e.kind() {
                std::io::ErrorKind::NotFound => "No such file",
                std::io::ErrorKind::PermissionDenied => "Permission denied",
                _ => "Could not read file",
            }
        ),
    }
}
