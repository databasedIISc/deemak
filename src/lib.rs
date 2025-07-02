#![allow(unused_variables, unused_mut, dead_code, unused_imports)]
pub mod commands;
pub mod menu;
pub mod utils;

use std::sync::OnceLock;
pub static DEBUG_MODE: OnceLock<bool> = OnceLock::new();
