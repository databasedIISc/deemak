use crate::metainfo::info_reader::{InfoError, read_validate_info};
use crate::utils::{globals, log};
use std::path::{Path, PathBuf};

/// Find the root directory of a sekai by finding "location": "home"
/// in nearest `.dir_info/info.json`
pub fn check_home(sekai_path: &Path) -> Result<Option<PathBuf>, InfoError> {
    let mut current = sekai_path.to_path_buf();
    // Check for info.json in current directory
    let info_path = current.join(".dir_info/info.json");
    match read_validate_info(&info_path) {
        Ok(info) => {
            if info.location == "HOME" {
                Ok(Some(current))
            } else {
                log::log_warning(
                    "SEKAI",
                    &format!(
                        "Found info.json at {}, but location is not 'HOME': {}",
                        info_path.display(),
                        info.location
                    ),
                );
                Ok(None)
            }
        }
        Err(InfoError::NotFound(_)) => {
            log::log_warning(
                "SEKAI",
                &format!(
                    "No info.json found at {}, checking parent directory",
                    info_path.display()
                ),
            );
            Ok(None)
        }
        Err(e) => Err(e),
    }
}

/// Returns the home directory
pub fn get_home(sekai_path: &Path) -> Option<PathBuf> {
    match check_home(sekai_path) {
        Ok(Some(home)) => Some(home),
        Ok(None) => None,
        Err(e) => {
            log::log_error("SEKAI", &format!("Error finding Sekai home: {e}"));
            None
        }
    }
}

/// Get the relative deemak path for a given path w.r.t sekai root directory.
pub fn relative_deemak_path(path: &Path, sekai_root_dir: Option<&Path>) -> PathBuf {
    let _sekai_dir = if let Some(dir) = sekai_root_dir {
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
