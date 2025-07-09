use super::info_reader::read_validate_info;
use std::path::Path;

/// Reads the lock permissions from an object.
/// Returns: (bool, bool) => (is_locked, is_level_locked)
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
        read_validate_info(&info_path).map_err(|e| format!("Failed to read info.json: {}", e))?;

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

    // bit count starts from the right
    let first_bit = match Iterator::nth(&mut lock_str.chars(), 0) {
        Some('1') => true,
        Some('0') => false,
        _ => return Err("Invalid lock string format".to_string()),
    };

    let second_bit = match lock_str.chars().nth(1) {
        Some('1') => true,
        Some('0') => false,
        _ => return Err("Invalid lock string format".to_string()),
    };

    Ok((first_bit, second_bit))
}
