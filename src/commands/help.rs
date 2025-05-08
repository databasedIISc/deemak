pub fn help() -> String {
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
"#;
    help_text.to_string()
}
