use super::info_reader::read_validate_info;
use crate::utils::log;
use std::path::Path;

/// Creates properly formatted .dir_info with valid JSON info.json
/// If the directory already has a .dir_info, it will merge with existing info
/// else use default values.
///
/// If the files present in info.json do not exist anymore, they will be removed
pub fn create_dir_info(dir: &Path, home_dir: bool) -> bool {
    // Skip if this is a .dir_info directory
    if dir.file_name().and_then(|n| n.to_str()) == Some(".dir_info") {
        return true;
    }

    let dir_info = dir.join(".dir_info");
    let info_path = dir_info.join("info.json");

    // Try to read existing info if present
    let mut existing_info = if info_path.exists() {
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

    // Get default values
    let default_info = super::info_reader::Info::default_for_path(dir, home_dir);

    // Get current directory entries (excluding .dir_info)
    let current_entries: std::collections::HashSet<String> = match std::fs::read_dir(dir) {
        Ok(entries) => entries
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name() != ".dir_info")
            .filter_map(|e| e.file_name().into_string().ok())
            .collect(),
        Err(_) => std::collections::HashSet::new(),
    };

    // Merge defaults into existing info (or create new from defaults)
    let info_to_write = match existing_info {
        Some(ref mut existing) => {
            // Fill in missing default fields
            if (*existing).location.trim().is_empty() {
                existing.location = default_info.location.clone();
            }
            if (*existing).about.trim().is_empty() {
                existing.about = default_info.about.clone();
            }

            // Remove objects that no longer exist in the directory
            existing.objects.retain(|k, _| current_entries.contains(k));

            // Add new objects from defaults that exist in directory but not in info
            for (key, default_val) in default_info.objects {
                if current_entries.contains(&key) && !existing.objects.contains_key(&key) {
                    existing.objects.insert(key, default_val);
                }
            }

            // Return the existing info for writing
            existing
        }
        None => {
            // No existing info - create new with only existing objects
            let mut new_info = super::info_reader::Info::default_for_path(dir, home_dir);
            new_info.objects.retain(|k, _| current_entries.contains(k));
            // Store it in existing_info so it lives long enough
            existing_info = Some(new_info);
            existing_info.as_mut().unwrap()
        }
    };

    // Write the merged info
    match std::fs::write(
        &info_path,
        match serde_json::to_string_pretty(&info_to_write) {
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
        Ok(_) => true,
        Err(e) => {
            log::log_error(
                "SEKAI",
                &format!("Failed to write info.json in {}: {}", dir.display(), e),
            );
            false
        }
    }
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

    // Check for just home directory validation
    if home_check {
        // Check if the home directory is valid and create if not
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
        return true;
    }

    // Process directories recursively with single-pass validation/creation
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
