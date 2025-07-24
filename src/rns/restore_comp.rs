use super::create_dmk_sekai::{deemak_encrypt_sekai, original_from_encrypted_sekai};
use crate::utils::log;
use std::fs;
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

pub fn generate_temp_path(usage: &str) -> PathBuf {
    let random_pass_hash = format!("{:x}", rand::random::<u64>());
    PathBuf::from(format!("/tmp/deemak-{usage}-{random_pass_hash}"))
}

fn copy_sekai_dir(from_location: &Path, to_location: &Path) -> io::Result<()> {
    // Move the restored files & directories to the original root path
    for entry in fs::read_dir(from_location)? {
        let entry = entry?;
        let path = entry.path();
        let target_path = to_location.join(path.file_name().unwrap());

        if path.is_dir() {
            // For directories, we need to create the directory and move contents
            fs::create_dir_all(&target_path)?;
            for file_entry in fs::read_dir(&path)? {
                let file_entry = file_entry?;
                let file_path = file_entry.path();
                let file_target = target_path.join(file_entry.file_name());

                // Use copy-then-delete for cross-device compatibility
                if file_path.is_dir() {
                    copy_dir_all(&file_path, &file_target)?;
                    fs::remove_dir_all(&file_path)?;
                } else {
                    fs::copy(&file_path, &file_target)?;
                    fs::remove_file(&file_path)?;
                }
            }
            // Remove the now-empty source directory
            fs::remove_dir(&path)?;
        } else {
            // For files, use copy-then-delete
            fs::copy(&path, &target_path)?;
            fs::remove_file(&path)?;
        }
    }

    // Helper function to recursively copy directories
    fn copy_dir_all(src: &Path, dst: &Path) -> io::Result<()> {
        fs::create_dir_all(dst)?;
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let ty = entry.file_type()?;
            let target = dst.join(entry.file_name());
            if ty.is_dir() {
                copy_dir_all(&entry.path(), &target)?;
            } else {
                fs::copy(entry.path(), &target)?;
            }
        }
        Ok(())
    }
    Ok(())
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
        return Ok(format!(
            "{backup_file:?} file already exists, skipping creation."
        ));
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
            "Failed to create Deemak encrypted file: {e}"
        )));
    }
    Ok(format!("Backup {usage} created at {backup_file:?}"))
}

/// Restores Sekai data from a Deemak encrypted file
pub fn restore_sekai(usage: &str, root_path: &Path) -> io::Result<String> {
    let dir_info_path = root_path.join(".dir_info");
    // Save the restore/save files temporarily and delete them from the original location
    let temp_restore = generate_temp_path("temp_restore");
    let temp_save = generate_temp_path("temp_save");
    let mut restore_moved = false;
    let mut save_moved = false;

    if dir_info_path.join(RESTORE_FILE).exists() {
        fs::copy(dir_info_path.join(RESTORE_FILE), &temp_restore)?;
        fs::remove_file(dir_info_path.join(RESTORE_FILE))?;
        restore_moved = true;
    }
    if dir_info_path.join(SAVE_FILE).exists() {
        fs::copy(dir_info_path.join(SAVE_FILE), &temp_save)?;
        fs::remove_file(dir_info_path.join(SAVE_FILE))?;
        save_moved = true;
    }
    // Validate usage parameter
    let backup_file = match usage {
        "restore" => &temp_restore,
        "save" => &temp_save,
        _ => {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "Invalid usage: must be 'restore' or 'save'",
            ));
        }
    };

    // Clear directory while preserving .dir_info
    for entry in fs::read_dir(root_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            fs::remove_dir_all(path)?;
        } else {
            fs::remove_file(path)?;
        }
    }

    let temp_restore_dir = generate_temp_path("temp_restore_dir");
    fs::create_dir_all(&temp_restore_dir).unwrap_or_else(|_| {
        log::log_error(
            "SEKAI",
            &format!(
                "Failed to create temporary restore directory: {}",
                temp_restore_dir.display()
            ),
        );
    });
    // Decrypt the backup
    let restored_path =
        match original_from_encrypted_sekai(backup_file, temp_restore_dir.as_path(), None) {
            Ok(path) => path,
            Err(e) => {
                return Err(Error::other(format!(
                    "Failed to restore Sekai to temporary directory: {e}"
                )));
            }
        };

    // Move the restored files & directories to the original root path
    if let Err(e) = copy_sekai_dir(&restored_path, root_path) {
        return Err(Error::other(format!(
            "Failed to copy restored files to original path: {e:?}",
        )));
    }
    log::log_info(
        "SEKAI",
        &format!(
            "Successfully restored Sekai from {usage} file to: {}",
            root_path.display()
        ),
    );

    // Add the restored file to the .dir_info directory
    if restore_moved {
        fs::copy(&temp_restore, dir_info_path.join(RESTORE_FILE))?;
        fs::remove_file(temp_restore)?;
    }
    if save_moved {
        fs::copy(&temp_save, dir_info_path.join(SAVE_FILE))?;
        fs::remove_file(temp_save)?;
    }

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
