use crate::utils::log;
use std::fs::{self, read_dir};
use std::io::Error;
use std::path::Path;

pub const LOCATIONS: [&str; 2] = ["/tmp", "/var/tmp"];
pub const DEEMAK_PREFIX: [&str; 2] = ["deemak_", "deemak-"];

/// Helper function to log warnings consistently.
fn log_cleanup_warning(context: &str, path_display: &str, error: &Error) {
    log::log_warning(
        "Cleanup",
        &format!("Failed to {context} {path_display}: {error}"),
    );
}

/// Helper function to log cleanup success consistently.
fn log_cleanup_success(context: &str, path_display: &str) {
    log::log_info("Cleanup", &format!("Successfully {context} {path_display}"));
}

/// Deletes a file or directory at the specified path, logging success or failure.
fn del_obj(path: &Path) -> Result<(), String> {
    let result = if path.is_dir() {
        fs::remove_dir_all(path)
            .map(|_| log_cleanup_success("removed directory", &path.display().to_string()))
    } else {
        fs::remove_file(path)
            .map(|_| log_cleanup_success("removed file", &path.display().to_string()))
    };

    result.map_err(|e| {
        log_cleanup_warning("remove", &path.display().to_string(), &e);
        format!("Failed to remove {}: {}", path.display(), e)
    })
}

/// Clean up all the temporary files created during deemak.
fn cleanup_at_location(dir: &str) -> Result<(), String> {
    let path = Path::new(dir);

    for entry in read_dir(path).map_err(|e| {
        let msg = format!("Failed to read directory {dir}: {e}");
        log_cleanup_warning("read directory", dir, &e);
        msg
    })? {
        let entry = entry.map_err(|e| {
            let msg = format!("Failed to read directory entry in {dir}: {e}");
            log_cleanup_warning("read directory entry", dir, &e);
            msg
        })?;

        let file_name = entry.file_name();
        let file_name_str = file_name.to_string_lossy();

        if DEEMAK_PREFIX
            .iter()
            .any(|prefix| file_name_str.starts_with(prefix))
        {
            let path_to_remove = entry.path();
            if let Err(e) = del_obj(&path_to_remove) {
                let _ = e;
            }
        }
    }
    Ok(())
}

/// Clean up only `.tmp.zlib` in the current working directory, just in case present
fn clean_cwd() -> Result<(), String> {
    let cwd = std::env::current_dir().map_err(|e| {
        log_cleanup_warning("get current directory", "current", &e);
        format!("Failed to get current working directory: {e}")
    })?;

    for entry in read_dir(&cwd).map_err(|e| {
        log_cleanup_warning("read current directory", "current", &e);
        format!("Failed to read current directory: {e}")
    })? {
        let entry = entry.map_err(|e| {
            log_cleanup_warning("read directory entry", "current", &e);
            format!("Failed to read directory entry in current directory: {e}")
        })?;

        let path_to_remove = entry.path();
        let file_name_str = path_to_remove
            .file_name()
            .map(|s| s.to_string_lossy())
            .unwrap_or_default();

        if file_name_str.ends_with(".tmp.zlib") && path_to_remove.is_file() {
            if let Err(e) = fs::remove_file(&path_to_remove) {
                log_cleanup_warning("remove file", &file_name_str, &e);
            }
        }
    }
    Ok(())
}

/// Clean up all temporary files in all locations.
pub fn cleanup_deemak() -> Result<(), String> {
    // clean up the current working directory first
    if let Err(e) = clean_cwd() {
        log_cleanup_warning(
            "cleanup current working directory",
            "current",
            &Error::other(e),
        );
    }
    // Clean up all specified locations
    for location in LOCATIONS.iter() {
        if let Err(e) = cleanup_at_location(location) {
            log_cleanup_warning("cleanup at location", location, &Error::other(e));
        }
    }
    Ok(())
}

/// Cleanup all temporary files and exit the DEEMAK shell with the specified exit code.
pub fn exit_deemak(code: i32) -> ! {
    log::log_info("Application", "Exiting DEEMAK Shell");
    let clean_result = cleanup_deemak();
    if let Err(e) = clean_result {
        log::log_error("Cleanup", &e);
        eprintln!("Error during cleanup: {e}");
    } else {
        log::log_info("Cleanup", "Temporary files cleaned up successfully.");
    }

    std::process::exit(code);
}
