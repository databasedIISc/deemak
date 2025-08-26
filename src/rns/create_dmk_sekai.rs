use super::passlock::{check_dmk_magic, decrypt_file, encrypt_file};
use super::passlock::{zlib_compress, zlib_decompress};
use super::restore_comp::generate_temp_path;
use crate::utils::log;
use std::path::{Path, PathBuf};

/// Encrypts a Sekai folder into a Deemak encrypted file
pub fn deemak_encrypt_sekai(
    sekai_path: &Path,
    output_path: &Path,
    password: &str,
    force: bool,
) -> Result<(), String> {
    log::log_debug(
        "Deemak Encryption",
        &format!(
            "Input Sekai Path: {}, Output Path: {}",
            sekai_path.display(),
            output_path.display(),
        ),
    );
    // Based on the type of output path, we make the encryption path
    let encryption_path: &Path = if output_path.extension().is_some_and(|ext| ext == "deemak") {
        output_path
    } else {
        let filename = sekai_path.file_name().unwrap();
        &output_path.join(filename).with_extension("deemak")
    };

    // Skip if already encrypted
    if !force && encryption_path.exists() && check_dmk_magic(encryption_path)? {
        log::log_info("DEEMAK", "File already encrypted, skipping");
        return Ok(());
    }

    // Create temp zlib file in same directory
    let temp_zlib = generate_temp_path("zlib_compress").with_extension("tmp.zlib");

    // Compress first
    if sekai_path.is_dir() {
        zlib_compress(sekai_path, &temp_zlib).map_err(|e| format!("Compression failed: {e}"))?;
        log::log_info(
            "Deemak Encryption",
            format!("Successfully compressed Sekai: {}", temp_zlib.display()).as_str(),
        );
    } else {
        return Err("Input must be a directory".to_string());
    }
    // Then encrypt
    encrypt_file(&temp_zlib, encryption_path, password)
        .map_err(|e| format!("Encryption failed: {e}"))?;

    // Clean up
    std::fs::remove_file(&temp_zlib).map_err(|e| format!("Failed to clean up temp file: {e}"))?;
    log::log_info(
        "DEEMAK",
        &format!("Successfully encrypted to {}", encryption_path.display()),
    );
    Ok(())
}

/// Extracts original path from encrypted Sekai file
pub fn original_from_encrypted_sekai(
    encrypted_path: &Path,
    output_path: &Path,
    password: Option<&str>,
) -> Result<PathBuf, String> {
    // Prepare paths
    let output_dir = if output_path.is_dir() {
        output_path.join(
            encrypted_path
                .file_stem()
                .ok_or("Invalid encrypted filename")?
                .to_string_lossy()
                .trim_end_matches(".deemak"),
        )
    } else {
        output_path.to_path_buf()
    };

    let temp_zlib = generate_temp_path("zlib_compress").with_extension("tmp.zlib");

    // Ensure clean state
    if output_dir.exists() {
        return Err(format!(
            "Output path already exists: {}",
            output_dir.display()
        ));
    }

    // Perform operations with proper error handling and cleanup
    let result = (|| {
        // Decrypt first
        decrypt_file(encrypted_path, &temp_zlib, password)
            .map_err(|e| format!("Decryption failed: {e}"))?;

        // Then decompress
        zlib_decompress(&temp_zlib, &output_dir)
            .map_err(|e| format!("Decompression failed: {e}"))?;

        Ok(&output_dir)
    })();

    // Clean up temp file in all cases
    if temp_zlib.exists()
        && let Err(e) = std::fs::remove_file(&temp_zlib) {
            log::log_error("DEEMAK", &format!("Failed to clean up temp file: {e}"));
        }

    match result {
        Ok(dir) => {
            log::log_info(
                "DEEMAK",
                &format!("Successfully decrypted to {}", dir.display()),
            );
            Ok(dir.to_path_buf())
        }
        Err(e) => {
            // Clean up partial output if operation failed
            if output_dir.exists() {
                let _ = std::fs::remove_dir_all(&output_dir);
            }
            Err(e)
        }
    }
}
