#![allow(unused_variables, unused_mut, dead_code)]
pub mod commands;
pub mod gui_shell;
pub mod keys;
pub mod login;
pub mod menu;
pub mod metainfo;
pub mod rns;
pub mod server;
pub mod utils;

use std::sync::OnceLock;

pub static DEBUG_MODE: OnceLock<bool> = OnceLock::new();
pub static SEKAI_DIR: OnceLock<String> = OnceLock::new();
