use once_cell::sync::OnceCell;
use std::path::PathBuf;

// Global variable to store the user ID. This will be set after the user logs in and can be accessed throughout the application.
pub static USER_ID: OnceCell<String> = OnceCell::new();

// Global variable to store the user password. This will be set after the user logs in and can be accessed throughout the application.
pub static USER_PASSWORD: OnceCell<String> = OnceCell::new();

// root dir can be acessed via the `WORLD_DIR` global variable
pub static WORLD_DIR: OnceCell<PathBuf> = OnceCell::new();
