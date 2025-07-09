use super::argparser::ArgParser;
use super::cmds::check_dir_info;
use super::whereami::display_relative_path;
use crate::metainfo::{info_reader, lock_perm};
use crate::utils::log;
use std::path::{Path, PathBuf};

pub const HELP_TXT: &str = r#"
Usage: go [destination]

Navigate to different directories:
- go <dirname>    : Enter specified directory
- go HOME         : Return to home directory
- go ..           : Move up one directory
- go back         : Same as `go ..`

"#;

pub fn navigate(destination: &str, current_dir: &PathBuf, root_dir: &Path) -> (PathBuf, String) {
    let new_path = match destination {
        "HOME" | "home" => root_dir.to_path_buf(),
        ".." | "back" => {
            if current_dir == root_dir {
                log::log_warning("go", "Attempted to go back from root directory");
                return (
                    current_dir.clone(),
                    "You are at the root. Cannot go back further".to_string(),
                );
            }
            current_dir.parent().unwrap().to_path_buf()
        }
        _ => {
            if check_dir_info(Path::new(destination)) {
                log::log_warning(
                    "go",
                    &format!(
                        "Attempted to go to/refers a restricted directory: {}. Operation Not Permitted.",
                        destination
                    ),
                );
                return (
                    current_dir.clone(),
                    format!(
                        "go: Attempted to go to/refers a restricted directory: {}. Operation Not Permitted",
                        destination
                    ),
                );
            } else {
                current_dir.join(destination)
            }
        }
    };

    // Normalize path and verify it exists
    let canonical_path = match new_path.canonicalize() {
        Ok(p) => p,
        Err(_) => {
            log::log_error(
                "go",
                &format!("No such directory, path: {}", new_path.display()),
            );
            return (
                current_dir.clone(),
                format!("go: {}: No such directory", destination),
            );
        }
    };

    // Verify it's within root and is a directory
    if !canonical_path.starts_with(root_dir) {
        log::log_warning(
            "go",
            &format!(
                "Access denied: Attempted to go outside root directory: {}",
                canonical_path.display()
            ),
        );
        return (
            current_dir.clone(),
            "go: Access denied: Cannot go outside root".to_string(),
        );
    }

    if !canonical_path.is_dir() {
        if canonical_path.is_file() {
            log::log_warning(
                "go",
                &format!(
                    "Attempted to go to a file instead of a directory: {}",
                    canonical_path.display()
                ),
            );
            return (
                current_dir.clone(),
                format!(
                    "go: {}: Is a file (try 'read {}')",
                    destination, destination
                ),
            );
        }

        log::log_warning(
            "go",
            &format!(
                "Attempted to go to a non-directory path: {}",
                canonical_path.display()
            ),
        );
        return (
            current_dir.clone(),
            format!("go: {}: Not a directory", destination),
        );
    }

    // Check if directory is locked
    if let Err(e) = lock_perm::operation_locked_perm(
        &canonical_path,
        "go",
        "Cannot enter locked directory. Unlock it first",
    ) {
        return (current_dir.clone(), e);
    }

    // Get directory info if available
    let info_path = canonical_path.join(".dir_info/info.json");
    let message = match info_reader::read_validate_info(&info_path) {
        Ok(info) => format!(
            "You have entered {}\n\nAbout:\n{}",
            display_relative_path(&canonical_path, root_dir),
            info.about.trim_matches('"')
        ),
        Err(_) => format!(
            "You have entered {}\n\nNo additional information available.",
            display_relative_path(&canonical_path, root_dir)
        ),
    };

    (canonical_path, message)
}

pub fn go(args: &[&str], current_dir: &PathBuf, root_dir: &Path) -> (PathBuf, String) {
    let mut parser = ArgParser::new(&[]);

    let args_string: Vec<String> = args.iter().map(|s| s.to_string()).collect();

    log::log_debug(
        "go",
        &format!(
            "Parsing arguments: {:?}, Current Directory: {}, Root Directory: {}",
            args_string,
            current_dir.display(),
            root_dir.display()
        ),
    );
    // Parse arguments
    match parser.parse(&args_string, "go") {
        Ok(_) => {
            let pos_args = parser.get_positional_args();

            if pos_args.is_empty() {
                return (
                    current_dir.clone(),
                    "go: missing directory operand\nTry 'help go' for more information."
                        .to_string(),
                );
            }
            if pos_args.len() > 1 {
                return (
                    current_dir.clone(),
                    "go: too many arguments\nTry 'help go' for more information.".to_string(),
                );
            }
            let target = pos_args[0].as_str();
            navigate(target, current_dir, root_dir)
        }
        Err(e) => match &e[..] {
            "help" => (current_dir.clone(), HELP_TXT.to_string()),
            "unknown" => (
                current_dir.clone(),
                "go: unknown flag\nTry 'help go' for more information.".to_string(),
            ),
            _ => (
                current_dir.clone(),
                "Error parsing arguments. Try 'help go' for more information.".to_string(),
            ),
        },
    }
}
