use super::passlock::{check_dmk_magic, decrypt_file, encrypt_file};
use super::restore_comp::{zlib_compress, zlib_decompress};
use crate::utils::log;
use std::path::{Path, PathBuf};

/// Encrypts a Sekai folder into a Deemak encrypted file
pub fn deemak_encrypt_sekai(sekai_path: &Path, password: &str) -> Result<(), String> {
    let output_path = sekai_path.with_extension("deemak");

    // Skip if already encrypted
    if output_path.exists() && check_dmk_magic(&output_path)? {
        log::log_info("DEEMAK", "File already encrypted, skipping");
        return Ok(());
    }

    // Create temp zlib file in same directory
    let temp_zlib = sekai_path.with_extension("tmp.zlib");

    // Compress first
    if sekai_path.is_dir() {
        zlib_compress(sekai_path, &temp_zlib).map_err(|e| format!("Compression failed: {}", e))?;
    } else {
        return Err("Input must be a directory".to_string());
    }

    // Then encrypt
    encrypt_file(&temp_zlib, &output_path, password)
        .map_err(|e| format!("Encryption failed: {}", e))?;

    // Clean up
    std::fs::remove_file(&temp_zlib).map_err(|e| format!("Failed to clean up temp file: {}", e))?;
    log::log_info(
        "DEEMAK",
        &format!("Successfully encrypted to {}", output_path.display()),
    );
    println!(
        "Successfully encrypted Sekai folder to {}",
        output_path.display()
    );

    Ok(())
}

/// Extracts original path from encrypted Sekai file
pub fn original_from_encrypted_sekai(encrypted_path: &Path) -> Result<PathBuf, String> {
    // Validate input
    if !encrypted_path.is_file() {
        return Err("Path is not a file. Please input correct Deemak file.".to_string());
    }

    if !check_dmk_magic(encrypted_path)? {
        return Err("Not a valid Deemak file".to_string());
    }

    let output_dir = encrypted_path.with_extension("");
    let temp_zlib = encrypted_path.with_extension("tmp.zlib");

    // Decrypt first
    decrypt_file(encrypted_path, &temp_zlib).map_err(|e| format!("Decryption failed: {}", e))?;

    // Then decompress
    zlib_decompress(&temp_zlib, &output_dir).map_err(|e| format!("Decompression failed: {}", e))?;

    // Clean up
    std::fs::remove_file(&temp_zlib).map_err(|e| format!("Failed to clean up temp file: {}", e))?;

    log::log_info(
        "DEEMAK",
        &format!("Successfully decrypted to {}", output_dir.display()),
    );
    println!(
        "Successfully restored Sekai folder to {}",
        output_dir.display()
    );

    Ok(output_dir)
}
