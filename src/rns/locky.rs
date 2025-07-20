use aes_gcm::aead::{Aead, rand_core::OsRng, rand_core::RngCore};
use aes_gcm::{Aes256Gcm, Key, KeyInit, Nonce}; // AES-GCM
use sha2::{Digest, Sha256};
use std::fs::{File, read, write};
use std::io::{self, Read, Write};
use std::path::Path;

const NONCE_SIZE: usize = 12;

fn derive_key_from_password(password: &str) -> Key<Aes256Gcm> {
    let mut hasher = Sha256::new_with_prefix(password.as_bytes());
    let hash = hasher.finalize();
    *Key::<Aes256Gcm>::from_slice(&hash[..32])
}

const MAGIC_HEADER: &[u8; 8] = b"dbdeemak";

fn encrypt_file(input_path: &Path, output_path: &Path, password: &str) -> io::Result<()> {
    let key = derive_key_from_password(password);
    let cipher = Aes256Gcm::new(&key);

    let mut plaintext = Vec::new();
    File::open(input_path)?.read_to_end(&mut plaintext)?;

    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .expect("encryption failed");

    let mut file = File::create(output_path)?;
    file.write_all(MAGIC_HEADER)?; // ⬅ Add header here
    file.write_all(&nonce_bytes)?;
    file.write_all(&ciphertext)?;

    xattr::set(output_path, "pass.deemak", password.as_bytes())?;
    Ok(())
}

fn decrypt_file(input_path: &Path, output_path: &Path) -> io::Result<()> {
    let data = read(input_path)?;
    if data.len() < MAGIC_HEADER.len() + NONCE_SIZE {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid file"));
    }

    let (magic, rest) = data.split_at(MAGIC_HEADER.len());
    if magic != MAGIC_HEADER {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid magic header",
        ));
    }

    let (nonce_bytes, ciphertext) = rest.split_at(NONCE_SIZE);
    let nonce = Nonce::from_slice(nonce_bytes);

    let password_bytes = xattr::get(input_path, "pass.deemak")?
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Missing metadata"))?;
    let password = String::from_utf8(password_bytes)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid UTF-8 in password"))?;

    let key = derive_key_from_password(&password);
    let cipher = Aes256Gcm::new(&key);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Decryption failed"))?;

    write(output_path, plaintext)?;
    Ok(())
}

// fn main() -> io::Result<()> {
//     // Ask user for input file
//     println!("Enter path to file to encrypt:");
//     let mut input_path = String::new();
//     io::stdin().read_line(&mut input_path)?;
//     let input_path = input_path.trim();
//
//     // Ask user for password
//     println!("Enter password:");
//     let mut password = String::new();
//     io::stdin().read_line(&mut password)?;
//     let password = password.trim();
//
//     // Encrypt
//     let encrypted_path = format!("{}.deemak", input_path);
//     encrypt_file(input_path, &encrypted_path, password)?;
//     println!("Encrypted to: {}", encrypted_path);
//
//     // Decrypt
//     let filename = Path::new(input_path)
//         .file_name()
//         .unwrap()
//         .to_string_lossy()
//         .into_owned();
//     let decrypted_path = format!("decrypted_{}", filename);
//     decrypt_file(&encrypted_path, &decrypted_path)?;
//     println!("Decrypted to: {}", decrypted_path);
//
//     Ok(())
// }
