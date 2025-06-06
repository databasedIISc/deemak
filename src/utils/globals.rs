use once_cell::sync::OnceCell;
use std::path::PathBuf;

// root dir can be acessed via the `WORLD_DIR` global variable
pub static WORLD_DIR: OnceCell<PathBuf> = OnceCell::new();
