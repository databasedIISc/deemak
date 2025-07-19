use crate::SEKAI_DIR;
use once_cell::sync::{Lazy, OnceCell};
use std::path::PathBuf;
use std::sync::Mutex;
pub static USER_NAME: OnceCell<String> = OnceCell::new();
pub static USER_SALT: OnceCell<String> = OnceCell::new();
pub static USER_PASSWORD: OnceCell<String> = OnceCell::new();
// Shell history to store the commands executed by the user.
pub static SHELL_HISTORY: Lazy<Mutex<Vec<String>>> = Lazy::new(|| Mutex::new(Vec::new()));

/// Gets the global if it has been set, otherwise returns None.
pub fn get_global_once<T: Clone>(cell: &OnceCell<T>) -> &T {
    cell.get().unwrap()
}

/// Sets the global variable if it has not been set yet.
pub fn set_global_once<T>(cell: &OnceCell<T>, value: T) -> Result<(), T> {
    cell.set(value)
}

// Specialized getter/setter for WORLD_DIR
pub fn get_world_dir() -> PathBuf {
    let world_dir = SEKAI_DIR.get();

    // Return original path if WORLD_DIR is empty
    if let Some(dir) = world_dir {
        if dir.is_empty() {
            return PathBuf::new();
        }
        return PathBuf::from(dir);
    }
    PathBuf::new()
}

/// Set the WORLD_DIR global variable.
pub fn set_world_dir(path: PathBuf) {
    SEKAI_DIR
        .set(path.to_string_lossy().to_string())
        .expect("WORLD_DIR already set");
}
