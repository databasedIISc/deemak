use super::*;
use crate::utils::prompt::UserPrompter;
use std::path::Path;
use std::path::PathBuf;

/// CommandResult enum to represent the result of a command execution
pub enum CommandResult {
    Output(String),
    ChangeDirectory(PathBuf, String),
    Clear,
    Exit,
    NotFound,
}

/// Normalizes a path by removing `.` and `..` components
pub fn normalize_path(path: &Path) -> PathBuf {
    let components = path.components();
    let mut normalized = PathBuf::new();

    for component in components {
        match component {
            std::path::Component::ParentDir => {
                if !normalized.pop() {
                    // If we can't go up further, keep the parent dir
                    normalized.push("..");
                }
            }
            std::path::Component::CurDir => {}
            _ => normalized.push(component.as_os_str()),
        }
    }
    normalized
}

/// Command manager that processes commands and processed to return appropriate outputs
pub fn cmd_manager(
    parts: &[&str],
    current_dir: &PathBuf,
    root_dir: &PathBuf,
    prompter: &mut dyn UserPrompter,
) -> CommandResult {
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
        "copy" => {
            let msg = copy::copy(&parts[1..], current_dir, root_dir, prompter);
            CommandResult::Output(msg)
        }
        "tap" => {
            let msg = tap(&parts[1..], current_dir, root_dir);
            CommandResult::Output(msg)
        }
        "del" => {
            let msg = del(&parts[1..], current_dir, root_dir, prompter);
            CommandResult::Output(msg)
        }
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
        "exit" => match exit(prompter) {
            (true, _) => CommandResult::Exit,
            (false, msg) => CommandResult::Output(msg),
        },
        "restore" => CommandResult::Output(restore::restore(&parts[1..], root_dir, prompter)),
        "save" => CommandResult::Output(save::save(&parts[1..], root_dir)),
        _ => CommandResult::NotFound,
    }
}
