use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
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
    /// Creates default Info values for a path
    pub fn default_for_path(path: &PathBuf, home_dir: bool) -> Self {
        let mut dir_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        if home_dir {
            dir_name = "home";
        }

        Info {
            location: if path.ends_with("sekai") {
                "home".to_string()
            } else {
                dir_name.to_string()
            },
            about: format!("You are in '{}'. Look around and explore!", dir_name),
        }
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
        Ok(())
    }
}

pub fn read_validate_info(info_path: &PathBuf) -> Result<Info, InfoError> {
    if !info_path.exists() {
        return Err(InfoError::NotFound(info_path.display().to_string()));
    }

    let contents = fs::read_to_string(info_path)?;
    let mut info: Info = serde_json::from_str(&contents)?;

    info.about = info.about.trim_matches('"').trim().to_string();
    info.location = info.location.trim().to_string();

    info.validate()?;
    Ok(info)
}
