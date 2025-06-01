use super::log;
use std::path::Path;

/// Checks if the given path exists and is a directory
fn check_directory_exists(path: &Path) -> bool {
    match (path.exists(), path.is_dir()) {
        (false, _) => {
            log::log_error(
                "SEKAI",
                &format!("Directory does not exist: {}", path.display()),
            );
            false
        }
        (true, false) => {
            log::log_error(
                "SEKAI",
                &format!("Path is not a directory: {}", path.display()),
            );
            false
        }
        (true, true) => {
            log::log_info("SEKAI", &format!("Directory found: {}", path.display()));
            true
        }
    }
}

/// Checks if info.json exists in the given directory
fn check_infojson_exists(dir: &Path) -> bool {
    let info_path = dir.join("info.json");
    if !info_path.exists() {
        log::log_warning(
            "SEKAI",
            &format!("info.json not found in: {}", dir.display()),
        );
        false
    } else {
        true
    }
}

/// Recursively checks all subdirectories for valid info.json files
fn check_subdirs(path: &Path) -> bool {
    let mut all_valid = true;

    let entries = match std::fs::read_dir(path) {
        Ok(entries) => entries,
        Err(e) => {
            log::log_error(
                "SEKAI",
                &format!("Failed to read directory {}: {}", path.display(), e),
            );
            return false;
        }
    };

    for entry in entries.filter_map(|e| e.ok()) {
        let entry_path = entry.path();

        if entry_path.is_dir() {
            // Only check directories that aren't hidden (like .git)
            if !entry_path
                .file_name()
                .and_then(|n| n.to_str())
                .map_or(false, |n| n.starts_with('.'))
            {
                if !check_infojson_exists(&entry_path) {
                    all_valid = false;
                }

                // Recursively check subdirectories
                if !check_subdirs(&entry_path) {
                    all_valid = false;
                }
            }
        }
    }

    all_valid
}

/// Main validation function for the Sekai directory structure
pub fn validate_sekai(sekai_path: &Path) -> bool {
    if !check_directory_exists(sekai_path) {
        return false;
    }

    let all_valid = check_subdirs(sekai_path);

    if all_valid {
        log::log_info("SEKAI", "Directory structure is valid");
    } else {
        log::log_error(
            "SEKAI",
            "Directory structure is invalid - missing or invalid info.json files",
        );
    }
    all_valid
}
