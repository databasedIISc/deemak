use super::argparser::ArgParser;
use crate::rns::restore_comp::{backup_sekai, can_restore, can_save, restore_sekai};
use crate::utils::log;
use crate::utils::prompt::UserPrompter;
use std::path::PathBuf;

pub const HELP_TEXT: &str = r#"
Usage: restore

- restore                     : Restore the Sekai directory from the last saved state. 
- restore -f | --force        : This means all your progress is gone, and you will have to start over from scratch.
"#;

pub fn restore(args: &[&str], root_path: &PathBuf, prompter: &mut dyn UserPrompter) -> String {
    let mut parser = ArgParser::new(&["-f", "--force"]);
    let args_string: Vec<String> = args.iter().map(|s| s.to_string()).collect();

    let mut err_msg: String = "restore: ".to_string();
    match parser.parse(&args_string, "restore") {
        Ok(_) => {
            let pos_args = parser.get_positional_args();
            if !pos_args.is_empty() {
                err_msg += "Too many positional arguments provided. No arguments expected.";
                log::log_info("restore", err_msg.as_str());
                return err_msg;
            }
            // Ask for confirmation

            if parser.has_flag("--force") || parser.has_flag("-f") {
                if !prompter
                    .confirm("Are you sure you want to restore? This will erase all progress.")
                {
                    return "Restore cancelled by user.".to_string();
                }
                if can_restore(root_path) {
                    // Restore file already exists.
                    log::log_info(
                        "restore",
                        "Restore file found. Proceeding with restoration.",
                    );
                    if restore_sekai("restore", root_path).is_err() {
                        err_msg +=
                            "Failed to restore Sekai. Please check the logs for more details.";
                        log::log_error("restore", err_msg.as_str());
                        return err_msg;
                    }
                    "Sekai restored successfully.\n".to_string()
                } else {
                    err_msg += "No restore file found. ";
                    // If restore file is not found, backup the current state
                    log::log_info("restore", err_msg.as_str());
                    if backup_sekai("restore", root_path).is_err() {
                        err_msg += "Failed to backup current state. Please check the logs for more details.";
                        log::log_error("restore", err_msg.as_str());
                        return err_msg;
                    }
                    err_msg + "Backup created successfully."
                }
            } else {
                if !prompter.confirm("Are you sure you want to restore to the last saved version?")
                {
                    return "Restore cancelled by user.".to_string();
                }
                log::log_info("restore", "SAVE  PARSED");
                // Restore operation
                if can_save(root_path) {
                    // `save_me` file already exists.
                    log::log_info("save", "Saved file found. Proceeding with restoration.");
                    if restore_sekai("save", root_path).is_err() {
                        err_msg +=
                            "Failed to restore Sekai. Please check the logs for more details.";
                        log::log_error("save", err_msg.as_str());
                        return err_msg;
                    }
                    "Sekai restored successfully from previously saved progress.\n".to_string()
                } else {
                    (err_msg + "No restore file found. Please save your progress first with `save` command.")
                        .to_string()
                }
            }
        }
        Err(e) => match &e[..] {
            "help" => HELP_TEXT.to_string(),
            "unknown" => "restore: unknown flag\nTry 'help save' for more information.".to_string(),
            _ => "Error parsing arguments. Try 'help restore' for more information.".to_string(),
        },
    }
}
