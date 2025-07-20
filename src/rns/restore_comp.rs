use super::passlock::{decrypt_file, encrypt_file};
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io;
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};

const RESTORE_FILE: &str = "restore_me";
const SAVE_FILE: &str = "save_me";

fn generate_temp_path(usage: &str, root_path: &Path) -> PathBuf {
    let mut hasher = DefaultHasher::new();
    root_path.hash(&mut hasher);
    let hash = hasher.finish();
    PathBuf::from(format!("/tmp/deemak-{usage}-{hash:x}"))
}

pub fn backup_sekai(usage: &str, root_path: &Path) -> io::Result<String> {
    let dir_info_path = root_path.join(".dir_info");
    fs::create_dir_all(&dir_info_path)?;

    let backup_file = match usage {
        "restore" => dir_info_path.join(RESTORE_FILE),
        "save" => dir_info_path.join(SAVE_FILE),
        _ => {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "Invalid usage: must be 'restore' or 'save'",
            ));
        }
    };

    if usage == "restore" && backup_file.exists() {
        return Ok("Restore file already exists, skipping creation.".to_string());
    }

    encrypt_file(
        &root_path.join(".dir_info").join(RESTORE_FILE),
        &backup_file,
        format!(
            "{sekai}_{usage}",
            sekai = root_path.display(),
            usage = usage
        )
        .as_str(),
    );
    Ok(format!("Backup {usage} created at {backup_file:?}"))
}

pub fn restore_sekai(usage: &str, root_path: &Path) -> io::Result<()> {
    let source_file = match usage {
        "restore" => root_path.join(".dir_info").join(RESTORE_FILE),
        "save" => root_path.join(".dir_info").join(SAVE_FILE),
        _ => {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "Invalid usage: must be 'restore' or 'save'",
            ));
        }
    };

    let temp_path = generate_temp_path(usage, root_path);
    fs::copy(&source_file, &temp_path)?;

    // Clear directory while preserving .dir_info
    for entry in fs::read_dir(root_path)? {
        let entry = entry?;
        let path = entry.path();

        if path == root_path.join(".dir_info") {
            if usage == "restore" {
                let save_path = path.join(SAVE_FILE);
                if save_path.exists() {
                    fs::copy(root_path.join(".dir_info").join(RESTORE_FILE), save_path)?;
                }
            }
            continue;
        }

        if path.is_dir() {
            fs::remove_dir_all(path)?;
        } else {
            fs::remove_file(path)?;
        }
    }

    zlib_decompress(&temp_path, root_path)?;
    fs::remove_file(temp_path)?;
    Ok(())
}

pub fn can_restore(root_path: &Path) -> bool {
    root_path.join(".dir_info").join(RESTORE_FILE).exists()
}

pub fn can_save(root_path: &Path) -> bool {
    root_path.join(".dir_info").join(SAVE_FILE).exists()
}
