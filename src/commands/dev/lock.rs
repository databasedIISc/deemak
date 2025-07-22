use super::super::argparser::ArgParser;
use super::super::cmds::normalize_path;
use crate::metainfo::info_reader::{read_get_obj_info, update_obj_status, del_compare_me_from_info, del_decrypt_me_from_info};
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
Usage: dev lock [OPTIONS_1] [OPTIONS_2] <PATH TO OBJECT> or
       dev lock [OPTIONS_1] <PATH TO LEVEL> <PATH TO NEXT LEVEL> 
use dev mode wth the below options to change set levels and chest as locked or unlocked and manage the solutions and flags to them
Option_1:
    -t, --type          Change whether this is a level or chest
                        Option_2: -l, --level, -c, --chest
    -s, --status        (Un)/lock a chest
                        Option_2: -l, --lock, -u, --unlock
    -ll, --level_lock   Create a level lock with a solution and flag
    -rm, --rm_level_lock
                        Remove level status from an level and delete decrypt_me and compare_me files if they exist

Examples:
"#;
pub fn dev_lock(
    args: &[&str],
    current_dir: &Path,
    root_dir: &Path,
) -> Result<String, String> {
    // Parse arguments
    let mut parser = ArgParser::new(&["-t", "--type", "-s", "--status", "-ll", "--level-lock"]);
    let args_string: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    let mut err_msg: String = "dev lock: ".to_string();
    log::log_debug(
        "dev_lock",
        &format!("args: {:?}", &args_string)
    );

    //match args[0]{
    match args[0]{
        "-t" | "--type" => {
            // Check if type is provided
            if args.len() < 2 {
                err_msg += "Type not provided. Expected -l for level or -c for chest.";
                log::log_info("dev_lock", err_msg.as_str());
                return Err(err_msg);
            }
            
            match args[1] {
                "-l" | "--level" => {
                    return dev_make_level(args[2], current_dir, root_dir);
                },
                "-c" | "--chest" => {
                    return dev_make_chest(args[2], current_dir, root_dir);
                },
                _ => {
                    err_msg += "Invalid type provided. Expected -l for level or -c for chest.";
                    log::log_info("dev_lock", err_msg.as_str());
                    return Err(err_msg);
                }
            }
        }
        "-s" | "--status" => {
            if args.len() < 2 {
                err_msg += "Status not provided. Expected -l for lock or -u for unlock.";
                log::log_info("dev_lock", err_msg.as_str());
                return Err(err_msg);
            }
            match args[1] {
                "-l" | "--lock" => {
                    return dev_lock_chest(&args[2..], current_dir, root_dir);
                },
                "-u" | "--unlock" => {
                    return dev_unlock_chest(&args[2..], current_dir, root_dir);
                },
                _ => {
                    err_msg += "Invalid status provided. Expected -l for lock or -u for unlock.";
                    log::log_info("dev_lock", err_msg.as_str());
                    return Err(err_msg);
                }
            }

        }
        "-ll" | "--level-lock" => {
            if args.len() < 3 {
                err_msg += "Not enough arguments for level lock. Expected <solve_from_path> <set_lock_to_path> <solution> <flag>";
                log::log_info("dev_lock", err_msg.as_str());
                return Err(err_msg);
            }
            let solve_from_path = normalize_path(&current_dir.join(args[1]));
            let set_lock_to_path = normalize_path(&current_dir.join(args[2]));
            return dev_create_level_lock(
                &solve_from_path,
                &set_lock_to_path,
                current_dir,
            );
        }
        "-rm" | "--rm-level-lock" => {
            if args.len() < 2 {
                err_msg += "Not enough arguments for removing level lock. Expected <path_to_level>";
                log::log_info("dev_lock", err_msg.as_str());
                return Err(err_msg);
            }
            let path_to_level = normalize_path(&current_dir.join(args[1]));
            return dev_remove_level_lock(&path_to_level, current_dir, root_dir);
        }
        _=>{
            Err(format!("Invalid option: {}. Use -t, --type, -s, --status, -ll, --level-lock or -rm, --rm-level-lock", args[0]))
        }



    }
    
}
fn dev_make_level(path_to_obj: &str, current_dir: &Path, root_dir: &Path) -> Result<String, String> {
    //if successfule it makes the object an unlocked level by default
    let path = normalize_path(&current_dir.join(path_to_obj));
    let obj_name = path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or("Invalid object name")?;
    if !path.exists() {
        return Err(format!("Path does not exist: {}", path_to_obj));    
    }
    let info_path = path.parent().unwrap().join(".dir_info/info.json");
    if !info_path.exists() {
        return Err(format!("Info file does not exist at: {}", info_path.display()));
    }
    //read_lock_perm
    let lock_perm = read_lock_perm(&path);
    if lock_perm.is_err() {
        return Err(format!("Failed to read lock permissions for the level: {}", path.display()));
    }
    let (is_level, is_locked) = lock_perm.unwrap();
    if is_level {
        return Err(format!("Object at {} is already a level.", path.display()));
    }

    let attempt = update_obj_status(&path, obj_name, "locked", serde_json::Value::String("10".into()));
    if attempt.is_err() {
        return Err(format!("Failed to update lock permissions for the level: {}", path.display()));
    }
    Ok("Level created successfully".into())
}

