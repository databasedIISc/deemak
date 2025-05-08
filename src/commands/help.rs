pub fn help() -> String {
    let help_text = r#"
Welcome to DBD Deemak Help. You can use the following commands:

- echo <message>: Echoes the message back to you.
- whoami: Displays the current user.
- help: Displays this help message.
- exit: Exits the program.
- clear: Clears the screen.
"#;
    help_text.to_string()
}
