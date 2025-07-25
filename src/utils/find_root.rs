use crate::metainfo::info_reader::{InfoError, read_validate_info};
use crate::utils::globals;
use crate::utils::log;
use std::path::{Path, PathBuf};

/// Find the root directory of a sekai by finding "location": "home"
/// in nearest `.dir_info/info.json` without going outside the starting directory
pub fn find_home(sekai_path: &Path) -> Result<Option<PathBuf>, InfoError> {
    let mut current = sekai_path.to_path_buf();
    let max_depth = 100; // Prevent infinite recursion
    let mut depth = 0;

    while depth < max_depth {
        // Check for info.json in current directory
        let info_path = current.join(".dir_info/info.json");
        match read_validate_info(&info_path) {
            Ok(info) => {
                if info.location == "HOME" {
                    return Ok(Some(current));
                }
            }
            Err(InfoError::NotFound(_)) => (), // Ignore not found errors
            Err(e) => return Err(e),           // Return other errors
        }

        // Check subdirectories
        if let Ok(entries) = std::fs::read_dir(&current) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() && path.file_name() != Some(std::ffi::OsStr::new(".dir_info")) {
                    let sub_info_path = path.join(".dir_info/info.json");
                    match read_validate_info(&sub_info_path) {
                        Ok(info) => {
                            if info.location == "HOME" {
                                return Ok(Some(path));
                            }
                        }
                        Err(InfoError::NotFound(_)) => (), // Ignore not found errors
                        Err(e) => return Err(e),           // Return other errors
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

    Ok(None)
}

/// Returns the home directory of a sekai if it exists
/// Use this when you have gaurantee that sekai home exists.
pub fn get_home(sekai_path: &Path) -> Option<PathBuf> {
    match find_home(sekai_path) {
        Ok(Some(home)) => Some(home),
        Ok(None) => None,
        Err(e) => {
            log::log_error("SEKAI", &format!("Error finding Sekai home: {e}"));
            None
        }
    }
}

/// Converts an absolute path to a path relative to WORLD_DIR
/// Returns the original path if WORLD_DIR isn't set or if the path isn't within WORLD_DIR
/// Also adds DEEMAK_TEMP prefix if the path is a temporary file
pub fn relative_deemak_path(path: &Path, sekai_dir: Option<&Path>) -> PathBuf {
    let _sekai_dir = if let Some(dir) = sekai_dir {
        dir
    } else {
        &globals::get_sekai_dir()
    };
    let all_temp_locs = super::cleanup::get_all_cleanup_locations();
    let temp_dir_prefix = PathBuf::from("/tmp");

    // Check if it's prefixed by world_dir
    if let Ok(relative_path) = path.strip_prefix(_sekai_dir) {
        if relative_path.components().count() == 0 {
            // Path is exactly world_dir, represent as "HOME"
            PathBuf::from("HOME")
        } else {
            PathBuf::from("HOME").join(relative_path)
        }
    } else {
        // Iterate through all_temp_locs to find a matching prefix
        for temp_loc_str in &all_temp_locs {
            let temp_prefix_path = PathBuf::from(temp_loc_str);

            if let Ok(relative_path) = path.strip_prefix(&temp_prefix_path) {
                let deemak_temp_replacement = PathBuf::from("DEEMAK_TEMP");

                if relative_path.components().count() == 0 {
                    // Path is exactly this temp_loc, replace with "DEEMAK_TEMP"
                    return deemak_temp_replacement;
                } else {
                    // Path is inside this temp_loc
                    return deemak_temp_replacement.join(relative_path);
                }
            }
        }
        // If no prefixes match, return the original path
        path.to_path_buf()
    }
}
