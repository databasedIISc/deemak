use argon2::password_hash::rand_core::le;
use serde::de;

use super::argparser::ArgParser;
use super::cmds::{check_dir_info, normalize_path};
use super::display_relative_path;
use crate::metainfo::info_reader::{add_obj_to_info,read_get_obj_info,get_encrypted_flag,get_level_name};
use crate::metainfo::lock_perm::read_lock_perm;
use crate::metainfo::valid_sekai::create_dir_info;
use crate::rns::security::{characterise_enc_key, decrypt, encrypt};
use crate::utils::log;
use std::path::{self, Path, PathBuf};
use crate::utils::globals::USER_ID;
use crate::commands::go::navigate;
pub const HELP_TEXT: &str = r#"
Usage: solve [OPTIONS] <LEVEL_NAME> <

Use Solve to enter your answer to a problem 
Options:

Examples:
"#;

pub fn solve(args: &[&str], current_dir: &PathBuf, root_dir: &Path) -> String {
    //only 1 argumen :path to level
    let mut parser = ArgParser::new(&[]);
    let args_string: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    let mut err_msg: String = "solve: ".to_string();
    log::log_debug(
        "solve",
        &format!(
            "Parsing arguments: {:?}, Current Directory: {}",
            args_string,
            current_dir.display(),
        ),
    );
    match parser.parse(&args_string, "solve") {
        Ok(_) => {
            let pos_args = parser.get_positional_args();
            if !pos_args.len() == 1 {
                err_msg += "Too many positional arguments provided. Only 1 argument expected.";
                log::log_info("solve", err_msg.as_str());
                return err_msg;
            }
            //now we know only 1 argument is there
            //test for valid level name
            let (mut target,_)=navigate(pos_args[0].as_str(), current_dir, root_dir);
            target = normalize_path(&target);
            if !target.exists() {
                err_msg += "Invalid path given";
                log::log_info("solve", err_msg.as_str());
                return err_msg;
            }
            //validated path. now check if it is a protected thing
            if let Ok((is_locked, is_level)) = read_lock_perm(&target) {
                if !is_level {
                    err_msg += "This is not a level. Cannot solve.";
                    log::log_info("solve", err_msg.as_str());
                    return err_msg;
                }
                if is_locked {
                    err_msg += "Level is locked. Cannot solve.";
                    log::log_info("solve", err_msg.as_str());
                    return err_msg;
                }
            } else {
                err_msg += "Failed to read lock permissions for the level.";
                log::log_error("solve", err_msg.as_str());
                return err_msg;
            }
           
            if let Ok(level_name) = get_level_name(&target) {
                log::log_info("solve", &format!("Level name: {}", level_name));
                if let Ok(decrypt_me) = get_encrypted_flag(&target, &level_name) {
                    log::log_info("solve", &format!("Encrypted flag: {}", decrypt_me));
                    let input="krishna".to_string();
                    log::log_info("solve", &format!("Input received: {}", input));
                    let flag= check_solve_input(input,&target,level_name);
                    log::log_info("solve", &format!("Flag obtained: {}", flag));
                    return err_msg;
                } else {
                    err_msg += "Failed to get encrypted flag for the level.";
                    log::log_error("solve", err_msg.as_str());
                    return err_msg;
                }
            } else {
                err_msg += "Failed to get level name.";
                log::log_error("solve", err_msg.as_str());
                return err_msg;
            }
        }
        Err(e) => match &e[..] {
            "help" => HELP_TEXT.to_string(),
            _ => "Error parsing arguments. Try 'help solve' for more information.".to_string(),
        },
    }
    //check solution
    //return level "<levelname>" else "try again"
    
}

fn check_solve_input(user_input: String,path_to_level:&PathBuf,level_name:&str) -> String {    
    
    let text_decrypt_me = get_encrypted_flag(path_to_level,level_name)
        .expect("Failed to get encrypted flag");
    let user_inp_enc_key = characterise_enc_key(&USER_ID.get().unwrap(), level_name);
    let decrypted_user_input = decrypt(&user_inp_enc_key, &user_input);
    //run some extra tests on decrypted user input 
    //use this to decrypt textfile
    let decrypted_decrypt_me = decrypt(
        &characterise_enc_key(level_name, &decrypted_user_input),
        &text_decrypt_me,
    );
    let user_flag: String = encrypt(
        &characterise_enc_key(
            &format!(
                "{}_{}",
                USER_ID.get().unwrap(),
                USER_ID.get().unwrap().len()
            ),
            &format!("{}_{}", USER_ID.get().unwrap(), level_name),
        ),
        &decrypted_decrypt_me,
    );
    user_flag
}
