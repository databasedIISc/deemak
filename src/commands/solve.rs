use super::argparser::ArgParser;
use super::cmds::normalize_path;
use crate::metainfo::info_reader::read_get_obj_info;
use crate::metainfo::lock_perm::read_lock_perm;
use crate::rns::security::{characterise_enc_key, decrypt, encrypt};
use crate::utils::{auth::get_current_username, log, prompt::UserPrompter};
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
            if pos_args.len() > 1 {
                err_msg += "Too many positional arguments provided. Only 1 argument expected.";
                log::log_info("solve", err_msg.as_str());
                return err_msg;
            }
            if pos_args.is_empty() {
                err_msg += "No positional argument provided. Expected path to level.";
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
                if is_locked {
                    err_msg += "Level is locked. You must first unlock it.";
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
            log::log_info("solve", &format!("Level name: {level_name}"));
            let user_input =
                prompter.input(&format!("> Enter your answer for level '{level_name}': "));
            if user_input.is_empty() {
                err_msg += "No input provided. Cannot solve.";
                log::log_info("solve", err_msg.as_str());
                err_msg
            } else {
                let username = get_current_username().unwrap_or("default_user");
                let user_flag = check_solve_input(user_input, &target, level_name, username);
                match user_flag {
                    Ok(flag) => {
                        log::log_info(
                            "solve",
                            &format!("Successfully generated User flag: {flag}"),
                        );
                        format!("User flag: {flag}")
                    }
                    Err(e) => {
                        err_msg += &format!("Error solving {level_name}: {e}");
                        log::log_error("solve", err_msg.as_str());
                        err_msg
                    }
                }
            }
        }

        Err(e) => match &e[..] {
            "help" => HELP_TEXT.to_string(),
            _ => "Error parsing arguments. Try 'help solve' for more information.".to_string(),
        },
    }
}

fn check_solve_input(
    user_input: String,
    path_to_level: &Path,
    level_name: &str,
    username: &str,
) -> Result<String, String> {
    let info_path = path_to_level.parent().unwrap().join(".dir_info/info.json");
    println!("info_path: {}", info_path.display());
    println!("level_name: {level_name}");
    if let Some(text_decrypt_me) = read_get_obj_info(&info_path, level_name)
        .unwrap()
        .properties
        .get("decrypt_me")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
    {
        println!("text_decrypt_me: {text_decrypt_me}");
        println!("Username: {username}");
        let user_inp_enc_key = characterise_enc_key(username, level_name);
        let decrypted_user_input = decrypt(&user_inp_enc_key, &user_input);
        println!("decrypted _user input: {decrypted_user_input}");

        //run some extra tests on decrypted user input
        //use this to decrypt textfile
        let decrypted_decrypt_me = decrypt(
            &characterise_enc_key(level_name, &decrypted_user_input),
            &text_decrypt_me,
        );
        println!("decrypted _decrypt me : {decrypted_decrypt_me}");
        let user_flag: String = encrypt(
            &characterise_enc_key(
                &format!("{}_{}", username, username.len()),
                &format!("{username}_{level_name}"),
            ),
            &decrypted_decrypt_me,
        );
        Ok(user_flag)
    } else {
        //unable to read decrypt_me property
        println!("Unable to read decrypt_me property from info.json for level: {level_name}");
        Err("Unable to read decrypt_me property from info.json".to_string())
    }
}
