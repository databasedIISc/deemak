use argon2::password_hash::SaltString;

use super::argparser::ArgParser;
use super::cmds::{check_dir_info, normalize_path};
use super::display_relative_path;
use crate::metainfo::info_reader::{add_obj_to_info, read_get_obj_info, get_encrypted_flag};
use crate::metainfo::lock_perm::{operation_locked_perm};
use crate::metainfo::read_lock_perm;
use crate::metainfo::valid_sekai::create_dir_info;
use crate::rns::security::{argonhash, characterise_enc_key, decrypt, encrypt};
use crate::utils::globals::USER_ID;
use crate::utils::log;
use base64ct::{Base64Unpadded, Encoding};
use std::path::{Path, PathBuf};
pub const HELP_TXT: &str = r#"
Usage: unlock [OPTIONS] <LEVEL/CHEST_NAME>

after obtaining a flag for a level you can use this command to unlock the level by using the flag 
Options:
    -l, --level       Move instead of copy (cut/paste)
    -c, --chest       Unlock 
    
Examples:
- copy file.txt new_file.txt         # Copy file

"#;
pub fn unlock(args: &[&str], current_dir: &PathBuf, root_dir: &Path) -> String {
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
            let pos_args = parser.get_positional_args();
            if pos_args.len() != 1 {
                err_msg += "Exactly one positional argument -giving path to directory/file to be unlocked -is expected.";
                log::log_info("unlock", err_msg.as_str());
                return err_msg;}         
            //now we know only 1 argument is there
            //validate path existence
            let target = Path::new(pos_args[0]); 
            let target = normalize_path(&target);
            if !target.exists() {
                err_msg += "Invalid path given";
                log::log_info("unlock", err_msg.as_str());
                return err_msg;
            }
            //validated path. now check if it is accessible
            if let Err(msg)=operation_locked_perm(&target.parent().unwrap(), "unlock", "you cannot try to unlock a chest/level nested inside a locked directory/level"){
                err_msg+=msg.as_str();
                log::log_info("unlock", err_msg.as_str());
                return err_msg;
            }
            //now check if it is a protected thing
            if let Ok((is_level, is_locked)) = read_lock_perm(&target) {
                if !is_locked {
                    err_msg += "Object requested is not locked or already unlocked. Cannot unlock. you can try accessing it directly.";
                    log::log_info("unlock", err_msg.as_str());
                    return err_msg;
                }
                //since it protected and open for unlocking read level/chest id 
            
                //get id of level/chest
                let locked_file_name = target
                    .file_name()
                    .and_then(|s| s.to_str())
                    .ok_or_else(|| "Invalid object name".to_string());
                if locked_file_name.is_err() {
                    err_msg += &format!("Failed to get locked file name: {}", locked_file_name.err().unwrap());
                    log::log_error("unlock", err_msg.as_str());
                    return err_msg;
                }
                let locked_file_name = locked_file_name.unwrap();

                let locked_file_info=read_get_obj_info(&target.parent().unwrap().join("./.dir_info/info.json"), locked_file_name);
                if locked_file_info.is_err() {
                    err_msg += &format!("Failed to read info.json for the locked object: {}", locked_file_info.err().unwrap());
                    log::log_error("unlock", err_msg.as_str());
                    return err_msg;
                    }
                let locked_file_info = locked_file_info.unwrap();
                
                // take flag 
                //ask for flag------------FILLIN-------------
                let mut user_flag = String::from("user_flag_placeholder");
                if is_level{
                    //unlocking level
                    if check_flag(user_flag, current_dir, root_dir);
                    return err_msg;
                }
                else {
                    //is chest 
                    return err_msg;
                    
                }
            }
            else {
                err_msg += "Failed to read lock permissions for the object.";
                log::log_error("unlock", err_msg.as_str());
                return err_msg;}
            }
            Err(e) => {
                err_msg += &format!("Failed to parse arguments: {}", e);
                log::log_error("unlock", err_msg.as_str());
                return err_msg;
            }
        }

            
    //velidity of path
    //test for operation_lock_perm except last one file
    //if permitted then ask for flag
    //take flag and check
    //return message
    }

fn check_flag(user_flag: String,level_name:&str ,level_id:&str, current_dir: &PathBuf, root_dir: &PathBuf) -> bool {
    let level_name = level_name;
    let level_id = level_id;
    let COMPARE_ME = r#"RAyDHwYErR{/)-RG/)-Z+[Vz/YVx/)RqYxszyCZqY+y=<+FrKzMsMzrGuDrzsyJCykMm9Ry2373dahat7qsmrCZF\<{4vefk(e;7tYLlhqdq*&C;3"#;
    const LEVEL_SALT: &str = "b2pjZWRtb25rYW5kYXN0aGVyZQ";
    let level_salt: SaltString = SaltString::from_b64(LEVEL_SALT).expect("Invalid salt");
    const USER_SALT: &str = "b2pftre4b25rYW5kdutyfytdmjgdtfrserVyZQ";
    let user_salt: SaltString = SaltString::from_b64(USER_SALT).expect("Invalid salt");
    let decrypted_user_flag = decrypt(
        &characterise_enc_key(
            &format!(
                "{}_{}",
                USER_ID.get().unwrap(),
                USER_ID.get().unwrap().len()
            ),
            &format!("{}_{}", USER_ID.get().unwrap(), level_name),
        ),
        &user_flag,
    );
    let l1_hashed_user_flag = argonhash(&level_salt, decrypted_user_flag);
    let hashed_with_usersalt = argonhash(&user_salt, l1_hashed_user_flag);
    let compare_me_decrypted = decrypt(&characterise_enc_key(level_id, level_name), COMPARE_ME);
    if &compare_me_decrypted == &hashed_with_usersalt {
        "unlocked".to_string()
    } else {
        "try again ".to_string()
    }
}
