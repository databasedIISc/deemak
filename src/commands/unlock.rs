use super::argparser::ArgParser;
use super::cmds::normalize_path;
use crate::metainfo::info_reader::{read_get_obj_info, update_obj_status};
use crate::metainfo::lock_perm::operation_locked_perm;
use crate::metainfo::read_lock_perm;
use crate::rns::security::{argonhash, characterise_enc_key, decrypt, encrypt};
use crate::utils::{auth::get_current_user, log, prompt::UserPrompter};
use argon2::password_hash::SaltString;
use std::path::Path;

pub const HELP_TXT: &str = r#"
Usage: unlock [OPTIONS] <LEVEL/CHEST_NAME>

after obtaining a flag for a level you can use this command to unlock the level by using the flag 
Options:
    -l, --level       Move instead of copy (cut/paste)
    -c, --chest       Unlock 
    
Examples:
- copy file.txt new_file.txt         # Copy file

"#;
pub fn unlock(
    args: &[&str],
    current_dir: &Path,
    root_dir: &Path,
    prompter: &mut dyn UserPrompter,
) -> String {
    //one argument giving path to the chest/level to be unlocked
    let mut parser = ArgParser::new(&["-l", "--level", "-c", "--chest"]);
    let args_string: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    let mut err_msg: String = "unlock: ".to_string();
    log::log_debug(
        "unlock",
        &format!(
            "Parsing arguments: {:?}, Current Directory: {}",
            args_string,
            current_dir.display(),
        ),
    );
    match parser.parse(&args_string, "unlock") {
        Ok(_) => {
            let user_info = match get_current_user() {
                Some(info) => info,
                None => {
                    err_msg += "User not authenticated. Please log in.";
                    log::log_error("unlock", err_msg.as_str());
                    return err_msg;
                }
            };
            let username = &user_info.username;
            let user_salt_hex = &user_info.salt;

            let pos_args = parser.get_positional_args();
            if pos_args.len() != 1 {
                err_msg += "Exactly one positional argument -giving path to directory/file to be unlocked -is expected.";
                log::log_info("unlock", err_msg.as_str());
                return err_msg;
            }
            //now we know only 1 argument is there
            //validate path existence
            let target = normalize_path(&current_dir.join(pos_args[0]));
            if !target.exists() {
                err_msg += "Invalid path given";
                log::log_info("unlock", err_msg.as_str());
                return err_msg;
            }
            //validated path. now check if it is accessible
            if let Err(msg) = operation_locked_perm(
                target.parent().unwrap(),
                "unlock",
                "you cannot try to unlock a chest/level nested inside a locked directory/level",
            ) {
                err_msg += msg.as_str();
                log::log_info("unlock", err_msg.as_str());
                return err_msg;
            }
            //now check if it is a protected thing
            if let Ok((is_level, is_locked)) = read_lock_perm(&target) {
                if !is_locked {
                    err_msg += format!(
                        "{} is not locked, you can try accessing it directly.",
                        pos_args[0]
                    )
                    .as_str();
                    log::log_info("unlock", err_msg.as_str());
                    return err_msg;
                }
                //since it protected and open for unlocking read level/chest id

                //get id of level/chest
                let locked_obj_name = target
                    .file_name()
                    .and_then(|s| s.to_str())
                    .ok_or_else(|| "Invalid object name".to_string());
                if locked_obj_name.is_err() {
                    err_msg += &format!(
                        "Failed to get locked file name: {}",
                        locked_obj_name.err().unwrap()
                    );
                    log::log_error("unlock", err_msg.as_str());
                    return err_msg;
                }
                let locked_obj_name = locked_obj_name.unwrap();

                let locked_obj_info = read_get_obj_info(
                    &target.parent().unwrap().join("./.dir_info/info.json"),
                    locked_obj_name,
                );
                if locked_obj_info.is_err() {
                    err_msg += &format!(
                        "Failed to read info.json for the locked object: {}",
                        locked_obj_info.err().unwrap()
                    );
                    log::log_error("unlock", err_msg.as_str());
                    return err_msg;
                }
                let locked_obj_info = locked_obj_info.unwrap();

                //read obj_salt
                let obj_salt = &locked_obj_info.properties["obj_salt"]
                    .as_str()
                    .ok_or_else(|| "Invalid 'obj_salt' property in info.json".to_string());
                if obj_salt.is_err() {
                    err_msg += &format!(
                        "Failed to get level salt {}",
                        obj_salt.as_ref().err().unwrap()
                    );
                    log::log_error("unlock", err_msg.as_str());
                    return err_msg;
                }
                let obj_salt = obj_salt.as_ref().unwrap();

                //reads decrypt_me from info.json
                let compare_me = &locked_obj_info.properties["compare_me"]
                    .as_str()
                    .ok_or_else(|| "Invalid 'compare_me' property in info.json".to_string());
                if compare_me.is_err() {
                    err_msg +=
                        &format!("Failed to get compare_me for the level/chest: {locked_obj_name}");
                    log::log_error("unlock", err_msg.as_str());
                    return err_msg;
                }
                let compare_me = compare_me.as_ref().unwrap();

                // take flag
                let user_flag =
                    prompter.input(format!("Enter the flag for {locked_obj_name}:").as_str());

                if is_level {
                    let decrypt_me = &locked_obj_info.properties["decrypt_me"]
                        .as_str()
                        .ok_or_else(|| "Invalid 'decrypt_me' property in info.json".to_string());
                    if decrypt_me.is_err() {
                        err_msg += &format!(
                            "Failed to get encrypted flag for the level/chest: {locked_obj_name}"
                        );
                        log::log_error("unlock", err_msg.as_str());
                        return err_msg;
                    }
                    let decrypt_me = decrypt_me.as_ref().unwrap();
                    let (result, message) = check_level(
                        user_flag,
                        locked_obj_name,
                        obj_salt,
                        decrypt_me,
                        compare_me,
                        username,
                        user_salt_hex,
                    );
                    if result {
                        //change lock status in info.json
                        let update_attempt = update_obj_status(
                            &target,
                            locked_obj_name,
                            "locked",
                            serde_json::Value::String("10".to_string()),
                        );
                        if update_attempt.is_err() {
                            err_msg += &format!(
                                "Failed to update lock status for {locked_obj_name}: {}",
                                update_attempt.err().unwrap()
                            );
                            log::log_error("unlock", err_msg.as_str());
                            return err_msg;
                        }
                        log::log_info(
                            "unlock",
                            "changes regarding lock status of object made to info.json successfully",
                        );
                        format!("{} is unlocked", locked_obj_name)
                    } else {
                        //flag incorrect or faced some error
                        err_msg += message.as_str();

                        err_msg += "Invalid flag. Try again.";
                        log::log_info("unlock", err_msg.as_str());
                        err_msg
                    }
                } else {
                    //if chest

                    let (result, message) = check_chest(
                        user_flag,
                        locked_obj_name,
                        obj_salt,
                        compare_me,
                        user_salt_hex,
                    );
                    if result {
                        //update obj_info_lock_perm
                        let update_attempt = update_obj_status(
                            &target,
                            locked_obj_name,
                            "locked",
                            serde_json::Value::from("00".to_string()),
                        );
                        if update_attempt.is_err() {
                            err_msg += &format!(
                                "Failed to update lock status for {locked_obj_name}: {}",
                                update_attempt.err().unwrap()
                            );
                            log::log_error("unlock", err_msg.as_str());
                            return err_msg;
                        }
                        log::log_info(
                            "unlock",
                            "changes regarding lock status of object made to info.json successfully",
                        );

                        format!(" Chest {} is unlocked", locked_obj_name)
                    } else {
                        //flag incorrect or faced some error
                        err_msg += message.as_str();
                        err_msg += "Invalid flag. Try again.";
                        log::log_info("unlock", err_msg.as_str());
                        err_msg
                    }
                }
            } else {
                err_msg += "Unable to read lock status of the given target. Cannot unlock.";
                log::log_info("unlock", err_msg.as_str());
                err_msg
            }
        }
        Err(e) => match &e[..] {
            "help" => HELP_TXT.to_string(),
            _ => "Error parsing arguments. Try 'help unlock' for more information.".to_string(),
        },
    }
}

