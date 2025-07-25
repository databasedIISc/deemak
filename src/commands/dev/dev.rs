use super::{init_info, lock};
use crate::utils::prompt::UserPrompter;
use std::path::Path;
pub fn dev(
    parts: &[&str],
    current_dir: &Path,
    root_dir: &Path,
    prompter: &mut dyn UserPrompter,
) -> String {
    if parts.is_empty() {
        return "Command not found".into();
    }

    match parts[0] {
        "lock" => {
            let msg = lock::dev_lock(&parts[1..], current_dir, root_dir, prompter);
            if msg.is_err() {
                return msg.err().unwrap();
            }
            msg.unwrap()
        }
        "info" => {
            let msg = init_info::dev_info(&parts[1..], current_dir, root_dir);
            if msg.is_err() {
                return msg.err().unwrap();
            }
            msg.unwrap()
        }

        _ => "Invalid dev command".to_string(),
    }
}
