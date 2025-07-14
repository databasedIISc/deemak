use super::argparser::ArgParser;
use super::cmds::normalize_path;
use crate::metainfo::info_reader::get_encrypted_flag;
use crate::metainfo::lock_perm::read_lock_perm;
use crate::rns::security::{characterise_enc_key, decrypt, encrypt};
use crate::utils::globals::USER_NAME;
use crate::utils::{log, prompt::UserPrompter};
use std::path::Path;
pub const HELP_TEXT: &str = r#"
Usage: solve [OPTIONS] <LEVEL_NAME> <

Use Solve to enter your answer to a problem 
Options:

Examples:
"#;

pub fn solve(
    args: &[&str],
    current_dir: &Path,
    root_dir: &Path,
    prompter: &mut dyn UserPrompter,
) -> String {
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
            let target = normalize_path(&current_dir.join(pos_args[0]));
            if !target.exists() {
                err_msg += "Invalid path given";
                log::log_info("solve", err_msg.as_str());
                return err_msg;
            }
            //validated path. now check if it is a protected thing
            if let Ok((is_level, is_locked)) = read_lock_perm(&target) {
                if !is_level {
                    err_msg += "This is not a level. Cannot solve.";
                    log::log_info("solve", err_msg.as_str());
                    return err_msg;
                }
                if !is_locked {
                    err_msg += "Level is unlocked. You have permission to access it.";
                    log::log_info("solve", err_msg.as_str());
                    return err_msg;
                }
            } else {
                err_msg += "Failed to read lock permissions for the level.";
                log::log_error("solve", err_msg.as_str());
                return err_msg;
            }

            let Ok(level_name) = target
                .file_name()
                .and_then(|s| s.to_str())
                .ok_or("Invalid level name")
            else {
                err_msg += "Failed to get level name from path.";
                log::log_error("solve", err_msg.as_str());
                return err_msg;
            };
            log::log_info("solve", &format!("Level name: {}", level_name));
            let user_input =
                prompter.input(&format!("Enter your answer for level '{}': ", level_name));
            if user_input.is_empty() {
                err_msg += "No input provided. Cannot solve.";
                log::log_info("solve", err_msg.as_str());
                err_msg
            } else {
                let user_flag = check_solve_input(user_input, &target, level_name);
                if user_flag.is_empty() {
                    err_msg += "User flag is empty. Cannot solve.";
                    log::log_warning("solve", err_msg.as_str());
                    err_msg
                } else {
                    log::log_info(
                        "solve",
                        &format!("Successfully generated User flag: {}", user_flag),
                    );
                    format!("Your flag is {}", user_flag)
                }
            }
        }
        Err(e) => match &e[..] {
            "help" => HELP_TEXT.to_string(),
            _ => "Error parsing arguments. Try 'help solve' for more information.".to_string(),
        },
    }
}

fn check_solve_input(user_input: String, path_to_level: &Path, level_name: &str) -> String {
    let text_decrypt_me =
        get_encrypted_flag(path_to_level, level_name).expect("Failed to get encrypted flag");
    let user_inp_enc_key = characterise_enc_key(USER_NAME.get().unwrap(), level_name);
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
                USER_NAME.get().unwrap(),
                USER_NAME.get().unwrap().len()
            ),
            &format!("{}_{}", USER_NAME.get().unwrap(), level_name),
        ),
        &decrypted_decrypt_me,
    );
    user_flag
}
