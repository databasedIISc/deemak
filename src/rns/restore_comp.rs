use super::create_dmk_sekai::{deemak_encrypt_sekai, original_from_encrypted_sekai};
use crate::utils::log;
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io;
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};

/*
RESTORE MECHANISM EXPLANATION:

When restoring, all the files including the `.dir_info` directory(of HOME as well), will be compressed into a zlib file and encrypted with Deemak Encryption.
This is called `restore_me.deemak` or `save_me.deemak` depending on the usage. Both should exist/created at the start of the Shell.

Any of these restore files should not contain the (HOME/.dir_info/)restore_me.deemak or save_me.deemak files.
When restoring, the sekai will be cleared and restored from the restore file
During the whole program, `restore_me.deemak` will remain untouched and unchanged. Only `save_me.deemak` will be created or updated with the current state of Sekai.

- When `restore` is called, it will look for `restore_me.deemak` in the `.dir_info` directory.
In this case, the `restore_me.deemak` will be copied to `save_me.deemak` so that it starts with a fresh state.

- When `save` is called, it will look for `save_me.deemak` in the `.dir_info` directory.
In this case, the `restore_me.deemak` will remain unchanged, and the `save_me.deemak` will be created or updated with the current state of Sekai.
*/

const RESTORE_FILE: &str = "restore_me.deemak";
const SAVE_FILE: &str = "save_me.deemak";

fn generate_temp_path(usage: &str, root_path: &Path) -> PathBuf {
    let mut hasher = DefaultHasher::new();
    root_path.hash(&mut hasher);
    let hash = hasher.finish();
    PathBuf::from(format!("/tmp/deemak-{usage}-{hash:x}"))
}

/// Backs up Sekai data to a Deemak encrypted file
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

    let random_pass_hash = format!("{:x}", rand::random::<u64>());
    let password = random_pass_hash + "_" + usage;

    if let Err(e) = deemak_encrypt_sekai(
        root_path,
        &root_path.join(".dir_info").join(&backup_file),
        password.as_str(),
        true, // Force encryption
    ) {
        return Err(Error::other(format!(
            "Failed to create Deemak encrypted file: {}",
            e
        )));
    }
    Ok(format!("Backup {usage} created at {backup_file:?}"))
}

/// Restores Sekai data from a Deemak encrypted file
pub fn restore_sekai(usage: &str, root_path: &Path) -> io::Result<String> {
    // Validate usage parameter
    let backup_file = match usage {
        "restore" => RESTORE_FILE,
        "save" => SAVE_FILE,
        _ => {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "Invalid usage: must be 'restore' or 'save'",
            ));
        }
    };

    let source_file = root_path.join(".dir_info").join(backup_file);
    let temp_path = generate_temp_path(usage, root_path);

    // Copy backup to temp location
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

    // Decrypt the backup
    let restored_path = match original_from_encrypted_sekai(
        &temp_path,
        &root_path.join(".dir_info").join(backup_file),
    ) {
        Ok(path) => {
            log::log_info(
                "SEKAI",
                &format!("Successfully restored Sekai to: {}", path.display()),
            );
            path
        }
        Err(e) => {
            // Clean up temp file before returning error
            let _ = fs::remove_file(&temp_path);
            return Err(Error::other(format!(
                "Failed to restore Sekai from backup: {}",
                e
            )));
        }
    };

    // Clean up temp file
    fs::remove_file(temp_path)?;

    Ok(format!(
        "Successfully restored Sekai from {usage} file at: {}",
        restored_path.display()
    ))
}

pub fn can_restore(root_path: &Path) -> bool {
    root_path.join(".dir_info").join(RESTORE_FILE).exists()
}

pub fn can_save(root_path: &Path) -> bool {
    root_path.join(".dir_info").join(SAVE_FILE).exists()
}
