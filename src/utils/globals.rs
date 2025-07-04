use once_cell::sync::{Lazy, OnceCell};
use std::path::PathBuf;
use std::sync::Mutex;

// Global variable to store the user ID. This will be set after the user logs in and can be accessed throughout the application.
pub static USER_ID: OnceCell<String> = OnceCell::new();

// Global variable to store the user password. This will be set after the user logs in and can be accessed throughout the application.
pub static USER_PASSWORD: OnceCell<String> = OnceCell::new();

// root dir can be acessed via the `WORLD_DIR` global variable
pub static WORLD_DIR: OnceCell<PathBuf> = OnceCell::new();

// Shell history to store the commands executed by the user.
pub static SHELL_HISTORY: Lazy<Mutex<Vec<String>>> = Lazy::new(|| Mutex::new(Vec::new()));
