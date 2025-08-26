use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Default)]
pub struct DeemakConfig {
    pub font_index: usize,
}

pub fn get_config_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or(".".to_string());
    let mut path = PathBuf::from(home);
    path.push(".config/deemak/config.json");
    path
}

pub fn load_config() -> DeemakConfig {
    let path = get_config_path();
    if let Ok(mut file) = File::open(&path) {
        let mut contents = String::new();
        if file.read_to_string(&mut contents).is_ok()
            && let Ok(cfg) = serde_json::from_str(&contents)
        {
            return cfg;
        }
    }
    DeemakConfig::default()
}

pub fn save_config(cfg: &DeemakConfig) {
    let path = get_config_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(mut file) = File::create(&path) {
        let _ = file.write_all(serde_json::to_string_pretty(cfg).unwrap().as_bytes());
    }
}
