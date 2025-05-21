use super::*;
use std::path::{Path, PathBuf};

pub enum CommandResult {
    Output(String),
    ChangeDirectory(PathBuf, String),
    Clear,
    Exit,
    NotFound,
}

pub fn cmd_manager(parts: &[&str], current_dir: &PathBuf, root_dir: &Path) -> CommandResult {
    if parts.is_empty() {
        return CommandResult::NotFound;
    }

    match parts[0] {
        "echo" => CommandResult::Output(echo(&parts[1..])),
        "whoami" => CommandResult::Output("Database Deemak User.".to_string()),
        "go" => {
            let (new_dir, msg) = go(&parts[1..], &current_dir, root_dir);
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
        _ => CommandResult::NotFound,
    }
}
