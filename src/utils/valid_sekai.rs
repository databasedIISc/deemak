use super::{log, read_validate_info};
use std::path::PathBuf;

/// Creates properly formatted .dir_info with valid JSON info.json
pub fn create_dir_info(dir: &PathBuf, home_dir: bool) -> bool {
    // Skip if this is a .dir_info directory
    if dir.file_name().and_then(|n| n.to_str()) == Some(".dir_info") {
        return true;
    }

    let dir_info = dir.join(".dir_info");
    let info_path = dir_info.join("info.json");

    // Skip if already exists
    if info_path.exists() {
        return true;
    }

    // Create directory if needed
    if let Err(e) = std::fs::create_dir_all(&dir_info) {
        log::log_error(
            "SEKAI",
            &format!("Failed to create .dir_info in {}: {}", dir.display(), e),
        );
        return false;
    }

    // Get default values from Info struct
    let default_info = super::info_reader::Info::default_for_path(dir, home_dir);

    // Write as proper JSON file
    match std::fs::write(
        &info_path,
        match serde_json::to_string_pretty(&default_info) {
            Ok(json) => json,
            Err(e) => {
                log::log_error(
                    "SEKAI",
                    &format!(
                        "Failed to serialize default info for {}: {}",
                        dir.display(),
                        e
                    ),
                );
                return false;
            }
        },
    ) {
        Ok(_) => {
            log::log_info(
                "SEKAI",
                &format!("Created valid info.json for: {}", dir.display()),
            );
            true
        }
        Err(e) => {
            log::log_error(
                "SEKAI",
                &format!("Failed to create info.json in {}: {}", dir.display(), e),
            );
            false
        }
    }
}

/// Checks if .dir_info/info.json exists and is valid (updated for PathBuf)
pub fn check_dir_info_exists(dir: &PathBuf) -> bool {
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

/// Validates an info.json file at the given path (updated for PathBuf)
fn validate_info_file(info_path: &PathBuf) -> bool {
    match read_validate_info(info_path) {
        Ok(info) => info.validate().is_ok(),
        Err(e) => {
            log::log_error(
                "SEKAI",
                &format!("Invalid info.json at {}: {}", info_path.display(), e),
            );
            false
        }
    }
}

/// Recursively checks directory structure (updated for PathBuf)
fn check_subdirectories(path: &PathBuf) -> bool {
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

            // Skip .dir_info directories
            if dir_name == ".dir_info" {
                continue;
            }

            let entry_path_buf = entry_path;
            if !check_dir_info_exists(&entry_path_buf) {
                all_valid = false;
            }
            if !check_subdirectories(&entry_path_buf) {
                all_valid = false;
            }
        }
    }
    all_valid
}

/// Main validation function with auto-creation
pub fn validate_or_create_sekai(sekai_path: &PathBuf) -> bool {
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

    // First validate
    let mut all_valid = check_dir_info_exists(sekai_path);
    all_valid &= check_subdirectories(sekai_path);

    // If invalid, attempt to fix
    if !all_valid {
        log::log_info("SEKAI", "Attempting to create missing .dir_info...");
        all_valid = true; // Reset and check again after creation

        // Create for root if missing
        if !sekai_path.join(".dir_info/info.json").exists() {
            all_valid &= create_dir_info(sekai_path, true);
        }

        // Create for subdirectories if missing
        if let Ok(entries) = std::fs::read_dir(sekai_path) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_dir()
                    && path.file_name().and_then(|n| n.to_str()) != Some(".dir_info")
                    && !path.join(".dir_info/info.json").exists()
                {
                    all_valid &= create_dir_info(&path, false);
                }
            }
        }

        // Re-validate after creation attempts
        if all_valid {
            all_valid = check_dir_info_exists(sekai_path) && check_subdirectories(sekai_path);
        }
    }

    if all_valid {
        log::log_info("SEKAI", "Directory structure is valid");
    } else {
        log::log_error(
            "SEKAI",
            "Directory structure could not be validated/created",
        );
    }

    all_valid
}