fn check_level(
    user_flag: String,
    level_name: &str,
    level_salt: &str,
    encrypted_flag: &str,
    compare_me: &str,
    username: &str,
    user_salt_hex: &str,
) -> (bool, String) {
    let obj_salt = SaltString::from_b64(level_salt).expect("Invalid obj_salt format");
    //read user salt from database using f

    let mut message = String::new();
    let obj_salt = SaltString::from_b64(level_salt);
    if obj_salt.is_err() {
        return (
            false,
            format!("Error in salt for chest {level_name}. Please contact your provider"),
        );
    }
    let obj_salt = obj_salt.unwrap();
    //read user salt from database using f
    let user_salt = SaltString::from_b64(user_salt_hex);
    if user_salt.is_err() {
        return (
            false,
            "Error in user salt. Please contact your provider".to_string(),
        );
    }
    let user_salt = user_salt.unwrap();
    let decrypted_user_flag = decrypt(
        &characterise_enc_key(
            &format!("{}_{}", username, username.len()),
            &format!("{username}_{level_name}"),
        ),
        &user_flag,
    );
    let l1_hashed_user_flag = argonhash(&obj_salt, decrypted_user_flag);
    let hashed_with_usersalt = argonhash(&user_salt, l1_hashed_user_flag);
    let compare_me_decrypted = decrypt(&characterise_enc_key(level_salt, level_name), compare_me);

    message = format!("Flag processed for level {level_name}.");
    log::log_info(
        "unlock: processing user flag",
        "successfully processed user flag",
    );
    (compare_me_decrypted == hashed_with_usersalt, message)
}
fn check_chest(
    user_flag: String,
    chest_name: &str,
    chest_salt: &str,
    encrypted_hashed_flag: &str,
    user_salt_hex: &str,
) -> (bool, String) {
    let obj_salt = SaltString::from_b64(chest_salt).expect("Invalid obj_salt format");
    //read user salt from database using f
    let user_salt = SaltString::from_b64(user_salt_hex).unwrap();
    let mut message = String::new();
    //read object salt from info.json
    let obj_salt = SaltString::from_b64(chest_salt);
    if obj_salt.is_err() {
        return (
            false,
            format!("Error in salt for chest {chest_name}. Please contact your provider"),
        );
    }
    let obj_salt = SaltString::from_b64(chest_salt).unwrap();
    let hashed_user_flag = argonhash(&obj_salt, user_flag);
    let encryped_hashed_user_flag = encrypt(
        &characterise_enc_key(chest_name, &hashed_user_flag),
        &hashed_user_flag,
    );
    message = format!("Flag processed for chest {chest_name}.");
    log::log_info(
        "unlock: processing user flag",
        "successfully processed user flag",
    );
    (encryped_hashed_user_flag == encrypted_hashed_flag, message)
}
