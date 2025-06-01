use serde::Deserialize;
use std::fs;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)] // Rejects unexpected fields
pub struct Info {
    #[serde(rename = "location")]
    pub location: String,
    #[serde(rename = "about")]
    pub about: String,
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
    /// Validates the struct contents
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

        Ok(())
    }
}

pub fn read_validate_info(info_path: &Path) -> Result<Info, InfoError> {
    // File existence check
    if !info_path.exists() {
        return Err(InfoError::NotFound(info_path.display().to_string()));
    }

    // Read and parse
    let contents = fs::read_to_string(info_path)?;
    let mut info: Info = serde_json::from_str(&contents)?;

    // Clean data
    info.about = info.about.trim_matches('"').trim().to_string();
    info.location = info.location.trim().to_string();

    // Validate
    info.validate()?;

    Ok(info)
}

pub fn validate_json_schema(json_str: &str) -> Result<(), InfoError> {
    let value: serde_json::Value = serde_json::from_str(json_str)?;

    match (value.get("location"), value.get("about")) {
        (Some(loc), Some(about)) => {
            if !loc.is_string() {
                return Err(InfoError::ValidationError(
                    "location must be a string".to_string(),
                ));
            }
            if !about.is_string() {
                return Err(InfoError::ValidationError(
                    "about must be a string".to_string(),
                ));
            }
        }
        _ => {
            return Err(InfoError::ValidationError(
                "Missing required fields: 'location' and 'about'".to_string(),
            ));
        }
    }

    Ok(())
}
