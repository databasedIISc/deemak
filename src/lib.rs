#![allow(unused_variables, unused_mut, dead_code)]
pub mod commands;
pub mod gui_main;
pub mod gui_shell;
pub mod keys;
pub mod login;
pub mod menu;
pub mod metainfo;
pub mod rns;
pub mod server;
pub mod utils;

use std::sync::OnceLock;

pub static DEV_MODE: OnceLock<bool> = OnceLock::new();
pub static SEKAI_DIR: OnceLock<String> = OnceLock::new();

// DEEMAK Macros!
#[macro_export]
macro_rules! fatal_error {
    ($module:expr, $($arg:tt)+) => {{
        let message = format!($($arg)+);
        $crate::utils::log::log_error($module, &message);
        eprintln!("Error: {}", message);
        $crate::utils::cleanup::exit_deemak(1);
    }};
}

#[macro_export]
macro_rules! epr_log_error {
    ($module:expr, $($arg:tt)+) => {{
        let message = format!($($arg)+);
        $crate::utils::log::log_error($module, &message);
        eprintln!("Error: {}", message);
    }};
}

#[macro_export]
macro_rules! pr_info {
    ($module:expr, $($arg:tt)+) => {{
        let message = format!($($arg)+);
        $crate::utils::log::log_info($module, &message);
        println!("Error: {}", message);
    }};
}
