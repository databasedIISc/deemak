#[cfg(debug_assertions)]
pub fn log_debug(message: &str, debug_mode: bool) {
    if debug_mode {
        println!("[DEBUG] {}", message);
    }
}

pub fn log_info(message: &str, debug_mode: bool) {
    if debug_mode {
        println!("[INFO] {}", message);
    }
}

pub fn log_warning(message: &str, debug_mode: bool) {
    if debug_mode {
        println!("[WARNING] {}", message);
    }
}

pub fn log_error(message: &str, debug_mode: bool) {
    if debug_mode {
        eprintln!("[ERROR] {}", message);
    }
}
