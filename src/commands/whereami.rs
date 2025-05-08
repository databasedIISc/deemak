use std::path::Path;

/// Helper function to display paths relative to root with HOME prefix
pub fn display_relative_path(path: &Path, root_dir: &Path) -> String {
    path.strip_prefix(root_dir)
        .map(|p| {
            if p.components().count() == 0 {
                "HOME".to_string()
            } else {
                format!("HOME/{}", p.display())
            }
        })
        .unwrap_or_else(|_| path.display().to_string())
}

/// This is similar to the `pwd` command in Unix-like systems.
pub fn whereami(current_dir: &Path, root_dir: &Path) -> String {
    display_relative_path(current_dir, root_dir)
}
