use super::info_reader::read_validate_info;
use crate::utils::log;
use crate::utils::relative_deemak_path;
use std::path::Path;

/// Reads the lock permissions from an object.
/// Returns: (bool, bool) => (is_level_locked, is_locked)
///
/// The lock corresponsds as below:
///     1st bit: Locked/Unlocked bit.
///     2nd bit: Type of lock. 1 => Level locking, 0 => Normal locking.
/// The bit correspondence: "1" => True, "0" => False
pub fn read_lock_perm(obj_path: &Path) -> Result<(bool, bool), String> {
    let info_path = obj_path
        .parent()
        .ok_or("Object has no parent directory")?
        .join(".dir_info/info.json");

    let info =
        read_validate_info(&info_path).map_err(|e| format!("Failed to read info.json: {e}"))?;

    let obj_name = obj_path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or("Invalid object name")?;

    let lock_str = info
        .objects
        .get(obj_name)
        .and_then(|obj| obj.properties.get("locked"))
        .and_then(|v| v.as_str())
        .ok_or("Lock status not found or invalid")?;

    if lock_str.len() != 2 {
        return Err("Lock string should be exactly 2 characters".to_string());
    }

    // Collect bits from right to left (LSB first)
    let mut bits = Vec::new();
    for c in lock_str.chars() {
        // Reverse iteration for right-to-left
        match c {
            '1' => bits.push(true),
            '0' => bits.push(false),
            _ => return Err("Invalid lock string format".to_string()),
        }
    }

    // Return tuple with first_bit of bits at right end
    Ok((bits[0], bits[1]))
}

/// Checks if the operation can be performed if object is unlocked.
/// Returns: Ok if operation can proceed, Err with message if locked. OR Err if lock status cannot
/// be determined.
pub fn operation_locked_perm(
    obj_path: &Path,
    operation: &str,
    message: &str,
) -> Result<(), String> {
    // Check all parents up to root
    let mut current = obj_path;
    while let Some(parent) = current.parent() {
        if let Ok((_, locked)) = read_lock_perm(current) {
            if locked {
                let rel_path = relative_deemak_path(current);
                log::log_warning(
                    operation,
                    &format!("Locked path: {} - {}", rel_path.display(), message),
                );
                return Err(format!(
                    "{}: {} is locked. {}",
                    operation,
                    rel_path.display(),
                    message
                ));
            }
        }
        current = parent;
    }

    // Check the object itself
    match read_lock_perm(obj_path) {
        Ok((_, true)) => {
            let rel_path = relative_deemak_path(obj_path);
            log::log_warning(
                operation,
                &format!("Locked: {} - {}", rel_path.display(), message),
            );
            Err(format!(
                "{}: {} is locked. {}",
                operation,
                rel_path.display(),
                message
            ))
        }
        Ok((_, false)) => Ok(()),
        Err(e) => {
            log::log_warning(
                operation,
                &format!(
                    "Lock check failed for {}: {}",
                    relative_deemak_path(obj_path).display(),
                    e
                ),
            );
            Ok(()) // Fail open if we can't check
        }
    }
}
