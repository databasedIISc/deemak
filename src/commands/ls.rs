use super::argparser::ArgParser;
use super::cmds::check_dir_info;
use super::whereami::display_relative_path;
use crate::metainfo::lock_perm;
use crate::utils::log;
use std::path::Path;

pub const HELP_TXT: &str = r#"
Usage: ls [directory]

Lists the objects and places you can go to in the specified(current by default) directory.
Example:
- ls                        : Lists the contents of the current directory. 
- ls directory_name         : Lists the contents of specified directory
"#;

/// Lists all files and directories in the given path, excluding .dir_info and info.json
/// Returns a tuple of (files, directories) as String vectors with lock status
pub fn list_directory_entries(target_path: &Path, root_dir: &Path) -> (Vec<String>, Vec<String>) {
    let entries = match std::fs::read_dir(target_path) {
        Ok(entries) => entries,
        Err(_) => return (Vec::new(), Vec::new()),
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
            directories.push(name.to_string());
        } else {
            files.push(name);
        }
    }

    files.sort();
    directories.sort();
    (files, directories)
}

pub fn ls(args: &[&str], current_dir: &Path, root_dir: &Path) -> String {
    let args_string: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    let mut parser = ArgParser::new(&[]);

    match parser.parse(&args_string, "ls") {
        Ok(_) => {
            let positional_args = parser.get_positional_args();

            // Handle directory argument
            if positional_args.len() > 1 {
                return "ls: too many arguments\nTry 'help ls' for more information.".to_string();
            }

            let target_path = if positional_args.is_empty() {
                current_dir.to_path_buf()
            } else {
                let dir_name = positional_args[0];

                // Check if directory is locked
                let dir_path = current_dir.join(dir_name);
                if let Ok((_, is_locked)) = lock_perm::read_lock_perm(&dir_path) {
                    if is_locked {
                        return format!(
                            "{} is locked. To list contents, unlock it first.",
                            dir_name
                        );
                    }
                }

                if check_dir_info(Path::new(dir_name)) {
                    log::log_warning(
                        "ls",
                        &format!(
                            "Attempted to list restricted directory: {} Operation Not Permitted",
                            dir_name
                        ),
                    );
                    return format!(
                        "Attempted to list restricted directory: {} Operation Not Permitted",
                        dir_name
                    );
                }

                let joined = current_dir.join(dir_name);
                if joined.starts_with(root_dir) {
                    joined
                } else {
                    return "ls: Access denied outside root directory".to_string();
                }
            };

            let (files_vec, directories_vec) = list_directory_entries(&target_path, root_dir);

            if files_vec.is_empty() && directories_vec.is_empty() {
                if let Err(e) = std::fs::read_dir(&target_path) {
                    let error_msg = if e.kind() == std::io::ErrorKind::NotFound {
                        "No such file or directory".to_string()
                    } else {
                        e.to_string()
                    };
                    return format!(
                        "ls: cannot access '{}': {}",
                        display_relative_path(&target_path, root_dir),
                        error_msg
                    );
                }
            }
            // Check lock status
            // Check lock status and format display names
            let files = if files_vec.is_empty() {
                "   (none)\n".to_string()
            } else {
                files_vec
                    .iter()
                    .map(|f| {
                        let is_locked = match lock_perm::read_lock_perm(&current_dir.join(f)) {
                            Ok((_, locked)) => locked,
                            Err(_) => false,
                        };
                        format!("   {}{}\n", f, if is_locked { " (locked)" } else { "" })
                    })
                    .collect()
            };

            let directories = if directories_vec.is_empty() {
                "   (none)\n".to_string()
            } else {
                directories_vec
                    .iter()
                    .map(|d| {
                        let dir_name = d.trim_end_matches('/');
                        let is_locked = match lock_perm::read_lock_perm(&current_dir.join(dir_name))
                        {
                            Ok((_, locked)) => locked,
                            Err(_) => false,
                        };
                        format!("   {}{}\n", d, if is_locked { " (locked)" } else { "" })
                    })
                    .collect()
            };

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
