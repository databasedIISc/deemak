
use super::argparser::ArgParser;
use super::cmds::normalize_path;
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
pub const HELP_TEXT: &str = r#"
Usage: dev [OPTIONS_1] [OPTIONS_2] <PATH TO OBJECT> 

use dev mode wth the below options to change set levels and chest as locked or unlocked and manage the solutions and flags to them
Option_1:
    -l, --lock       Lock a object or reset its key(s):{solution, flag}
    -u, --unlock     Unlock an object
    -s, --set
Option_2:
    -c, --chest      Lock/unlock a chest
    -lv, --level      Lock/unlock a level

Examples:
"#;

pub static level_order:Vec<String>=vec![room_2,room_3,room_4,room_5,room_6,room_7,room_8,room_9,room_10];

pub fn dev_remove_level_status(path_to_obj:&Path,path_to_next_level:Option<&Path>)->Resukt<(),String>{
    //validate path_to_obj, get info_path
    //check if the object is a level
    //check if the object is locked at all
    //remove decrypt_me and compare_me
    //set locked to "00"
}
pub fn create_level_lock(
    solve_from_path: &Path,
    set_lock_to_path: &Path,
    current_dir:&Path,
    solution: String,
    flag: String,
) -> Result<(), String> {
    //validate solve_from_path, set_lock_to_path
    let path_1 = normalize_path(&current_dir.join(solve_from_path));
            if !path_1.exists() {
                err_msg += format!("Invalid `solve_from_path`:{} given",solve_from_path);
                log::log_info("solve", err_msg.as_str());
                return Err(err_msg);
            }
    let path_2=norm_path(&current_dir.join(set_lock_to_path));
            if !path_2.exists() {
                err_msg += format!("Invalid `set_lock_to_path`:{} path given",set_lock_to_path);
                log::log_info("solve", err_msg.as_str());
                return Err(err_msg);
            }
    
    //check if solve_from_path  and set_lock_to_path is a level since only a level allows command `solve`
    let lock_perm_path_1=read_lock_perm(path_1);
    if lock_perm_path_1.is_err(){return formt!("unable to read `locked` for `{}`",solve_from_path);}
    let (path_1_is_level,path_1_is_locked)=lock_perm_path_1.unwrap();

    let lock_perm_path_2=read_lock_perm(path2);
    if lock_perm_path_2.is_err(){return formt!("unable to read `locked` for `{}`",set_lock_to_path);}
    let (path_2_is_level,path_2_is_locked)=lock_perm_path_2.unwrap();

    if !(path_2_is_level && path_1_is_level){err_msg+="One or both the paths provided are not levels.";
        return Err(err_msg);}
    //conclude:both are levels
    let Ok(level_1_name) = path_1
                .file_name()
                .and_then(|s| s.to_str())
                .ok_or("Invalid level name")
            else {
                err_msg += "Failed to get level name from path.";
                log::log_error("solve", err_msg.as_str());
                return err_msg;
            };

    let decrypt_me_paht_1=encrypt(&characterise_enc_key(level_name,solution),&flag);
    // comare_me_path_2=?
    object_salt_2=read_get_obj_info(info_path: &Path, obj_name: &str) 
    hash_1=argonhash()


}


pub fn dev_lock_chest(args: &[&str],
    current_dir: &Path,
    root_dir: &Path,
    prompter_for_answer: &mut dyn UserPrompter,prompter_confirm:&mut dyn Prompter)->Result<(),String>{
        //check if path valid and get info path
        //check if chest
        //check if unlocked
        //if locked still ask to lock with new answer

    }
    pub fn dev_unlock_chest(args: &[&str],
    current_dir: &Path,
    root_dir: &Path)->Result<(),String>{
        //check if path valid and get info path
        //check if is chest
        //check if is locked
        //set lock perm to "00"
        //delete decrypt_me


}