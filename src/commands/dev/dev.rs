use super::super::argparser::ArgParser;
use super::super::cmds::normalize_path;
use crate::metainfo::info_reader::{read_get_obj_info, update_obj_status};
use crate::metainfo::lock_perm::operation_locked_perm;
use crate::metainfo::read_lock_perm;
use crate::rns::security::{argonhash, characterise_enc_key, decrypt, encrypt};
use crate::utils::{
    //globals::{USER_NAME, USER_SALT},
    log,
    prompt::UserPrompter,
};
use argon2::password_hash::SaltString;
use rocket::http::tls::rustls::internal::msgs::message;
use serde_json::Value;
use std::path::Path;
use std::result;
use super::{lock, init_info};
pub fn dev(
    parts: &[&str],
    current_dir: &Path,
    root_dir: &Path,
    prompter: &mut dyn UserPrompter,
) -> String {
    if parts.is_empty() {
        return "Command not found".into();
    }

    match parts[0] {
        "lock" => {
            let msg = lock::dev_lock(&parts[1..], current_dir, root_dir,prompter);
            if msg.is_err() {
                return msg.err().unwrap();
            }
            return msg.unwrap();
        }
        "info" => {
            let msg = init_info::dev_info(&parts[1..], current_dir, root_dir);
            if msg.is_err() {   
                return msg.err().unwrap();
            }
            return msg.unwrap();
        }

        _ => {
            return "Invalid dev command".to_string()
        },
    }
}