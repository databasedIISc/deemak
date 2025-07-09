use super::argparser::ArgParser;
use super::cmds::{check_dir_info, normalize_path};
use super::display_relative_path;
use crate::utils::log;
use crate::utils::prompt::UserPrompter;
use std::fs;
use std::path::{Path, PathBuf};

pub const HELP_TXT: &str = r#"
Usage: del [OPTIONS] [PATH]

Delete file or directory in the current directory. By default, deletes a file.
Options:
    -d, --dir       Delete a directory (must be empty unless -f is used)
    -f, --force     Force delete (recursively delete directories)
    -h, --help      Display this help message

Examples:
- del file.txt
- del -d empty_directory
- del -f directory_with_contents
"#;

/// Delete a file at the given path
pub fn delete_file(path: &Path, root_dir: &Path) -> String {
    if !path.exists() {
        return format!(
            "del: {}: No such file",
            display_relative_path(path, root_dir)
        );
    }

    if path.is_dir() {
        return format!(
            "del: {}: Is a directory (use -d flag)",
            display_relative_path(path, root_dir)
        );
    }

    match std::fs::remove_file(path) {
        Ok(_) => format!("Deleted file: {}", display_relative_path(path, root_dir)),
        Err(e) => format!("del: {}: {}", display_relative_path(path, root_dir), e),
    }
}

/// Delete a directory at the given path
pub fn delete_directory(path: &Path, root_dir: &Path, force: bool) -> String {
    if !path.exists() {
        return format!(
            "del: {}: No such directory",
            display_relative_path(path, root_dir)
        );
    }
    if !path.is_dir() {
        return format!(
            "del: {}: Not a directory",
            display_relative_path(path, root_dir)
        );
    }

    // Check if directory only contains .dir_info
    let only_has_dir_info = || {
        if let Ok(entries) = fs::read_dir(path) {
            entries.count() == 1 && path.join(".dir_info").exists()
        } else {
            false
        }
    };

    // Handle special case for just .dir_info
    if only_has_dir_info() {
        let _ = fs::remove_dir_all(path.join(".dir_info"));
        return match fs::remove_dir(path) {
            Ok(_) => format!(
                "Deleted directory: {}",
                display_relative_path(path, root_dir)
            ),
            Err(e) => format!("del: {}: {}", display_relative_path(path, root_dir), e),
        };
    }

    // Normal deletion
    let result = if force {
        fs::remove_dir_all(path)
    } else {
        fs::remove_dir(path)
    };

    match result {
        Ok(_) => format!(
            "Deleted directory: {}",
            display_relative_path(path, root_dir)
        ),
        Err(e) => format!("del: {}: {}", display_relative_path(path, root_dir), e),
    }
}

/// Checks the validity of the deletion path
fn validate_deletion_path(
    path: &Path,
    current_dir: &Path,
    root_dir: &Path,
) -> Result<PathBuf, String> {
    let mut full_path = current_dir.join(path);
    full_path = normalize_path(&full_path);

    log::log_debug("del", &format!("Path to delete: {}", full_path.display()));

    // Verify the path is within root directory
    if !full_path.starts_with(root_dir) {
        log::log_warning("del", "Attempted to delete outside of root directory");
        return Err(format!(
            "del: {}: Cannot delete outside of root directory",
            path.display()
        ));
    }

    // Check if path exists
    if !full_path.exists() {
        log::log_warning("del", "Path does not exist");
        return Err(format!(
            "del: {}: No such file or directory",
            path.display()
        ));
    }

    Ok(full_path)
}

/// Main delete command function
pub fn del(
    args: &[&str],
    current_dir: &Path,
    root_dir: &Path,
    prompter: &mut dyn UserPrompter,
) -> String {
    let mut parser = ArgParser::new(&["-d", "--dir", "-f", "--force"]);

    let args_string: Vec<String> = args.iter().map(|s| s.to_string()).collect();

    log::log_debug(
        "del",
        &format!(
            "Parsing arguments: {:?}, Current Directory: {}",
            args_string,
            current_dir.display(),
        ),
    );

    match parser.parse(&args_string, "del") {
        Ok(_) => {
            let destination = args
                .iter()
                .find(|&&arg| !arg.starts_with('-'))
                .unwrap_or(&"");

            if destination.is_empty() {
                return "del: No destination specified. Use 'del --help' for usage.".to_string();
            }
            if check_dir_info(Path::new(destination)) {
                return "del: Cannot delete/refer restricted file or directory. Operation Not Allowed."
                    .to_string();
            }

            if !prompter.confirm(&format!(
                "Are you sure you want to delete '{}'? This action cannot be undone.",
                destination
            )) {
                return "Deletion cancelled by user.".to_string();
            }

            let destination_path = Path::new(destination);
            match validate_deletion_path(destination_path, current_dir, root_dir) {
                Ok(full_path) => {
                    let force = args.contains(&"-f") || args.contains(&"--force");

                    if args.contains(&"-d") || args.contains(&"--dir") || full_path.is_dir() {
                        delete_directory(&full_path, root_dir, force)
                    } else {
                        delete_file(&full_path, root_dir)
                    }
                }
                Err(e) => e,
            }
        }
        Err(e) => match &e[..] {
            "help" => HELP_TXT.to_string(),
            "unknown" => "del: unknown flag\nTry 'help del' for more information.".to_string(),
            _ => "Error parsing arguments. Try 'help del' for more information.".to_string(),
        },
    }
}
