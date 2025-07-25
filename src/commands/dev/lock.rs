use super::super::argparser::ArgParser;
use super::super::cmds::normalize_path;
use crate::metainfo::info_reader::{
    create_compare_me_in_info, del_compare_me_from_info, del_decrypt_me_from_info,
    read_get_obj_info, update_obj_status,
};
use crate::metainfo::read_lock_perm;
use crate::rns::security::{argonhash, characterise_enc_key, decrypt, encrypt};
use crate::utils::{auth::get_current_user, log, prompt::UserPrompter};
use argon2::password_hash::SaltString;
use std::path::Path;

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
                        Remove level status from a level and delete decrypt_me and compare_me files if they exist

Examples:
"#;
pub fn dev_lock(
    args: &[&str],
    current_dir: &Path,
    root_dir: &Path,
    prompter: &mut dyn UserPrompter,
) -> Result<String, String> {
    // Parse arguments
    let mut parser = ArgParser::new(&["-t", "--type", "-s", "--status", "-ll", "--level-lock"]);
    let args_string: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    let mut err_msg: String = "dev lock: ".to_string();
    log::log_debug("dev_lock", &format!("args: {:?}", &args_string));

    //match args[0]{
    match args[0] {
        "-t" | "--type" => {
            // Check if type is provided
            if args.len() < 2 {
                err_msg += "Type not provided. Expected -l for level or -c for chest.";
                log::log_info("dev_lock", err_msg.as_str());
                return Err(err_msg);
            }

            match args[1] {
                "-l" | "--level" => dev_make_level(args[2], current_dir, root_dir),
                "-c" | "--chest" => dev_make_chest(args[2], current_dir, root_dir),
                _ => {
                    err_msg += "Invalid type provided. Expected -l for level or -c for chest.";
                    log::log_info("dev_lock", err_msg.as_str());
                    Err(err_msg)
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
                    //prompt for solution

                    let solution = prompter.input("> Enter your solution for the lock : ");
                    dev_lock_chest(args[2], &solution, current_dir, root_dir)
                }
                "-u" | "--unlock" => dev_unlock_chest(args[2], current_dir, root_dir),
                _ => {
                    err_msg += "Invalid status provided. Expected -l for lock or -u for unlock.";
                    log::log_info("dev_lock", err_msg.as_str());
                    Err(err_msg)
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
            let solution = prompter.input("> Enter your solution for the level lock : ");
            let flag = prompter.input("> Enter your flag for the lock : ");
            dev_create_level_lock(
                &solve_from_path,
                &set_lock_to_path,
                current_dir,
                &solution,
                &flag,
            )
        }
        "-rm" | "--rm-level-lock" => {
            if args.len() < 2 {
                err_msg += "Not enough arguments for removing level lock. Expected <path_to_level>";
                log::log_info("dev_lock", err_msg.as_str());
                return Err(err_msg);
            }
            let path_to_level = normalize_path(&current_dir.join(args[1]));
            let path_to_next_level = normalize_path(&current_dir.join(args[2]));
            dev_remove_level_lock(&path_to_level, current_dir, root_dir)
        }
        _ => Err(format!(
            "Invalid option: {}. Use -t, --type, -s, --status, -ll, --level-lock or -rm, --rm-level-lock",
            args[0]
        )),
    }
}
fn dev_make_level(
    path_to_obj: &str,
    current_dir: &Path,
    root_dir: &Path,
) -> Result<String, String> {
    //if successfule it makes the object an unlocked level by default
    let path = normalize_path(&current_dir.join(path_to_obj));
    let obj_name = path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or("Invalid object name")?;
    if !path.exists() {
        return Err(format!("Path does not exist: {path_to_obj}"));
    }
    let info_path = path.parent().unwrap().join(".dir_info/info.json");
    if !info_path.exists() {
        return Err(format!(
            "Info file does not exist at: {}",
            info_path.display()
        ));
    }
    //read_lock_perm
    let lock_perm = read_lock_perm(&path);
    if lock_perm.is_err() {
        return Err(format!(
            "Failed to read lock permissions for the level: {}",
            path.display()
        ));
    }
    let (is_level, is_locked) = lock_perm.unwrap();
    if is_level {
        return Err(format!("Object at {} is already a level.", path.display()));
    }

    let attempt = update_obj_status(
        &path,
        obj_name,
        "locked",
        serde_json::Value::String("10".into()),
    );
    if attempt.is_err() {
        return Err(format!(
            "Failed to update lock permissions for the level: {}",
            path.display()
        ));
    }
    Ok("Level created successfully".into())
}

fn dev_make_chest(
    path_to_obj: &str,
    current_dir: &Path,
    root_dir: &Path,
) -> Result<String, String> {
    //only an unlocked level can be converted to a chest
    let path = normalize_path(&current_dir.join(path_to_obj));
    let obj_name = path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or("Invalid object name")?;
    if !path.exists() {
        return Err(format!("Path does not exist: {path_to_obj}"));
    }
    let info_path = path.parent().unwrap().join(".dir_info/info.json");
    if !info_path.exists() {
        return Err(format!(
            "Info file does not exist at: {}",
            info_path.display()
        ));
    }
    //read_lock_perm
    let lock_perm = read_lock_perm(&path);
    if lock_perm.is_err() {
        return Err(format!(
            "Failed to read lock permissions for the level: {}",
            path.display()
        ));
    }
    let (is_level, is_locked) = lock_perm.unwrap();
    if !is_level {
        Ok("Object is already a chest.".to_string())
    } else {
        //is level
        //remove decrypt_me and compare_me from object info  if they exist
        let attempt1 = del_compare_me_from_info(&path, obj_name);
        let attempt2 = del_decrypt_me_from_info(&path, obj_name);
        if attempt1.is_err() || attempt2.is_err() {
            return Err(format!(
                "Failed to remove compare_me or decrypt_me from info for the level: {}",
                path.display()
            ));
        }
        //set lock perm to "00"
        let attempt = update_obj_status(
            &path,
            obj_name,
            "locked",
            serde_json::Value::String("00".into()),
        );
        if attempt.is_err() {
            return Err(format!(
                "Failed to update lock permissions for the level: {}",
                path.display()
            ));
        }
        //return success message
        Ok("Chest created from level successfully".into())
    }
}

fn dev_unlock_chest(
    path_to_obj: &str,
    current_dir: &Path,
    root_dir: &Path,
) -> Result<String, String> {
    //check if path valid and get info path
    let path = normalize_path(&current_dir.join(path_to_obj));
    let obj_name = path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or("Invalid object name")?;
    if !path.exists() {
        return Err(format!("Path does not exist: {path_to_obj}"));
    }
    let info_path = path.parent().unwrap().join(".dir_info/info.json");
    if !info_path.exists() {
        return Err(format!(
            "Info file does not exist at: {}",
            info_path.display()
        ));
    }
    //check if is chest
    let lock_perm = read_lock_perm(&path);
    if lock_perm.is_err() {
        return Err(format!(
            "Failed to read lock permissions for the chest: {}",
            path.display()
        ));
    }
    let (is_level, is_locked) = lock_perm.unwrap();
    if is_level {
        return Err(format!(
            "Object {} is a level, not a chest.",
            path.display()
        ));
    }
    //check if is locked
    if !is_locked {
        return Ok(format!("Chest {} is already unlocked.", path.display()));
    }

    //remove compare_me
    let attempt1 = del_compare_me_from_info(&path, obj_name);
    if attempt1.is_err() {
        return Err(format!(
            "Failed to remove compare_me from info for the chest: {}",
            path.display()
        ));
    }
    //set lock perm to "00"
    let attempt = update_obj_status(
        &path,
        obj_name,
        "locked",
        serde_json::Value::String("00".into()),
    );
    if attempt.is_err() {
        return Err(format!(
            "Failed to update lock permissions for the chest: {}",
            path.display()
        ));
    }
    Ok(format!("Chest {} unlocked successfully.", path.display()))
}

fn dev_lock_chest(
    path_to_obj: &str,
    solution: &str,
    current_dir: &Path,
    root_dir: &Path,
) -> Result<String, String> {
    //check if path valid and get info path
    let path = normalize_path(&current_dir.join(path_to_obj));
    let obj_name = path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or("Invalid object name")?;
    if !path.exists() {
        return Err(format!("Path does not exist: {path_to_obj}"));
    }
    let info_path = path.parent().unwrap().join(".dir_info/info.json");
    if !info_path.exists() {
        return Err(format!(
            "Info file does not exist at: {}",
            info_path.display()
        ));
    }
    //check if is chest
    let lock_perm = read_lock_perm(&path);
    if lock_perm.is_err() {
        return Err(format!(
            "Failed to read lock permissions for the chest: {}",
            path.display()
        ));
    }
    let (is_level, is_locked) = lock_perm.unwrap();
    if is_level {
        return Err(format!(
            "Object {} is a level, not a chest.",
            path.display()
        ));
    }
    //check if is locked

    //create compare_me
    let attempt1 = create_compare_me_in_info(&path, obj_name, solution.to_string());
    if attempt1.is_err() {
        return Err(format!(
            "Failed to remove compare_me from info for the chest: {}",
            path.display()
        ));
    }
    //set lock perm to "00"
    let attempt = update_obj_status(
        &path,
        obj_name,
        "locked",
        serde_json::Value::String("01".into()),
    );
    if attempt.is_err() {
        return Err(format!(
            "Failed to update lock permissions for the chest: {}",
            path.display()
        ));
    }
    Ok(format!("Chest {} unlocked successfully.", path.display()))
}

pub fn dev_create_level_lock(
    solve_from_path: &Path,
    set_lock_to_path: &Path,
    current_dir: &Path,
    solution: &str,
    flag: &str,
) -> Result<String, String> {
    //validate solve_from_path, set_lock_to_path
    let mut err_msg = String::new();
    let path_1 = normalize_path(&current_dir.join(solve_from_path));
    if !path_1.exists() {
        err_msg += &format!("Invalid `solve_from_path`:{solve_from_path:?} given");
        log::log_info("solve", err_msg.as_str());
        return Err(err_msg);
    }
    let path_2 = normalize_path(&current_dir.join(set_lock_to_path));
    if !path_2.exists() {
        err_msg += &format!("Invalid `set_lock_to_path`:{set_lock_to_path:?} path given");
        log::log_info("solve", err_msg.as_str());
        return Err(err_msg);
    }

    //check if solve_from_path  and set_lock_to_path is a level since only a level allows command `solve`
    let lock_perm_path_1 = read_lock_perm(&path_1);
    if lock_perm_path_1.is_err() {
        return Ok(format!("unable to read `locked` for `{solve_from_path:?}`"));
    }
    let (path_1_is_level, path_1_is_locked) = lock_perm_path_1.unwrap();

    let lock_perm_path_2 = read_lock_perm(&path_2);
    if lock_perm_path_2.is_err() {
        return Ok(format!(
            "unable to read `locked` for `{set_lock_to_path:?}`"
        ));
    }
    let (path_2_is_level, path_2_is_locked) = lock_perm_path_2.unwrap();

    if !(path_2_is_level && path_1_is_level) {
        err_msg += "One or both the paths provided are not levels.";
        return Err(err_msg);
    }
    //conclude:both are levels
    let Ok(level_1_name) = path_1
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or("Invalid level name")
    else {
        err_msg += "Failed to get level name from path.";
        log::log_error("solve", err_msg.as_str());
        return Err(err_msg);
    };
    let Ok(level_2_name) = path_2
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or("Invalid level name")
    else {
        err_msg += "Failed to get level name from path.";
        log::log_error("solve", err_msg.as_str());
        return Ok(err_msg);
    };
    let level_2_info=//read salt from info.json
        read_get_obj_info(&path_2, level_2_name);
    if level_2_info.is_err() {
        err_msg += &format!("Failed to read salt for level 2: {level_2_name}");
        log::log_error("solve", err_msg.as_str());
        return Err(err_msg);
    }
    let level_2_info = level_2_info.unwrap();
    let level_2_salt = level_2_info.properties.get("obj_salt");

    if level_2_salt.is_none() {
        err_msg += &format!("Salt not found for level 2: {level_2_name}");
        log::log_error("solve", err_msg.as_str());
        return Err(err_msg);
    }
    let level_2_salt = level_2_salt.unwrap().as_str().unwrap();
    let level_2_salt = SaltString::from_b64(level_2_salt);
    if level_2_salt.is_err() {
        err_msg += "Failed to convert level 2 salt from base64.";
        log::log_error("solve", err_msg.as_str());
        return Err(err_msg);
    }
    let level_2_salt = level_2_salt.unwrap();

    //create decrypt_me and compare_me

    let decrypt_me_path_1 = encrypt(&characterise_enc_key(level_1_name, solution), flag);
    // compare_me_path_2=?
    let user_info = match get_current_user() {
        Some(info) => info,
        None => {
            err_msg += "User not authenticated. Please log in.";
            log::log_error("unlock", err_msg.as_str());
            return Err(err_msg);
        }
    };
    let username = &user_info.username;
    let user_salt_hex = &user_info.salt;
    let user_salt = SaltString::from_b64(user_salt_hex);
    let user_salt = match user_salt {
        Ok(salt) => salt,
        Err(_) => {
            err_msg += "Failed to convert user salt from base64.";
            log::log_error("unlock", err_msg.as_str());
            return Err(err_msg);
        }
    };
    let decrypted_user_flag = decrypt(
        &characterise_enc_key(
            &format!("{}_{}", username, username.len()),
            &format!("{username}_{level_1_name}"),
        ),
        flag,
    );
    let l1_hashed_user_flag = argonhash(&level_2_salt, decrypted_user_flag);
    let hashed_with_usersalt = argonhash(&user_salt, l1_hashed_user_flag);
    let compare_me_path_2 = encrypt(
        &characterise_enc_key(level_2_salt.as_str(), level_2_name),
        &hashed_with_usersalt,
    );
    //write compare_me and decrypt_me to info.json along with level permissions
    let attempt1 = update_obj_status(
        &path_2,
        level_2_name,
        "locked",
        serde_json::Value::String("11".into()),
    );
    let attempt2 = update_obj_status(
        &path_1,
        level_1_name,
        "decrypt_me",
        serde_json::Value::String(decrypt_me_path_1),
    );
    let attempt3 = update_obj_status(
        &path_2,
        level_2_name,
        "compare_me",
        serde_json::Value::String(compare_me_path_2),
    );
    if attempt1.is_err() || attempt2.is_err() || attempt3.is_err() {
        err_msg += &format!(
            "Failed to update lock info at the desired locations for the lock from: {} to :{}",
            path_1.display(),
            path_2.display()
        );
        log::log_error("solve", err_msg.as_str());
        return Err(err_msg);
    }
    Ok("created lock successfully".to_string())
}

pub fn dev_remove_level_lock(
    path_to_level: &Path,
    current_dir: &Path,
    root_dir: &Path,
) -> Result<String, String> {
    //create path and validate

    let path = normalize_path(&current_dir.join(path_to_level));
    if !path.exists() {
        return Err(format!("Path does not exist: {path_to_level:?}. try again"));
    }
    //info_path
    let info_path = path.parent().unwrap().join(".dir_info/info.json");
    if !info_path.exists() {
        return Err(format!(
            "Info file does not exist at: {}",
            info_path.display()
        ));
    }
    //erase decrypt_me and compare_me from info.json
    let obj_name = path.file_name().and_then(|s| s.to_str());
    if obj_name.is_none() {
        return Err("Invalid object name".to_string());
    }
    let obj_name = obj_name.unwrap();
    //read lock perm to validate if level
    let lock_perm = read_lock_perm(&path);
    if lock_perm.is_err() {
        return Err(format!(
            "Failed to read lock permissions for the level: {}",
            path.display()
        ));
    }
    let (is_level, is_locked) = lock_perm.unwrap();
    if !is_level {
        return Err(format!("Object {} is not a level.", path.display()));
    }
    let attempt1 = del_compare_me_from_info(&path, obj_name);
    let attempt2 = del_decrypt_me_from_info(&path, obj_name);
    if attempt1.is_err() || attempt2.is_err() {
        return Err(format!(
            "Failed to remove compare_me or decrypt_me from info for the level: {}",
            path.display()
        ));
    }
    Ok(format!(
        "Removed level lock from {} successfully.",
        path.display()
    ))
}
