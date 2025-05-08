use serde::Deserialize;
use std::fs;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Deserialize)]
pub struct Info {
    pub location: String,
    pub about: String,
}

#[derive(Debug, Error)]
pub enum InfoError {
    #[error("Info file not found")]
    NotFound,
    #[error("Failed to read info file: {0}")]
    ReadError(std::io::Error),
    #[error("Failed to parse info file: {0}")]
    ParseError(serde_json::Error),
}

pub fn read_info(info_path: &Path) -> Result<Info, InfoError> {
    if !info_path.exists() {
        return Err(InfoError::NotFound);
    }

    let contents = fs::read_to_string(info_path).map_err(InfoError::ReadError)?;

    let mut info: Info = serde_json::from_str(&contents).map_err(InfoError::ParseError)?;

    // Clean the about field
    info.about = info.about.trim_matches('"').to_string();

    Ok(info)
}
