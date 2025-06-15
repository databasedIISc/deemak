use super::*;
use std::path::PathBuf;
use crate::utils::prompt::UserPrompter;

/// CommandResult enum to represent the result of a command execution
pub enum CommandResult {
    Output(String),
    ChangeDirectory(PathBuf, String),
    Clear,
    Exit,
    NotFound,
}

/// Command manager that processes commands and processed to return appropriate outputs
pub fn cmd_manager(parts: &[&str], current_dir: &PathBuf, root_dir: &PathBuf, prompter: &mut dyn UserPrompter) -> CommandResult {
    if parts.is_empty() {
        return CommandResult::NotFound;
    }

    match parts[0] {
        "echo" => CommandResult::Output(echo(&parts[1..])),
        "whoami" => CommandResult::Output("Database Deemak User.".to_string()),
        "go" => {
            let (new_dir, msg) = go(&parts[1..], current_dir, root_dir);
            CommandResult::ChangeDirectory(new_dir, msg)
        }
        "ls" => CommandResult::Output(ls(&parts[1..], current_dir, root_dir)),
        "read" => CommandResult::Output(read(&parts[1..], current_dir, root_dir)),
        "whereami" => CommandResult::Output(whereami(current_dir, root_dir)),
        "help" => {
            if parts.len() > 1 {
                match help::get_command_help(parts[1]) {
                    Some(msg) => CommandResult::Output(msg.to_string()),
                    None => CommandResult::Output(format!("No help available for '{}'", parts[1])),
                }
            } else {
                CommandResult::Output(help(""))
            }
        }
        "clear" => CommandResult::Clear,
        "exit" => CommandResult::Exit,
        "restore" => CommandResult::Output(restore::restore(&parts[1..], root_dir, prompter)),
        "save" => CommandResult::Output(save::save(&parts[1..], root_dir)),
        _ => CommandResult::NotFound,
    }
}
