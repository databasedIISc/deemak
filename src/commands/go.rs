use super::whereami::display_relative_path;
use crate::utils::info_reader;
use std::path::{Path, PathBuf};

pub fn go(args: &[&str], current_dir: &PathBuf, root_dir: &Path) -> (PathBuf, String) {
    if args.is_empty() {
        return (
            current_dir.clone(),
            "go: missing directory operand".to_string(),
        );
    }

    let target = args[0];
    let new_path = match target {
        "HOME" | "home" => root_dir.to_path_buf(),
        ".." | "back" | "up" => {
            if current_dir == root_dir {
                return (
                    current_dir.clone(),
                    "You are at the root. Cannot go back further".to_string(),
                );
            }
            current_dir.parent().unwrap().to_path_buf()
        }
        _ => current_dir.join(target),
    };

    // Normalize path and verify it exists
    let canonical_path = match new_path.canonicalize() {
        Ok(p) => p,
        Err(_) => {
            return (
                current_dir.clone(),
                format!("go: {}: No such directory", target),
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
                format!("go: {}: Is a file (try 'read {}')", target, target),
            );
        }
        return (
            current_dir.clone(),
            format!("go: {}: Not a directory", target),
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
