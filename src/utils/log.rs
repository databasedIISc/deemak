use crate::DEBUG_MODE;

pub fn debug_mode() -> bool {
    *DEBUG_MODE.get().unwrap_or(&false)
}

#[cfg(debug_assertions)]
/// Logger for debugging elements.
/// Args:
///     `feature` - the feature/command/module name
///     `message` - the debug message.
/// Example:
///     log_debug("go", "Parsing arguments: ... ");
///     log_debug("info_reader", "Reading info from file: ...");
pub fn log_debug(feature: &str, message: &str) {
    if debug_mode() {
        println!("\x1b[34m[DEBUG] \x1b[0m {} :: {}", feature, message);
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
        println!("[INFO] {} :: {}", feature, message);
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
        eprintln!("\x1b[33m[WARNING] \x1b[0m {} :: {}", feature, message);
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
        eprintln!("\x1b[31m[ERROR] \x1b[0m {} :: {}", feature, message);
    }
}
