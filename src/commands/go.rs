use super::whereami::display_relative_path;
use crate::info_utils::info_reader;
use std::path::{Path, PathBuf};

pub fn go(args: &[&str], current_dir: &mut PathBuf, root_dir: &Path) -> String {
    if args.is_empty() {
        return "go: missing directory operand".to_string();
    }

    let target = args[0];
    let new_path = match target {
        "HOME" => root_dir.to_path_buf(),
        "back" | ".." => {
            if current_dir == root_dir {
                return "You are at the root. You cannot go back anymore".to_string();
            }
            current_dir.parent().unwrap().to_path_buf()
        }
        _ => current_dir.join(target),
    };

    // Normalize path and check boundaries
    let new_path = match new_path.canonicalize() {
        Ok(p) if p.starts_with(root_dir) => p,
        Ok(_) => return "Access denied outside root directory".to_string(),
        Err(_) => return format!("go: {}: No such directory", target),
    };

    if !new_path.is_dir() {
        return format!("go: {}: Not a directory", target);
    }

    // Update directory and get info
    *current_dir = new_path.clone();
    let info_path = new_path.join("info.json");

    match info_reader::read_info(&info_path) {
        Ok(info) => format!(
            "You are now in {}\n\nAbout:\n{}\n",
            display_relative_path(&new_path, root_dir),
            info.about.trim_matches('"')
        ),
        Err(_) => format!("Entered {}", display_relative_path(&new_path, root_dir)),
    }
}
