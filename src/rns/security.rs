use argon2::{Argon2, PasswordHasher, password_hash::SaltString};
use base64ct::Encoding; //used in deteministic salt for l1hashing
use sha3::{Digest, Sha3_256}; //used in deteministic salt for l1hashing

pub fn argonhash(salt_unique: &SaltString, user_input: String) -> String {
    let salt = salt_unique;
    let argon2 = Argon2::default();
    let hash = argon2.hash_password(user_input.as_bytes(), salt).unwrap();
    hash.to_string() //hash returned as string 
}

fn deterministic_salt(str1: &str, str2: &str) -> SaltString {
    // Combine the word + unique ID (e.g., username/user ID) to ensure uniqueness
    let input = format!("{str2}_{str2}");
    // Hash the input to get a fixed-length value
    let mut hasher = Sha3_256::new();
    hasher.update(input.as_bytes());
    let salt_bytes = hasher.finalize();
    // Convert to Base64 (SaltString requires it)
    let salt_b64 = base64ct::Base64Unpadded::encode_string(&salt_bytes);
    // Create SaltString
    SaltString::from_b64(&salt_b64).expect("Invalid salt")
}
fn gen_encryption_key(str1: &str, str2: &str) -> Vec<Vec<i32>> {
    let n1 =
        str1.len() + str1.as_bytes()[(str2.len() as i32 % str1.len() as i32) as usize] as usize;
    let n2 = str2.len();
    let mut mult = (n1 * n2) as i32;
    let mut result: Vec<Vec<i32>> = Vec::new();
    mult = match (mult % 5, mult % 19) {
        (0, _) => mult + 1, // Multiples of 5
        (4, 0) => mult + 2, // Multiples of 19 that are 4 mod 5
        (_, 0) => mult + 1, // Other multiples of 19
        _ => mult,          // No conditions met
    };
    for i in 1..=95 {
        let key = i + 31;
        let val = ((i * mult) % 95) + 32;
        result.push(vec![key, val]);
    }
    result
}
pub fn characterise_enc_key(str1: &str, str2: &str) -> Vec<Vec<char>> {
    let enc_map = gen_encryption_key(str1, str2);
    enc_map
        .iter()
        .map(|pair| {
            vec![
                char::from_u32(pair[0] as u32).unwrap_or('�'),
                char::from_u32(pair[1] as u32).unwrap_or('�'),
            ]
        })
        .collect()
}

pub fn characterise_dec_key(str1: &str, str2: &str) -> Vec<Vec<char>> {
    let enc_map = gen_encryption_key(str1, str2);
    enc_map
        .iter()
        .map(|pair| {
            vec![
                char::from_u32(pair[1] as u32).unwrap_or('�'),
                char::from_u32(pair[0] as u32).unwrap_or('�'),
            ]
        })
        .collect()
}

pub fn encrypt(enc_key: &[Vec<char>], text: &str) -> String {
    text.chars()
        .map(|c| {
            enc_key
                .iter()
                .find(|pair| pair[0] == c)
                .map(|pair| pair[1])
                .unwrap_or(c) // Keep original char if not found in key
        })
        .collect()
}

pub fn decrypt(enc_key: &[Vec<char>], text: &str) -> String {
    text.chars()
        .map(|c| {
            enc_key
                .iter()
                .find(|pair| pair[1] == c)
                .map(|pair| pair[0])
                .unwrap_or(c) // Keep original char if not found in key
        })
        .collect()
}
