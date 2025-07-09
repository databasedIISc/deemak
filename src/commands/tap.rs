use super::argparser::ArgParser;
use super::cmds::{check_dir_info, normalize_path};
use super::display_relative_path;
use crate::utils::log;
use crate::utils::valid_sekai::create_dir_info;
use std::path::{Path, PathBuf};

pub const HELP_TXT: &str = r#"
Usage: tap [OPTIONS] [NAME]

Create file or directory in the current directory. By default, creates a file.
Options:
    -d, --dir       Create a directory
    -h, --help      Display this help message

Examples:
- tap new_file
- tap -d new_directory
- tap new_dir/    # usage of trailing slash to create a directory
"#;

pub fn create_file(destination: &str, current_dir: &Path, root_dir: &Path) -> String {
    let new_path: &Path = &current_dir.join(destination);

    // Check if the path already exists
    if new_path.exists() {
        return format!("tap: {}: File or directory already exists", destination);
    }

    // Create the file or directory
    match std::fs::File::create(new_path) {
        Ok(_) => format!(
            "Created file: {}",
            display_relative_path(new_path, root_dir)
        ),
        Err(e) => format!("tap: {}: {}", destination, e),
    }
}

pub fn create_directory(destination: &str, current_dir: &Path, root_dir: &Path) -> String {
    let new_path: &PathBuf = &current_dir.join(destination);

    // Check if the path already exists
    if new_path.exists() {
        return format!(
            "tap: {}: Directory already exists",
            display_relative_path(new_path, root_dir)
        );
    }

    // Create the directory
    match std::fs::create_dir(new_path) {
        Ok(_) => {
            // create .dir_info automatically
            if !create_dir_info(new_path, false) {
                return format!(
                    "tap: Failed to create .dir_info: {}",
                    display_relative_path(new_path, root_dir)
                );
            }
            format!(
                "Created directory: {}",
                display_relative_path(new_path, root_dir)
            )
        }
        Err(e) => format!("tap: {}: {}", display_relative_path(new_path, root_dir), e),
    }
}

/// Checks the validity of the destination path based on root_dir and current_dir.
fn handle_destination(
    destination: &Path,
    current_dir: &Path,
    root_dir: &Path,
) -> Result<PathBuf, String> {
    // Get absolute path by joining with current_dir and normalizing
    let mut new_path = current_dir.join(destination);
    new_path = normalize_path(&new_path);
    log::log_debug("tap", &format!("New Path to tap: {}", new_path.display()));

    // Verify the path is within root directory
    if !new_path.starts_with(root_dir) {
        log::log_warning("tap", "Attempted to create outside of root directory");
        return Err(format!(
            "tap: {}: Cannot create outside of root directory",
            destination.display()
        ));
    }
    // Check if path already exists
    if new_path.exists() {
        log::log_warning("tap", "File or directory already exists");
        return Err(format!(
            "tap: {}: File or directory already exists",
            destination.display()
        ));
    }
    log::log_info("tap", &format!("Valid path: {}", new_path.display()));
    Ok(new_path)
}

// Check if the destination is within the root directory
pub fn tap(args: &[&str], current_dir: &Path, root_dir: &Path) -> String {
    let valid_flags = ["-d", "--dir", "-h", "--help"];
    let mut parser = ArgParser::new(&valid_flags);

    let args_string: Vec<String> = args.iter().map(|s| s.to_string()).collect();

    log::log_debug(
        "tap",
        &format!(
            "Parsing arguments: {:?}, Current Directory: {}",
            args_string,
            current_dir.display(),
        ),
    );
    match parser.parse(&args_string, "tap") {
        Ok(_) => {
            let mut destination = args
                .iter()
                .find(|&&arg| !valid_flags.contains(&arg))
                .unwrap_or(&"") as &str;

            if destination.is_empty() {
                return "tap: No destination specified. Use 'tap --help' for usage.".to_string();
            }
            if check_dir_info(Path::new(destination)) {
                return "tap: Cannot create/refer restricted files or directory. Operation Not Allowed."
                    .to_string();
            }
            // handle destination path
            let destination_path = Path::new(destination);
            if handle_destination(destination_path, current_dir, root_dir).is_err() {
                // the error string will be printed
                return handle_destination(destination_path, current_dir, root_dir).unwrap_err();
            } else {
                // This is relative valid path
                destination = destination_path.to_str().unwrap();
            }

            if args.contains(&"-d") || args.contains(&"--dir") {
                destination = destination.trim_end_matches('/');
                // Create a directory
                create_directory(destination, current_dir, root_dir)
            } else if destination.ends_with('/') {
                create_directory(destination.trim_end_matches('/'), current_dir, root_dir)
            } else {
                // Create a file
                create_file(destination, current_dir, root_dir)
            }
        }
        Err(e) => match &e[..] {
            "help" => HELP_TXT.to_string(),
            "unknown" => "tap: unknown flag\nTry 'help tap' for more information.".to_string(),
            _ => "Error parsing arguments. Try 'help tap' for more information.".to_string(),
        },
    }
}
