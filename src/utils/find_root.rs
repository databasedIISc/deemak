use super::read_validate_info;
use std::path::PathBuf;

pub fn find_home(world_path: &PathBuf) -> Option<PathBuf> {
    // let mut current = std::env::current_dir().ok()?;
    let mut current = world_path.clone();
    let max_depth = 10; // Prevent infinite recursion

    for _ in 0..max_depth {
        // Check all directories at this level
        if let Ok(entries) = std::fs::read_dir(&current) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let info_path = path.join(".dir_info").join("info.json");
                    if let Ok(info) = read_validate_info(&info_path) {
                        if info.location == "home" {
                            return Some(path);
                        }
                    }
                }
            }
        }

        // Move up one directory level
        if !current.pop() {
            break;
        }
    }

    None
}
