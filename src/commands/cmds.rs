use super::*;

pub enum CommandResult {
    Output(String),
    Clear,
    Exit,
    NotFound,
}

pub fn cmd_manager(parts: &[&str]) -> CommandResult {
    if parts.is_empty() {
        return CommandResult::NotFound;
    }

    match parts[0] {
        "echo" => CommandResult::Output(echo(&parts[1..])),
        "whoami" => CommandResult::Output(whoami()),
        "help" => CommandResult::Output(help()),
        "clear" => CommandResult::Clear,
        "exit" => CommandResult::Exit,
        _ => CommandResult::NotFound,
    }
}
