use crate::DEV_MODE;
use crate::utils::relative_deemak_path;
use std::path::Path;

pub fn debug_mode() -> bool {
    *DEV_MODE.get().unwrap_or(&false)
}

/// Replaces the Paths in the message with their relative paths.
pub fn filter_msg(message: &str, sekai_dir: Option<&Path>) -> String {
    message
        .split_whitespace()
        .map(|word| {
            let punc = |c: char| c == '\'' || c == '"' || c == ',' || c == '.' || c == ';';
            let trimmed_word = word.trim_matches(punc);
            let path = Path::new(trimmed_word);
            let replaced_path = relative_deemak_path(path, sekai_dir);
            if replaced_path.to_string_lossy() != trimmed_word {
                word.replace(trimmed_word, &replaced_path.to_string_lossy())
            } else {
                word.to_string()
            }
        })
        .collect::<Vec<String>>()
        .join(" ")
}

/// Logger for debugging elements.
/// Args:
///     `feature` - the feature/command/module name
///     `message` - the debug message.
/// Example:
///     log_debug("go", "Parsing arguments: ... ");
///     log_debug("info_reader", "Reading info from file: ...");
pub fn log_debug(feature: &str, message: &str) {
    if debug_mode() {
        println!(
            "\x1b[34m[DEBUG] \x1b[0m {} :: {}",
            feature,
            filter_msg(message, None)
        );
    }
}

/// Logger for general info
/// Args:
///    `feature` - the feature/command/module name
///    `message` - the info message.
/// Example:
///     log_info("go", "You have entered the directory: ...");
///     log_info("info_reader", "Successfully read info from file: ...");
pub fn log_info(feature: &str, message: &str) {
    if debug_mode() {
        println!(
            "\x1b[32m[INFO]\x1b[0m {} :: {}",
            feature,
            filter_msg(message, None)
        );
    }
}

/// Logger for warnings
/// Args:
///     `feature` - the feature/command/module name
///     `message` - the warning message.
/// Example:
///     log_warning("go", "Attempted to go to a file instead of a directory: ...");
///     log_warning("info_reader", "The info.json contains incorrect fields: ...");
pub fn log_warning(feature: &str, message: &str) {
    if debug_mode() {
        eprintln!(
            "\x1b[33m[WARNING] \x1b[0m {} :: {}",
            feature,
            filter_msg(message, None)
        );
    }
}

/// Logger for errors
/// Args:
///     `feature` - the feature/command/module name
///     `message` - the error message.
/// Example:
///     log_error("go", "Failed to change directory: ...");
///     log_error("info_reader", "Failed to parse: ...");
pub fn log_error(feature: &str, message: &str) {
    if debug_mode() {
        eprintln!(
            "\x1b[31m[ERROR] \x1b[0m {} :: {}",
            feature,
            filter_msg(message, None)
        );
    }
}

/// Common Result Logger for operations that return Result<(), E> where E: Display
/// Uses `Info` for success and `Warning` for failure.
/// For more complex logging needs, handle logging manually.
pub fn log_result<E: std::fmt::Display>(feature: &str, result: Result<(), E>, message: &str) {
    match result {
        Ok(_) => log_info(feature, &format!("Success: {}", filter_msg(message, None))),
        Err(e) => log_warning(
            feature,
            &format!("Failed: {}: {}", filter_msg(message, None), e),
        ),
    }
}
