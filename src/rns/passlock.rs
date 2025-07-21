use aes_gcm::aead::{Aead, rand_core::OsRng, rand_core::RngCore};
use aes_gcm::{Aes256Gcm, Key, KeyInit, Nonce}; // AES-GCM
use flate2::Compression;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use sha2::{Digest, Sha256};
use std::fs::{File, read, write};
use std::io::{self, Write};
use std::path::Path;
use tar::{Archive, Builder};
use walkdir::WalkDir;

use crate::utils::log;

const NONCE_SIZE: usize = 12;
const MAGIC_HEADER: &[u8; 8] = b"dbdeemak";

/*
ORDER OF OPERATIONS FOR DEEMAK ENCRYPTION:

Naming Convention:
- Zlib Compressed File: `.tmp.zlib`
- Deemak Encrypted File: `.deemak`

I. Creating a Deemak Encrypted Sekai file:
Requirements: - Sekai file/directory which is already not encrypted
            - Password for encryption

Process:
1. Check for requirements and proceed if valid.
2. Compress the Sekai directory to a zlib tarball.
3. Encrypt the zlib tarball using AES-GCM with the provided password.
4. Write the encrypted data to a file with a .deemak extension and add headers.
5. Clean up temporary files if necessary.

II. Restoring original Sekai from Deemak Encrypted file:
Requirements: - Deemak encrypted file

Process:
1. Check if the file is a valid Deemak file by checking the magic header and xattr metadata.
2. Read password from xattr metadata.
3. Decrypt the file using AES-GCM with the password to obtain the zlib tarball.
4. Decompress the zlib tarball to restore the original Sekai directory.
5. Clean up temporary files if necessary.

USE CASES OF THIS MODULE:
1. Deemak will read a Sekai file which is Deemak Encrypted.
2. restore_me, save_me, etc. files will all be Deemak Encrypted files.
*/

/// Compresses a file/directory to a zlib tarball
pub fn zlib_compress(source_path: &Path, output_file: &Path) -> io::Result<()> {
    let file = File::create(output_file)?;
    let mut encoder = ZlibEncoder::new(file, Compression::best());

    {
        let mut tar_builder = Builder::new(&mut encoder);
        for entry in WalkDir::new(source_path).min_depth(1) {
            let entry = entry?;
            let path = entry.path();
            let relative_path = path.strip_prefix(source_path).unwrap();

            if path.is_file() {
                tar_builder.append_file(relative_path, &mut File::open(path)?)?;
            } else if path.is_dir() {
                tar_builder.append_dir(relative_path, path)?;
            }
        }
        tar_builder.finish()?;
    } // tar_builder drops here, releasing its borrow of encoder

    encoder.finish()?;
    Ok(())
}

/// Decompresses a zlib tarball to a file/directory
pub fn zlib_decompress(archive_path: &Path, output_path: &Path) -> io::Result<()> {
    let file = File::open(archive_path)?;
    let decoder = ZlibDecoder::new(file);
    let mut archive = Archive::new(decoder);
    archive.unpack(output_path)?;
    Ok(())
}

fn derive_key_from_password(password: &str) -> Key<Aes256Gcm> {
    let mut hasher = Sha256::new_with_prefix(password.as_bytes());
    let hash = hasher.finalize();
    *Key::<Aes256Gcm>::from_slice(&hash[..32])
}

pub fn encrypt_file(input_path: &Path, output_path: &Path, password: &str) -> Result<(), String> {
    log::log_debug(
        "Encryption",
        &format!(
            "Input Path: {}, Output Path: {}",
            input_path.display(),
            output_path.display()
        ),
    );

    let output_path = &output_path.with_extension("deemak");
    let key = derive_key_from_password(password);
    let cipher = Aes256Gcm::new(&key);

    let plaintext = read(input_path).map_err(|e| format!("Failed to read input file: {e}"))?;

    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|_| "Encryption failed".to_string())?;

    let mut file =
        File::create(output_path).map_err(|e| format!("Failed to create output file: {e}"))?;

    file.write_all(MAGIC_HEADER)
        .map_err(|e| format!("Failed to write header: {e}"))?;
    file.write_all(&nonce_bytes)
        .map_err(|e| format!("Failed to write nonce: {e}"))?;
    file.write_all(&ciphertext)
        .map_err(|e| format!("Failed to write ciphertext: {e}"))?;

    xattr::set(output_path, "pass.deemak", password.as_bytes())
        .map_err(|e| format!("Failed to set xattr metadata: {e}"))?;

    Ok(())
}

pub fn check_dmk_magic(sekai_path: &Path) -> Result<bool, String> {
    let data =
        std::fs::read(sekai_path).map_err(|e| format!("Failed to read input file: {e}"))?;

    // Check magic header exists and matches
    let magic_ok = data.len() >= MAGIC_HEADER.len() + NONCE_SIZE && data.starts_with(MAGIC_HEADER);

    // Check xattr exists
    let xattr_ok = xattr::get(sekai_path, "pass.deemak").is_ok();
    Ok(magic_ok && xattr_ok)
}

pub fn decrypt_file(input_path: &Path, output_path: &Path) -> Result<(), String> {
    // Check if the file has the correct magic header
    if !check_dmk_magic(input_path)? {
        return Err("File does not have the correct magic header".to_string());
    }
    let data = read(input_path).map_err(|e| format!("Failed to read input file: {e}"))?;

    if data.len() < MAGIC_HEADER.len() + NONCE_SIZE {
        return Err("Invalid encrypted file".to_string());
    }

    let (magic, rest) = data.split_at(MAGIC_HEADER.len());
    if magic != MAGIC_HEADER {
        return Err("Invalid magic header, not a valid .deemak file".to_string());
    }

    let (nonce_bytes, ciphertext) = rest.split_at(NONCE_SIZE);
    let nonce = Nonce::from_slice(nonce_bytes);

    let password_bytes = xattr::get(input_path, "pass.deemak")
        .map_err(|e| format!("Failed to read metadata: {e}"))?
        .ok_or_else(|| "Missing password metadata".to_string())?;

    let password = String::from_utf8(password_bytes)
        .map_err(|_| "Password metadata is not valid UTF-8".to_string())?;

    let key = derive_key_from_password(&password);
    let cipher = Aes256Gcm::new(&key);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| "Decryption failed — wrong password or corrupted file".to_string())?;

    write(output_path, plaintext).map_err(|e| format!("Failed to write decrypted file: {e}"))?;

    Ok(())
}
