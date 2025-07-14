use crate::commands::cmds::normalize_path;
use crate::utils::relative_deemak_path;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ObjectInfo {
    #[serde(flatten)]
    pub properties: HashMap<String, Value>,
}

pub static DEFAULT_PERMISSIONS: &str = "00";

impl ObjectInfo {
    pub fn new() -> Self {
        Self {
            properties: HashMap::new(),
        }
    }

    pub fn with_locked(locked: String) -> Self {
        let mut obj = Self::new();
        obj.properties
            .insert("locked".to_string(), Value::String(locked));
        obj
    }

    pub fn with_decrypt_me(decrypt_me: String) -> Self {
        let mut obj = Self::new();
        obj.properties
            .insert("decrypt_me".to_string(), Value::String(decrypt_me));
        obj
    }

    pub fn with_obj_salt(obj_salt: String) -> Self {
        let mut obj = Self::new();
        obj.properties
            .insert("obj_salt".to_string(), Value::String(obj_salt));
        obj
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Info {
    #[serde(rename = "location")]
    pub location: String,
    #[serde(rename = "about")]
    pub about: String,
    #[serde(rename = "objects")]
    pub objects: HashMap<String, ObjectInfo>,
}

#[derive(Debug, Error)]
pub enum InfoError {
    #[error("Info file not found at {0}")]
    NotFound(String),
    #[error("Failed to read info file: {0}")]
    ReadError(#[from] std::io::Error),
    #[error("Failed to parse info file: {0}")]
    ParseError(#[from] serde_json::Error),
    #[error("Validation error: {0}")]
    ValidationError(String),
}

impl Info {
    /// Creates default Info values for a path
    pub fn default_for_path(path: &Path, home_dir: bool) -> Self {
        let norm_path = normalize_path(path);
        // NOTE: Since deafult Permission is "00", decrypt_me and obj_salt are not going to be set.
        Info {
            location: Self::default_location(&norm_path, home_dir),
            about: Self::default_about(&norm_path, home_dir),
            objects: Self::default_objects(&norm_path),
        }
    }

    pub fn default_about(path: &Path, home_dir: bool) -> String {
        format!(
            "You are in '{}'. Look around and explore!",
            Self::default_location(path, home_dir)
        )
    }
    pub fn default_location(path: &Path, home_dir: bool) -> String {
        if home_dir {
            "HOME".to_string()
        } else {
            relative_deemak_path(path).display().to_string()
        }
    }

    pub fn default_objects(path: &Path) -> HashMap<String, ObjectInfo> {
        // Create default empty objects map
        let mut objects = HashMap::new();

        // If the path is a directory, read its files/dir and add to objects
        if path.is_dir() {
            if let Ok(entries) = std::fs::read_dir(path) {
                for entry in entries.filter_map(|e| e.ok()) {
                    if let Some(name) = entry.file_name().to_str() {
                        if name != ".dir_info" {
                            objects.insert(
                                name.to_string(),
                                ObjectInfo::with_locked(DEFAULT_PERMISSIONS.to_string()),
                            );
                        }
                    }
                }
            }
        }
        objects
    }

    pub fn validate(&self) -> Result<(), InfoError> {
        if self.location.trim().is_empty() {
            return Err(InfoError::ValidationError(
                "Location cannot be empty".to_string(),
            ));
        }
        if self.about.trim().is_empty() {
            return Err(InfoError::ValidationError(
                "About cannot be empty".to_string(),
            ));
        }
        // NOTE: For now, we allow `Objects` to be possibly empty, since we can have an empty directory.
        Ok(())
    }
}

/// Reads, validates and returns Info
pub fn read_validate_info(info_path: &Path) -> Result<Info, InfoError> {
    if !info_path.exists() {
        return Err(InfoError::NotFound(info_path.display().to_string()));
    }

    let contents = fs::read_to_string(info_path)?;
    let mut info: Info = serde_json::from_str(&contents)?;

    // Trim string fields
    info.about = info.about.trim().to_string();
    info.location = info.location.trim().to_string();

    // Validate and normalize object properties
    for obj_info in info.objects.values_mut() {
        // Check if "locked" is a string of 2 bits
        if let Some(Value::String(s)) = obj_info.properties.get("locked") {
            if s.len() == 2 && s.chars().all(|c| c == '0' || c == '1') {
                // Valid locked value, do nothing
            } else {
                return Err(InfoError::ValidationError(
                    "Invalid 'locked' value, must be a 2-bit string".to_string(),
                ));
            }

            // If is_level is '1', further checks are needed
            if let Some(is_locked) = s.chars().nth(1).map(|c| c == '1') {
                if !is_locked {
                    continue; // Not locked, skip further checks
                }
                // ensure it has a 'decrypt_me' property
                if !obj_info.properties.contains_key("decrypt_me") {
                    return Err(InfoError::ValidationError(
                        "Locked objects must have a 'decrypt_me' property".to_string(),
                    ));
                }
                // obj_salt is required for locked objects
                if !obj_info.properties.contains_key("obj_salt") {
                    return Err(InfoError::ValidationError(
                        "Locked objects must have an 'obj_salt' property".to_string(),
                    ));
                }
                // enure it has a "compare me property
                if !obj_info.properties.contains_key("compare_me") {
                    return Err(InfoError::ValidationError(
                        "Locked objects must have a 'compare_me' property".to_string(),
                    ));
                }
            } else {
                // If not locked, ensure 'decrypt_me' is not present
                obj_info.properties.remove("decrypt_me");
            }
        }

        // Trim any string values in properties
        for value in obj_info.properties.values_mut() {
            if let Value::String(s) = value {
                *value = Value::String(s.trim().to_string());
            }
        }
    }

    info.validate()?;
    Ok(info)
}

/// Add an object to info.json with optional initial properties
/// Example:
///     add_obj_to_info(&info_path, "file.txt", None);
pub fn add_obj_to_info(
    obj_path: &Path,
    obj_name: &str,
    initial_props: Option<HashMap<String, Value>>,
) -> Result<(), InfoError> {
    let info_path = &obj_path
        .parent()
        .unwrap()
        .join(".dir_info")
        .join("info.json");
    let mut info = read_validate_info(info_path)?;

    if !info.objects.contains_key(obj_name) {
        let obj_info = if let Some(props) = initial_props {
            ObjectInfo { properties: props }
        } else {
            ObjectInfo::with_locked(DEFAULT_PERMISSIONS.to_string()) // Default to locked=false if no props provided
        };

        info.objects.insert(obj_name.to_string(), obj_info);

        let json = serde_json::to_string_pretty(&info)?;
        std::fs::write(info_path, json)?;
    }

    Ok(())
}

/// Delete an object from info.json
pub fn del_obj_from_info(obj_path: &Path, obj_name: &str) -> Result<(), InfoError> {
    let info_path = &obj_path
        .parent()
        .unwrap()
        .join(".dir_info")
        .join("info.json");
    let mut info = read_validate_info(info_path)?;

    if info.objects.remove(obj_name).is_some() {
        let json = serde_json::to_string_pretty(&info)?;
        std::fs::write(info_path, json)?;
    }
    Ok(())
}

/// Update or add a status property for an object
///
/// # Arguments
/// * `obj_path` - Path to the object whose status is to be updated
/// * `obj_name` - Name of the object to update
/// * `status` - Status key to update (e.g., "locked", "hidden")
/// * `st_value` - Value to set (must be serializable to JSON)
pub fn update_obj_status(
    obj_path: &Path,
    obj_name: &str,
    status: &str,
    st_value: Value,
) -> Result<(), InfoError> {
    let info_path = &obj_path
        .parent()
        .unwrap()
        .join(".dir_info")
        .join("info.json");
    let mut info = read_validate_info(info_path)?;

    // Get or create the object entry
    let obj_info = info
        .objects
        .entry(obj_name.to_string())
        .or_insert_with(ObjectInfo::default);

    // Update or add the property
    obj_info.properties.insert(status.to_string(), st_value);

    // Write back the updated info
    let json = serde_json::to_string_pretty(&info)?;
    std::fs::write(info_path, json)?;

    Ok(())
}

/// Gets object info from a directory's info.json, returning the existing info or a default
/// Returns Error if the info.json is invalid or can't be read
pub fn read_get_obj_info(info_path: &Path, obj_name: &str) -> Result<ObjectInfo, InfoError> {
    let info = read_validate_info(info_path)?;
    Ok(info
        .objects
        .get(obj_name)
        .cloned() // This does the same as .map(|x| x.clone())
        .unwrap_or_default())
}

pub fn get_encrypted_flag(path: &Path, level_name: &str) -> Result<String, String> {
    //the flag is stored in ./dir_info/info.json of parent directory
    match read_get_obj_info(
        &path.parent().unwrap().join(".dir_info/info.json"),
        level_name,
    ) {
        Ok(obj_info) => {
            if let Some(decrypt_me) = obj_info.properties.get("decrypt_me") {
                if let serde_json::Value::String(flag_str) = decrypt_me {
                    Ok(flag_str.clone())
                } else {
                    Err("decrypt_me property is not a string.".to_string())
                }
            } else {
                Err("decrypt_me property not found in object info.".to_string())
            }
        }
        Err(e) => Err("Error reading object info".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_update_obj_status() {
        // Create a temporary directory for the test
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".dir_info")).unwrap();
        let info_path = dir.path().join(".dir_info/info.json");

        // Create a dummy info.json file
        let mut file = File::create(&info_path).unwrap();
        file.write_all(b"{\"location\":\"test\",\"about\":\"test\",\"objects\":{}}")
            .unwrap();

        // Create a dummy object file
        let obj_path = dir.path().join("file.txt");
        File::create(&obj_path).unwrap();

        // Update the object's status
        update_obj_status(
            &obj_path,
            "file.txt",
            "locked",
            serde_json::Value::Bool(true),
        )
        .unwrap();

        // Verify the update
        let data = fs::read_to_string(&info_path).unwrap();
        let info: Info = serde_json::from_str(&data).unwrap();
        assert_eq!(
            info.objects
                .get("file.txt")
                .unwrap()
                .properties
                .get("locked")
                .unwrap(),
            &serde_json::Value::Bool(true)
        );
    }
}
