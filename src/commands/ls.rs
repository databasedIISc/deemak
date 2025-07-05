use super::argparser::ArgParser;
use super::whereami::display_relative_path;
use crate::utils::log;
use std::path::Path;

pub const HELP_TXT: &str = r#"
Usage: ls [directory]

Lists the objects and places you can go to in the specified(current by default) directory.
"#;

/// Lists all files and directories in the given path, excluding .dir_info and info.json
/// Returns a tuple of (files, directories) as String vectors
pub fn list_directory_entries(target_path: &Path, root_dir: &Path) -> (Vec<String>, Vec<String>) {
    let entries = match std::fs::read_dir(target_path) {
        Ok(entries) => entries,
        Err(_) => return (Vec::new(), Vec::new()), // Error handling remains in main function
    };

    let mut files = Vec::new();
    let mut directories = Vec::new();

    for entry in entries {
        let Ok(entry) = entry else { continue };
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        if name == ".dir_info" || name == "info.json" {
            continue;
        }
        if path.is_dir() {
            directories.push(format!("{}/", name));
        } else {
            files.push(name);
        }
    }
    // Sort files and directories
    files.sort();
    directories.sort();

    (files, directories)
}

pub fn ls(args: &[&str], current_dir: &Path, root_dir: &Path) -> String {
    // Create parser with no flags (since ls doesn't need any) and our help text
    let args_string: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    let mut parser = ArgParser::new(&[]);

    // Parse arguments
    match parser.parse(&args_string) {
        Ok(_) => {
            let positional_args = parser.get_positional_args();

            // Determine target path with security checks
            let target_path = if positional_args.is_empty() {
                current_dir.to_path_buf()
            } else {
                if positional_args.len() > 1 {
                    return "ls: too many arguments\nTry 'help ls' for more information."
                        .to_string();
                }

                if positional_args[0] == ".dir_info" {
                    log::log_warning("ls", "Attempted to read .dir_info directory.");
                    return "ls: cannot list .dir_info directory. Access now allowed to ANY user"
                        .to_string();
                }

                let joined = current_dir.join(positional_args[0]);
                if joined.starts_with(root_dir) {
                    joined
                } else {
                    return "ls: Access denied outside root directory".to_string();
                }
            };

            // Read directory entries
            let (files_vec, directories_vec) = list_directory_entries(&target_path, root_dir);

            if files_vec.is_empty() && directories_vec.is_empty() {
                if let Err(e) = std::fs::read_dir(&target_path) {
                    let mut error_msg = e.to_string();
                    if e.kind() == std::io::ErrorKind::NotFound {
                        error_msg = "No such file or directory".to_string();
                    }
                    return format!(
                        "ls: cannot access '{}': {}",
                        display_relative_path(&target_path, root_dir),
                        error_msg
                    );
                }
            }

            // Format the output strings
            let files = if files_vec.is_empty() {
                "   (none)\n".to_string()
            } else {
                files_vec.iter().map(|f| format!("   {}\n", f)).collect()
            };

            let directories = if directories_vec.is_empty() {
                "   (none)\n".to_string()
            } else {
                directories_vec
                    .iter()
                    .map(|d| format!("   {}\n", d))
                    .collect()
            };

            // Displaying files and directories
            // Format output with relative path header
            format!(
                "\nObjects:\n{files}\nFrom inside here, you can go to:\n{directories}",
                files = files,
                directories = directories
            )
        }
        Err(e) => match &e[..] {
            "help" => HELP_TXT,
            "unknown" => "ls: unknown flag\nTry 'help ls' for more information.",
            _ => "Error parsing arguments. Try 'help ls' for more information.",
        }
        .to_string(),
    }
}
