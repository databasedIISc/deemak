use crate::DEBUG_MODE;

pub fn debug_mode() -> bool {
    *DEBUG_MODE.get().unwrap_or(&false)
}

#[cfg(debug_assertions)]
pub fn log_debug(message: &str) {
    if debug_mode() {
        println!("[DEBUG] {}", message);
    }
}

pub fn log_info(message: &str) {
    if debug_mode() {
        println!("[INFO] {}", message);
    }
}

pub fn log_warning(message: &str) {
    if debug_mode() {
        println!("[WARNING] {}", message);
    }
}

pub fn log_error(message: &str) {
    if debug_mode() {
        eprintln!("[ERROR] {}", message);
    }
}