fn dev_make_chest(path_to_obj: &str, current_dir: &Path, root_dir: &Path) -> Result<String, String> {
    //only an unlocked level can be converted to a chest
    let path = normalize_path(&current_dir.join(path_to_obj));
    let obj_name = path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or("Invalid object name")?;
    if !path.exists() {
        return Err(format!("Path does not exist: {}", path_to_obj));    
    }
    let info_path = path.parent().unwrap().join(".dir_info/info.json");
    if !info_path.exists() {
        return Err(format!("Info file does not exist at: {}", info_path.display()));
    }
    //read_lock_perm
    let lock_perm = read_lock_perm(&path);
    if lock_perm.is_err() {
        return Err(format!("Failed to read lock permissions for the level: {}", path.display()));
    }
    let (is_level, is_locked) = lock_perm.unwrap();
    if !is_level {
        return Ok("Object is already a chest.".to_string());
    }
    else{
        //is level
        //remove decrypt_me and compare_me from object info  if they exist
        let attempt1=del_compare_me_from_info(&path, obj_name);
        let attempt2=del_decrypt_me_from_info(&path, obj_name);
        if attempt1.is_err() || attempt2.is_err() {
            return Err(format!("Failed to remove compare_me or decrypt_me from info for the level: {}", path.display()));
        }
        //set lock perm to "00"
        let attempt = update_obj_status(&path, obj_name, "locked", serde_json::Value::String("00".into()));
        if attempt.is_err() {
            return Err(format!("Failed to update lock permissions for the level: {}", path.display()));
        }
        //return success message
        return Ok("Chest created from level successfully".into());
    }
}

fn dev_unlock_chest(
    path_to_obj: &str,
    current_dir: &Path,
    root_dir: &Path)->
    Result<String,String>{
        //check if path valid and get info path
    let path = normalize_path(&current_dir.join(path_to_obj));
    let obj_name = path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or("Invalid object name")?;
    if !path.exists() {
        return Err(format!("Path does not exist: {}", path_to_obj));    
    }
    let info_path = path.parent().unwrap().join(".dir_info/info.json");
    if !info_path.exists() {
        return Err(format!("Info file does not exist at: {}", info_path.display()));
    }
        //check if is chest
    let lock_perm = read_lock_perm(&path);
    if lock_perm.is_err() {
        return Err(format!("Failed to read lock permissions for the chest: {}", path.display()));
    }
    let (is_level, is_locked) = lock_perm.unwrap();
    if is_level {
        return Err(format!("Object {} is a level, not a chest.", path.display()));}
        //check if is locked
    if !is_locked {
        return Ok(format!("Chest {} is already unlocked.", path.display()));}
    
    //remove compare_me
    let attempt1 = del_compare_me_from_info(&path, obj_name);
    if attempt1.is_err() {
        return Err(format!("Failed to remove compare_me from info for the chest: {}", path.display()));
    }
        //set lock perm to "00"
    let attempt = update_obj_status(&path, obj_name, "locked", serde_json::Value::String("00".into()));
    if attempt.is_err() {
        return Err(format!("Failed to update lock permissions for the chest: {}", path.display()));
    }
       return Ok(format!("Chest {} unlocked successfully.", path.display()));
    }



fn dev_lock_chest(
    path_to_obj: &str,
    solution:&str,
    current_dir:&Path,
    root_dir:&Path)->
    Result<String,String>{
    //check if path valid and get info path
    let path = normalize_path(&current_dir.join(path_to_obj));
    let obj_name = path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or("Invalid object name")?;
    if !path.exists() {
        return Err(format!("Path does not exist: {}", path_to_obj));    
    }
    let info_path = path.parent().unwrap().join(".dir_info/info.json");
    if !info_path.exists() {
        return Err(format!("Info file does not exist at: {}", info_path.display()));
    }
        //check if is chest
    let lock_perm = read_lock_perm(&path);
    if lock_perm.is_err() {
        return Err(format!("Failed to read lock permissions for the chest: {}", path.display()));
    }
    let (is_level, is_locked) = lock_perm.unwrap();
    if is_level {
        return Err(format!("Object {} is a level, not a chest.", path.display()));}
        //check if is locked
    
    //create compare_me
    let attempt1 = create_compare_me_for_info(&path, obj_name,solution);
    if attempt1.is_err() {
        return Err(format!("Failed to remove compare_me from info for the chest: {}", path.display()));
    }
        //set lock perm to "00"
    let attempt = update_obj_status(&path, obj_name, "locked", serde_json::Value::String("01".into()));
    if attempt.is_err() {
        return Err(format!("Failed to update lock permissions for the chest: {}", path.display()));
    }
       return Ok(format!("Chest {} unlocked successfully.", path.display()));
    }




pub fn dev_create_level_lock(
    solve_from_path: &Path,
    set_lock_to_path: &Path,
    current_dir:&Path
) -> Result<String, String> {
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
    let Ok(level_2_name)=path_2
                .file_name()
                .and_then(|s| s.to_str())
                .ok_or("Invalid level name")
            else {
                err_msg += "Failed to get level name from path.";
                log::log_error("solve", err_msg.as_str());
                return err_msg;
            };

    let decrypt_me_paht_1=encrypt(&characterise_enc_key(level_1_name,solution),&flag);
    // comare_me_path_2=?
    info_path_2=path_1.
    level_2_info=read_get_obj_info(info_path:, obj_name: &str) 
    hash_1=argonhash()


}
