use super::argparser::ArgParser;
use crate::utils::log;
use crate::utils::restore_comp::{backup_sekai, can_save, restore_sekai};
use std::path::PathBuf;

pub const HELP_TEXT: &str = r#"
Usage: save

Saves the progress of your sekai progress. This means all your progress is saved till and you can restore it later.
- save                       : Save your current progress of the Sekai.
- save -r | --restore        : Restore the Sekai directory from the last saved state. 
"#;

pub fn save(args: &[&str], root_path: &PathBuf) -> String {
    let mut parser = ArgParser::new(&["-r", "--restore"]);
    let args_string: Vec<String> = args.iter().map(|s| s.to_string()).collect();

    let mut err_msg: String = "save: ".to_string();

    match parser.parse(&args_string) {
        Ok(_) => {
            let pos_args = parser.get_positional_args();
            if !pos_args.is_empty() {
                err_msg += "Too many positional arguments provided. No arguments expected.";
                log::log_info("restore", err_msg.as_str());
                return err_msg;
            }
            if parser.has_flag("--restore") || parser.has_flag("-r") {
                log::log_info("save", "RESTORE PARSED");
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
            } else {
                // Save operation
                if backup_sekai("save", root_path).is_err() {
                    err_msg += "Failed to save Sekai. Please check the logs for more details.";
                    log::log_error("save", err_msg.as_str());
                    return err_msg;
                }
                "Sekai saved successfully from previously saved progress.\n".to_string()
            }
        }
        Err(e) => match &e[..] {
            "help" => HELP_TEXT.to_string(),
            "unknown" => "restore: unknown flag\nTry 'help save' for more information.".to_string(),
            _ => "Error parsing arguments. Try 'help save' for more information.".to_string(),
        },
    }
}
