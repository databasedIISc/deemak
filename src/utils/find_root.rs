use crate::metainfo::info_reader::read_validate_info;
use crate::utils::globals;
use std::path::{Path, PathBuf};

/// Find the root directory of a sekai by finding "location": "home"
/// in nearest `.dir_info/info.json` without going outside the starting directory
pub fn find_home(sekai_path: &Path) -> Option<PathBuf> {
    let mut current = sekai_path.to_path_buf();
    let max_depth = 100; // Prevent infinite recursion
    let mut depth = 0;

    while depth < max_depth {
        // Check for info.json in current directory
        let info_path = current.join(".dir_info/info.json");
        if let Ok(info) = read_validate_info(&info_path) {
            if info.location == "HOME" {
                return Some(current);
            }
        }

        // Check subdirectories
        if let Ok(entries) = std::fs::read_dir(&current) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() && path.file_name() != Some(std::ffi::OsStr::new(".dir_info")) {
                    let sub_info_path = path.join(".dir_info/info.json");
                    if let Ok(info) = read_validate_info(&sub_info_path) {
                        if info.location == "HOME" {
                            // The message for home found exists in `main.rs`
                            return Some(path);
                        }
                    }
                }
            }
        }

        // Only move up if we're not already at the starting directory
        if current == sekai_path {
            break;
        }

        if !current.pop() {
            break;
        }
        depth += 1;
    }

    // The message for no home found exists in `main.rs`
    None
}

/// Converts an absolute path to a path relative to WORLD_DIR
/// Returns the original path if WORLD_DIR isn't set or if the path isn't within WORLD_DIR
pub fn relative_deemak_path(path: &Path) -> PathBuf {
    let world_dir = globals::get_world_dir();

    path.strip_prefix(&world_dir)
        .map(|relative_path| {
            if relative_path.components().count() == 0 {
                PathBuf::from("HOME")
            } else {
                PathBuf::from("HOME").join(relative_path)
            }
        })
        .unwrap_or_else(|_| path.to_path_buf())
}
