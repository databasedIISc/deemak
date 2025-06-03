use super::{log, read_validate_info};
use std::path::Path;

/// Validates an info.json file at the given path
fn validate_info_file(info_path: &Path) -> bool {
    match read_validate_info(info_path) {
        Ok(info) => {
            if let Err(e) = info.validate() {
                log::log_error(
                    "SEKAI",
                    &format!("Invalid info.json at {}: {}", info_path.display(), e),
                );
                false
            } else {
                true
            }
        }
        Err(e) => {
            log::log_error(
                "SEKAI",
                &format!("Failed to read info.json at {}: {}", info_path.display(), e),
            );
            false
        }
    }
}

/// Checks if .dir_info/info.json exists and is valid
fn check_dir_info_exists(dir: &Path) -> bool {
    let info_path = dir.join(".dir_info/info.json");

    if !info_path.exists() {
        log::log_warning(
            "SEKAI",
            &format!("info.json not found in: {}/.dir_info", dir.display()),
        );
        return false;
    }

    validate_info_file(&info_path)
}

/// Recursively checks directory structure
fn check_subdirectories(path: &Path) -> bool {
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
            let dir_name = entry_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");

            // Skip recursing into .dir_info directory
            if dir_name == ".dir_info" {
                continue;
            }

            // All directories (including dot-directories) must have .dir_info/info.json
            if !check_dir_info_exists(&entry_path) {
                all_valid = false;
            }

            // Recursively check other subdirectories
            if !check_subdirectories(&entry_path) {
                all_valid = false;
            }
        }
    }

    all_valid
}

/// Main validation function
pub fn validate_sekai(sekai_path: &Path) -> bool {
    if !sekai_path.exists() {
        log::log_error(
            "SEKAI",
            &format!("Directory does not exist: {}", sekai_path.display()),
        );
        return false;
    }

    if !sekai_path.is_dir() {
        log::log_error(
            "SEKAI",
            &format!("Path is not a directory: {}", sekai_path.display()),
        );
        return false;
    }

    log::log_info(
        "SEKAI",
        &format!("Validating directory: {}", sekai_path.display()),
    );

    // First check the root directory's .dir_info
    if !check_dir_info_exists(sekai_path) {
        return false;
    }

    let all_valid = check_subdirectories(sekai_path);

    if all_valid {
        log::log_info("SEKAI", "Directory structure is valid");
    } else {
        log::log_error(
            "SEKAI",
            "Directory structure is invalid - missing or invalid .dir_info/info.json files",
        );
    }

    all_valid
}
