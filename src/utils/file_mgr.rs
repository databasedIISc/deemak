use super::{globals::set_sekai_dir, log};
use crate::metainfo::valid_sekai::validate_or_create_sekai;
use crate::rns::{passlock::check_dmk_magic, restore_comp::generate_temp_path};
use crate::{DEV_MODE, epr_log_error, fatal_error};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default)]
pub struct DeemakSekaiMgr {
    pub abs_path: PathBuf,
    pub password: Option<String>,
    pub is_directory: bool,
    pub valid_sekai: bool,
    pub temp_location: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SekaiOperation {
    Play,    // User can play the sekai
    Create,  // Dev will create a new sekai
    Restore, // Dev will restore a sekai from Deemak File
    Invalid, // Invalid operation
}

impl SekaiOperation {
    /// Check if the operation is present in the allowed operations.
    pub fn is_present(&self, set_opers: Vec<SekaiOperation>) -> bool {
        set_opers.iter().any(|op| op == self)
    }

    pub fn oper_to_string(&self) -> String {
        match self {
            SekaiOperation::Play => "Play".to_string(),
            SekaiOperation::Create => "Create".to_string(),
            SekaiOperation::Restore => "Restore".to_string(),
            SekaiOperation::Invalid => "Invalid".to_string(),
        }
    }

    pub fn log_err(&self) {
        let oper_str = self.oper_to_string();
        epr_log_error!(
            "SEKAI",
            "{} operation is not allowed. Criterion Failed.",
            oper_str
        );
    }
}

/// Input password, optionally confirming it if `confirm` is true.
pub fn input_file_password(confirm: bool) -> String {
    let mut pwd = dialoguer::Password::new().with_prompt("Enter password");
    if confirm {
        pwd = pwd.with_confirmation("Confirm password", "Passwords don't match!");
    }
    match pwd.interact() {
        Ok(pwd) => pwd,
        Err(e) => {
            fatal_error!("SEKAI", "Password input failed: {e}");
        }
    }
}

impl DeemakSekaiMgr {
    pub fn new(sekai_path: PathBuf, password: Option<String>) -> Self {
        DeemakSekaiMgr {
            abs_path: Self::get_absolute_path(&sekai_path),
            password,
            is_directory: sekai_path.is_dir(),
            valid_sekai: Self::check_validity(&sekai_path),
            temp_location: Self::get_absolute_path(&sekai_path),
        }
    }

    /// If sekai is a file, it will check if its a valid deemak file.
    /// If sekai is a directory, it will validate or create the sekai dir_info's
    pub fn check_validity(path: &Path) -> bool {
        let mut path = Self::get_absolute_path(&PathBuf::from(path));
        if path.is_file() {
            if path
                .extension()
                .is_none_or(|ext| ext.to_string_lossy() != "deemak")
            {
                path.set_extension("deemak");
            }
            // File extension should be `.deemak`
            check_dmk_magic(&path).unwrap_or(false)
        } else {
            let home_ok = validate_or_create_sekai(&path, true);
            set_sekai_dir(path.clone()); // This line HAS to be sandwiched between the two calls
            let rest_ok = validate_or_create_sekai(&path, false);
            home_ok && rest_ok
        }
    }

    /// Get the absolute path of a given path.
    pub fn get_absolute_path(path: &Path) -> PathBuf {
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir()
                .expect("Failed to get current directory")
                .join(path.file_name().unwrap_or_else(|| std::ffi::OsStr::new("")))
        }
    }

    /// Manage if the sekai_path is a file or a directory.
    /// 1. If it's a directory, you can create a new sekai or play the sekai only in dev mode.
    /// 2. If it's a file, then user can play the sekai or Restore command is allowed.
    pub fn oper_allowed(&mut self) -> Vec<SekaiOperation> {
        let dev_mode = *DEV_MODE.get().unwrap_or(&false);
        let mut allowed_opers: Vec<SekaiOperation> = Vec::new();
        if self.valid_sekai {
            if self.is_directory {
                // Point number 1
                allowed_opers.push(SekaiOperation::Create);
                if dev_mode {
                    allowed_opers.push(SekaiOperation::Play);
                }
            } else {
                // Point number 2
                allowed_opers.push(SekaiOperation::Restore);
                if !dev_mode {
                    self.temp_location = generate_temp_path("sekai_wd");
                    allowed_opers.push(SekaiOperation::Play);
                }
            }
        } else {
            allowed_opers.push(SekaiOperation::Invalid);
        }
        if SekaiOperation::Invalid.is_present(allowed_opers.clone()) {
            allowed_opers = vec![SekaiOperation::Invalid];
        }
        allowed_opers
    }

    /// Set Xattr password to the file
    /// Sets a random password if the provided password is not provided
    pub fn set_password(&mut self, get_password: bool) -> Result<(), String> {
        if get_password {
            self.password = input_file_password(true).into();
        }

        log::log_debug(
            "Xattr Set",
            &format!(
                "Setting xattr password for file: {}",
                self.abs_path.display()
            ),
        );
        if self.password.is_none() {
            // Generate a random password if not provided
            self.password = Some(format!("sekai_{}", rand::random::<u64>()));
        }

        xattr::set(
            &self.abs_path,
            "pass.deemak",
            self.password.as_ref().unwrap().as_bytes(),
        )
        .map_err(|e| format!("Failed to set xattr metadata: {e}"))?;
        Ok(())
    }

    /// Get Xattr password from the file
    pub fn get_password(&mut self) -> String {
        let password_bytes =
            xattr::get(&self.abs_path, "pass.deemak").expect("Failed to read xattr metadata");
        match password_bytes {
            Some(bytes) => String::from_utf8(bytes).expect("Password metadata is not valid UTF-8"),
            None => {
                // If no password is set, generate a new one
                log::log_warning("Xattr Set", "No password found, generating a random one.");
                let new_pass = format!("sekai_{}", rand::random::<u64>());
                let _ = self.set_password(false);
                new_pass
            }
        }
    }
}
