use flate2::Compression;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use std::collections::hash_map::DefaultHasher;
use std::fs::{self, File};
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use walkdir::WalkDir;

const RESTORE_FILE: &str = "restore_me";
const SAVE_FILE: &str = "save_me";

fn generate_temp_path(usage: &str, root_path: &PathBuf) -> PathBuf {
    let mut hasher = DefaultHasher::new();
    root_path.hash(&mut hasher);
    let hash = hasher.finish();
    PathBuf::from(format!("/tmp/deemak-{}-{:x}", usage, hash))
}

pub fn backup_sekai(usage: &str, root_path: &PathBuf) -> std::io::Result<String> {
    let dir_info_path = root_path.join(".dir_info");
    fs::create_dir_all(&dir_info_path)?;

    let backup_file = match usage {
        "restore" => dir_info_path.join(RESTORE_FILE),
        "save" => dir_info_path.join(SAVE_FILE),
        _ => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid usage: must be 'restore' or 'save'",
            ));
        }
    };

    // if `restore_me` already exists, then do not recreate it
    if usage == "restore" && dir_info_path.join(RESTORE_FILE).exists() {
        return Ok("Restore file already exists, skipping creation.".to_string());
    }

    let file = File::create(&backup_file)?;
    let mut encoder = ZlibEncoder::new(file, Compression::best());

    {
        let mut tar_builder = tar::Builder::new(&mut encoder);

        for entry in WalkDir::new(root_path).min_depth(1) {
            let entry = entry?;
            let path = entry.path();

            if path == backup_file {
                continue;
            }

            let relative_path = path.strip_prefix(root_path).unwrap();

            if path.is_file() {
                tar_builder.append_file(relative_path, &mut File::open(path)?)?;
            } else if path.is_dir() {
                tar_builder.append_dir(relative_path, path)?;
            }
        }

        tar_builder.finish()?;
    }

    encoder.finish()?;
    Ok(format!("Backup created at {:?}", backup_file))
}

pub fn restore_sekai(usage: &str, root_path: &PathBuf) -> std::io::Result<()> {
    let source_file = match usage {
        "restore" => root_path.join(".dir_info").join(RESTORE_FILE),
        "save" => root_path.join(".dir_info").join(SAVE_FILE),
        _ => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid usage: must be 'restore' or 'save'",
            ));
        }
    };

    let temp_path = generate_temp_path(usage, root_path);
    fs::copy(&source_file, &temp_path)?;

    // Clear directory (keeping .dir_info but removing save file if restoring)
    for entry in fs::read_dir(root_path)? {
        let entry = entry?;
        let path = entry.path();

        // Preserve .dir_info but clean contents appropriately
        if path == root_path.join(".dir_info") {
            if usage == "restore" {
                // Remove save_me when doing full restore
                let save_path = path.join(SAVE_FILE);
                if save_path.exists() {
                    fs::remove_file(save_path)?;
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

    // Restore from backup
    let file = File::open(&temp_path)?;
    let decoder = ZlibDecoder::new(file);
    let mut archive = tar::Archive::new(decoder);
    archive.unpack(root_path)?;

    // Clean up temporary file
    fs::remove_file(temp_path)?;
    Ok(())
}

pub fn can_restore(root_path: &PathBuf) -> bool {
    root_path.join(".dir_info").join(RESTORE_FILE).exists()
}

pub fn can_save(root_path: &PathBuf) -> bool {
    root_path.join(".dir_info").join(SAVE_FILE).exists()
}
