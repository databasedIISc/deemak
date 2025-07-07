use super::argparser::ArgParser;
use crate::rns::restore_comp::backup_sekai;
use crate::utils::log;
use std::path::PathBuf;

pub const HELP_TEXT: &str = r#"
Usage: save

Saves the progress of your sekai progress. This means all your progress is saved till and you can restore it later.
- save  : Save your current progress of the Sekai.
"#;

pub fn save(args: &[&str], root_path: &PathBuf) -> String {
    let mut parser = ArgParser::new(&[]);
    let args_string: Vec<String> = args.iter().map(|s| s.to_string()).collect();

    let mut err_msg: String = "save: ".to_string();

    match parser.parse(&args_string, "save") {
        Ok(_) => {
            let pos_args = parser.get_positional_args();
            if !pos_args.is_empty() {
                err_msg += "Too many positional arguments provided. No arguments expected.";
                log::log_info("restore", err_msg.as_str());
                return err_msg;
            }
            if backup_sekai("save", root_path).is_err() {
                err_msg += "Failed to save Sekai. Please check the logs for more details.";
                log::log_error("save", err_msg.as_str());
                return err_msg;
            }
            "Sekai saved successfully \n".to_string()
        }
        Err(e) => match &e[..] {
            "help" => HELP_TEXT.to_string(),

            _ => "Error parsing arguments. Try 'help save' for more information.".to_string(),
        },
    }
}
