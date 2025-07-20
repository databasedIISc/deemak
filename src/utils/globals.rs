use crate::SEKAI_DIR;
use once_cell::sync::{Lazy, OnceCell};
use std::path::PathBuf;
use std::sync::Mutex;

/// Shell history to store the commands executed by the user.
pub static SHELL_HISTORY: Lazy<Mutex<Vec<String>>> = Lazy::new(|| Mutex::new(Vec::new()));

/// Global user information instance
pub static USER_INFO: OnceCell<UserInfo> = OnceCell::new();

/// User information structure with expandable functionality
#[derive(Debug, Clone)]
pub struct UserInfo {
    pub username: String,
    pub salt: String,
    pub password_hash: String,
    pub is_authenticated: bool,
    pub login_time: Option<std::time::SystemTime>,
}

impl UserInfo {
    /// Create a new UserInfo instance
    pub fn new(username: String, salt: String, password_hash: String) -> Self {
        Self {
            username,
            salt,
            password_hash,
            is_authenticated: false,
            login_time: None,
        }
    }

    /// Create a default UserInfo instance
    pub fn default() -> Self {
        Self {
            username: String::new(),
            salt: String::new(),
            password_hash: String::new(),
            is_authenticated: false,
            login_time: None,
        }
    }

    /// Authenticate the user
    pub fn authenticate(&mut self) {
        self.is_authenticated = true;
        self.login_time = Some(std::time::SystemTime::now());
    }

    /// Check if user is authenticated
    pub fn is_authenticated(&self) -> bool {
        self.is_authenticated
    }

    /// Get username
    pub fn get_username(&self) -> &str {
        &self.username
    }

    /// Set user credentials
    pub fn set_credentials(&mut self, username: String, salt: String, password_hash: String) {
        self.username = username;
        self.salt = salt;
        self.password_hash = password_hash;
    }

    /// Clear user information (logout)
    pub fn clear(&mut self) {
        self.username.clear();
        self.salt.clear();
        self.password_hash.clear();
        self.is_authenticated = false;
        self.login_time = None;
    }

    /// Get login duration
    pub fn get_login_duration(&self) -> Option<std::time::Duration> {
        if let Some(login_time) = self.login_time {
            std::time::SystemTime::now().duration_since(login_time).ok()
        } else {
            None
        }
    }
}

/// Gets the global if it has been set, otherwise returns None.
pub fn get_global_once<T: Clone>(cell: &OnceCell<T>) -> &T {
    cell.get().unwrap()
}

/// Sets the global variable if it has not been set yet.
pub fn set_global_once<T>(cell: &OnceCell<T>, value: T) -> Result<(), T> {
    cell.set(value)
}

/// Get global user info
pub fn get_user_info() -> Option<&'static UserInfo> {
    USER_INFO.get()
}

/// Set global user info
pub fn set_user_info(user_info: UserInfo) -> Result<(), UserInfo> {
    USER_INFO.set(user_info)
}

/// Initialize or update global user info
pub fn init_user_info(username: String, salt: String, password_hash: String) {
    let user_info = UserInfo::new(username, salt, password_hash);
    // The `set` method returns Ok(()) if the cell was empty and is now filled,
    // and Err(value) if the cell was already filled.
    // We can ignore the result if we don't need to handle the case where it's already set.
    let _ = USER_INFO.set(user_info);
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
