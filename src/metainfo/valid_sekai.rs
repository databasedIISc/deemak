use super::read_validate_info;
use crate::utils::log;
use std::path::Path;

/// Creates properly formatted .dir_info with valid JSON info.json
pub fn create_dir_info(dir: &Path, home_dir: bool) -> bool {
    // Skip if this is a .dir_info directory
    if dir.file_name().and_then(|n| n.to_str()) == Some(".dir_info") {
        return true;
    }

    let dir_info = dir.join(".dir_info");
    let info_path = dir_info.join("info.json");

    // Try to read existing info if present
    let existing_info = if info_path.exists() {
        read_validate_info(&info_path).ok()
    } else {
        None
    };

    // Create directory if needed
    if let Err(e) = std::fs::create_dir_all(&dir_info) {
        log::log_error(
            "SEKAI",
            &format!("Failed to create .dir_info in {}: {}", dir.display(), e),
        );
        return false;
    }

    // Get default values, preserving existing valid fields
    let mut default_info = super::info_reader::Info::default_for_path(dir, home_dir);

    if let Some(existing) = existing_info {
        // Preserve existing valid fields
        if !existing.location.trim().is_empty() {
            default_info.location = existing.location;
        }
        if !existing.about.trim().is_empty() {
            default_info.about = existing.about;
        }

        // Merge objects maps, preserving existing entries
        for (key, val) in existing.objects {
            default_info.objects.entry(key).or_insert(val);
        }
    } else {
        println!("No existing info found, using defaults");
    }
    // Write the merged info
    match std::fs::write(
        &info_path,
        match serde_json::to_string_pretty(&default_info) {
            Ok(json) => json,
            Err(e) => {
                log::log_error(
                    "SEKAI",
                    &format!("Failed to serialize info for {}: {}", dir.display(), e),
                );
                return false;
            }
        },
    ) {
        Ok(_) => {
            log::log_info(
                "SEKAI",
                &format!("Updated info.json for: {}", dir.display()),
            );
            true
        }
        Err(e) => {
            log::log_error(
                "SEKAI",
                &format!("Failed to write info.json in {}: {}", dir.display(), e),
            );
            false
        }
    }
}

/// Checks if .dir_info/info.json exists and is valid (updated for PathBuf)
pub fn check_dir_info_exists_valid(dir: &Path) -> bool {
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
fn validate_info_file(info_path: &Path) -> bool {
    match read_validate_info(info_path) {
        Ok(info) => info.validate().is_ok(),
        Err(e) => {
            log::log_warning(
                "SEKAI",
                &format!("Invalid info.json at {}: {}", info_path.display(), e),
            );
            false
        }
    }
}

/// Recursively checks directory structure (updated for PathBuf)
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

            // Skip .dir_info directories
            if dir_name == ".dir_info" {
                continue;
            }

            let entry_path_buf = entry_path;
            if !check_dir_info_exists_valid(&entry_path_buf) {
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
pub fn validate_or_create_sekai(sekai_path: &Path, home_check: bool) -> bool {
    // Initial path checks
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

    // Process directories recursively with single-pass validation/creation
    if home_check {
        // Check if the home directory is valid and create if not
        if !check_dir_info_exists_valid(sekai_path) {
            log::log_info(
                "SEKAI",
                &format!(
                    "Creating valid .dir_info for home directory: {}",
                    sekai_path.display()
                ),
            );
            if !create_dir_info(sekai_path, true) {
                log::log_error(
                    "SEKAI",
                    &format!(
                        "Failed to create valid .dir_info for home directory: {}",
                        sekai_path.display()
                    ),
                );
                return false;
            }
        }
        return true;
    }
    let all_valid = process_directory_recursive(sekai_path, true);

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

/// Recursively processes directories to validate or create valid .dir_info
fn process_directory_recursive(dir: &Path, is_home: bool) -> bool {
    let mut all_valid = true;

    // Skip .dir_info directories
    if dir.file_name().and_then(|n| n.to_str()) == Some(".dir_info") {
        return true;
    }

    // Check/create info for current directory
    let info_path = dir.join(".dir_info/info.json");
    if !info_path.exists() {
        log::log_info(
            "SEKAI",
            &format!("Creating valid .dir_info for: {}", dir.display()),
        );
        if !create_dir_info(dir, is_home) {
            all_valid = false;
        }
    // Else if not valid, try to create it
    } else if !create_dir_info(dir, is_home) {
        log::log_error(
            "SEKAI",
            &format!("Failed to create valid .dir_info for: {}", dir.display()),
        );
        all_valid = false;
    }
    // Process subdirectories if current directory is valid
    if all_valid {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_dir() {
                    all_valid &= process_directory_recursive(&path, false);
                }
            }
        }
    }

    all_valid
}
