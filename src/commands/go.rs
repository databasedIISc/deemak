use super::argparser::ArgParser;
use super::whereami::display_relative_path;
use crate::utils::info_reader;
use std::{
    env::current_dir,
    path::{Path, PathBuf},
};

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
                return (
                    current_dir.clone(),
                    "You are at the root. Cannot go back further".to_string(),
                );
            }
            current_dir.parent().unwrap().to_path_buf()
        }
        _ => current_dir.join(destination),
    };

    // Normalize path and verify it exists
    let canonical_path = match new_path.canonicalize() {
        Ok(p) => p,
        Err(_) => {
            return (
                current_dir.clone(),
                format!("go: {}: No such directory", destination),
            );
        }
    };

    // Verify it's within root and is a directory
    if !canonical_path.starts_with(root_dir) {
        return (
            current_dir.clone(),
            "Access denied: Cannot go outside root".to_string(),
        );
    }

    if !canonical_path.is_dir() {
        if canonical_path.is_file() {
            return (
                current_dir.clone(),
                format!(
                    "go: {}: Is a file (try 'read {}')",
                    destination, destination
                ),
            );
        }
        return (
            current_dir.clone(),
            format!("go: {}: Not a directory", destination),
        );
    }

    // Get directory info if available
    let info_path = canonical_path.join("info.json");
    let message = match info_reader::read_info(&info_path) {
        Ok(info) => format!(
            "You have entered {}\n\nAbout:\n{}",
            display_relative_path(&canonical_path, root_dir),
            info.about.trim_matches('"')
        ),
        Err(_) => format!(
            "Entered {}",
            display_relative_path(&canonical_path, root_dir)
        ),
    };

    (canonical_path, message)
}

pub fn go(args: &[&str], current_dir: &PathBuf, root_dir: &Path) -> (PathBuf, String) {
    let mut parser = ArgParser::new(&[]);

    let args_string: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    // Parse arguments
    match parser.parse(&args_string) {
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
                "ls: unknown flag\nTry 'help go' for more information.".to_string(),
            ),
            _ => (
                current_dir.clone(),
                "Error parsing arguments. Try 'help ls' for more information.".to_string(),
            ),
        },
    }
}
