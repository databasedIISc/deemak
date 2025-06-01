pub mod commands;
pub mod menu;
pub mod utils;

use std::sync::OnceLock;
pub static DEBUG_MODE: OnceLock<bool> = OnceLock::new();
