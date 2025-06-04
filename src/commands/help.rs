use super::*;

pub fn get_command_help(command: &str) -> Option<&'static str> {
    match command {
        "echo" => Some(echo::HELP_TXT),
        "go" => Some(go::HELP_TXT),
        "ls" => Some(ls::HELP_TXT),
        "help" => Some("help [command]: Displays help for the specified command."),
        "read" => Some(read::HELP_TXT),
        "whereami" => Some("whereami: Displays the current directory."),
        "whoami" => Some("whoami: Displays who you are."),
        "exit" => Some("exit: Exits the program."),
        "clear" => Some("clear: Clears the screen."),
        "restore" => Some(restore::HELP_TEXT),
        "save" => Some(save::HELP_TEXT),
        _ => None,
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
- whereami: Displays where you are.
- help: Displays this help message.
- exit: Exits the program.
- clear: Clears the screen.
- restore: Restores the Sekai from starting point.
- save: Saves your current progress of the Sekai.

Try `help command` to get help on a specific command.
"#;
        help_text.to_string()
    } else {
        get_command_help(cmd)
            .unwrap_or("No help available for this command.")
            .to_string()
    }
}
