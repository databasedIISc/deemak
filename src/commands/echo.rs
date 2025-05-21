pub fn echo(args: &[&str]) -> String {
    if args.is_empty() {
        return String::new();
    }
    args.join(" ")
}

pub const HELP_TXT: &str = r#"
Usage: echo [message]

Echoes the message back to you.
"#;
