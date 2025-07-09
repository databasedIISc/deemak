use rocket::form::Form;
use rocket::serde::json;
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::{post, FromForm};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use ring::{digest, pbkdf2, rand::{self, SecureRandom}};
use data_encoding::HEXUPPER;
use std::num::NonZeroU32;

const USER_FILE: &str = "database.json";
const ITERATIONS: NonZeroU32 = unsafe { NonZeroU32::new_unchecked(100_000) };
const CREDENTIAL_LEN: usize = digest::SHA512_OUTPUT_LEN;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    pub username: String,
    pub salt: String,
    pub password_hash: String,
}
#[derive(FromForm, Deserialize, Serialize)]
pub struct AuthInput {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    status: bool,
    message: String,
}
// File-based DB
pub fn load_users() -> Vec<User> {
    if !Path::new(USER_FILE).exists() {
        return vec![];
    }

    let mut file = File::open(USER_FILE).expect("Failed to open file");
    let mut data = String::new();
    file.read_to_string(&mut data).expect("Failed to read file");
    serde_json::from_str(&data).unwrap_or_else(|_| {
        eprintln!("Failed to parse user JSON");
        vec![]
    })
}

fn save_users(users: &[User]) {
    let data = serde_json::to_string_pretty(users).expect("Failed to serialize users");
    let mut file = File::create(USER_FILE).expect("Failed to write file");
    file.write_all(data.as_bytes()).unwrap();
}

// Password hashing
fn hash_password(password: &str) -> Result<(String, String), ring::error::Unspecified> {
    let rng = rand::SystemRandom::new();
    let mut salt = [0u8; CREDENTIAL_LEN];
    rng.fill(&mut salt)?;

    let mut hash = [0u8; CREDENTIAL_LEN];
    pbkdf2::derive(
        pbkdf2::PBKDF2_HMAC_SHA512,
        ITERATIONS,
        &salt,
        password.as_bytes(),
        &mut hash,
    );

    Ok((HEXUPPER.encode(&salt), HEXUPPER.encode(&hash)))
}

// Password verification
pub fn verify_password(password: &String, salt_hex: &str, hash_hex: &str) -> bool {
    let salt = match HEXUPPER.decode(salt_hex.as_bytes()) {
        Ok(s) => s,
        Err(_) => return false,
    };

    let expected_hash = match HEXUPPER.decode(hash_hex.as_bytes()) {
        Ok(h) => h,
        Err(_) => return false,
    };

    pbkdf2::verify(
        pbkdf2::PBKDF2_HMAC_SHA512,
        ITERATIONS,
        &salt,
        password.as_bytes(),
        &expected_hash,
    ).is_ok()
}
#[post("/register", data = "<input>")]
pub fn register(input: Form<AuthInput>) -> Json<AuthResponse> {
    let mut users = load_users();

    if users.iter().any(|u| u.username == input.username) {
        return Json(AuthResponse {
            status: false,
            message: "Username already exists".into(),
        });
    }

    let (salt, hash) = match hash_password(&input.password) {
        Ok(res) => res,
        Err(_) => {
            return Json(AuthResponse {
                status: false,
                message: "Failed to hash password".into(),
            })
        }
    };

    users.push(User {
        username: input.username.clone(),
        salt,
        password_hash: hash,
    });

    save_users(&users);

    Json(AuthResponse {
        status: true,
        message: "User registered successfully".into(),
    })
}

#[post("/login", data = "<input>")]
pub fn login(input: Form<AuthInput>) -> Json<AuthResponse> {
    let users = load_users();

    if let Some(user) = users.iter().find(|u| u.username == input.username) {
        if verify_password(&input.password, &user.salt, &user.password_hash) {
            return Json(AuthResponse {
                status: true,
                message: "Login successful".into(),
            });
        } else {
            return Json(AuthResponse {
                status: false,
                message: "Invalid password".into(),
            });
        }
    }

    Json(AuthResponse {
        status: false,
        message: "User not found".into(),
    })
}