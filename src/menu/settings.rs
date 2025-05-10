use raylib::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Settings {
    pub theme: Theme,
    pub font_size: u32,
    pub show_line_numbers: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Theme {
    Dark,
    Light,
    Custom(Color, Color, Color),
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: Theme::Dark,
            font_size: 16,
            show_line_numbers: true,
        }
    }
}

impl Settings {
    pub fn load() -> Self {
        let path = Path::new("settings.json");
        if path.exists() {
            match fs::read_to_string(path) {
                Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
                Err(_) => Settings::default(),
            }
        } else {
            Settings::default()
        }
    }

    pub fn save(&self) {
        let path = Path::new("settings.json");
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write(path, json);
        }
    }

    pub fn apply_theme(&self, d: &mut RaylibDrawHandle) {
        match self.theme {
            Theme::Dark => {
                d.clear_background(Color::BLACK);
            }
            Theme::Light => {
                d.clear_background(Color::WHITE);
            }
            Theme::Custom(bg, _, _) => {
                d.clear_background(bg);
            }
        }
    }
}
