use super::*;

pub fn get_command_help(command: &str) -> Option<&'static str> {
    match command {
        "echo" => Some(echo::HELP_TXT),
        "go" => Some(go::HELP_TXT),
        "ls" => Some(ls::HELP_TXT),
        "help" => Some("help [command]: Displays help for the specified command."),
        "read" => Some(read::HELP_TXT),
        "copy" => Some(copy::HELP_TXT),
        "tap" => Some(tap::HELP_TXT),
        "del" => Some(del::HELP_TXT),
        "whereami" => Some("whereami: Displays the current directory."),
        "whoami" => Some("whoami: Displays who you are."),
        "exit" => Some("exit: Exits the program."),
        "clear" => Some("clear: Clears the screen."),
        "restore" => Some(restore::HELP_TEXT),
        "save" => Some(save::HELP_TEXT),
        "solve" => Some(solve::HELP_TEXT),
        "unlock" => Some(unlock::HELP_TXT),
        _ => Some("No help available for this command. Check if the command is valid."),
    }
}

pub fn help(cmd: &str) -> String {
    if cmd.is_empty() {
        let help_text = r#"
Welcome to DBD Deemak Help. You can use the following commands:

- echo <message>: Echoes the message back to you.
- whoami: Displays who you are.
- go <directory>: Changes the current directory to the specified directory.
- ls: Lists the objects and places you can go to in the current directory.
- read <file>: Reads the specified file.
- copy <source> <destination>: Copies a file/directory from source to destination.
- tap <file>: Creates new file/directory with the specified name.
- del <file>: Deletes the specified file/directory.
- whereami: Displays where you are.
- help: Displays this help message.
- exit: Exits the program.
- clear: Clears the screen.
- restore: Restores the Sekai to last saved version or to starting point( if forced)
- save: Saves your current progress of the Sekai.
-solve: after completing a level, use this command to submit your answer and obtain the flag.
-unlock: use flag to unlock levels and chests.
- help <command>: Displays help for the specified command.
"#;
        help_text.to_string()
    } else {
        get_command_help(cmd)
            .unwrap_or("No help available for this command.")
            .to_string()
    }
}
