use super::argparser::ArgParser;
use super::whereami::display_relative_path;
use std::path::Path;

pub const HELP_TXT: &str = r#"
Usage: ls [directory]

Lists the objects and places you can go to in the specified(current by default) directory.
"#;

pub fn ls(args: &[&str], current_dir: &Path, root_dir: &Path) -> String {
    // Create parser with no flags (since ls doesn't need any) and our help text
    let args_string: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    let mut parser = ArgParser::new(&[]);

    // Parse arguments
    match parser.parse(&args_string) {
        Ok(_) => {
            let positional_args = parser.get_positional_args();

            // Determine target path with security checks
            let target_path = if positional_args.is_empty() {
                current_dir.to_path_buf()
            } else {
                let joined = current_dir.join(&positional_args[0]);
                if joined.starts_with(root_dir) {
                    joined
                } else {
                    return "ls: Access denied outside root directory".to_string();
                }
            };

            // Read directory entries
            let entries = match std::fs::read_dir(&target_path) {
                Ok(entries) => entries,
                Err(e) => {
                    let mut error_msg = e.to_string();
                    if e.kind() == std::io::ErrorKind::NotFound {
                        error_msg = "No such file or directory".to_string();
                    }
                    return format!(
                        "ls: cannot access '{}': {}",
                        display_relative_path(&target_path, root_dir),
                        error_msg
                    );
                }
            };

            let mut files = String::new();
            let mut directories = String::new();

            for entry in entries {
                match entry {
                    Ok(entry) => {
                        let path = entry.path();
                        let name = entry.file_name().to_string_lossy().into_owned();

                        if name == "info.json" {
                            continue; // Skip info.json
                        }

                        if path.is_dir() {
                            directories.push_str(&format!("   {}/\n", name));
                        } else {
                            files.push_str(&format!("   {}\n", name));
                        }
                    }
                    Err(e) => {
                        return format!("Error reading entry: {}\n", e);
                    }
                }
            }

            if directories.is_empty() {
                directories = "   (none)\n".to_string();
            }

            // Format output with relative path header
            format!(
                "\nObjects:\n{files}\nFrom inside here, you can go to:\n{directories}",
                files = files,
                directories = directories
            )
        }
        Err(e) => match &e[..] {
            "help" => HELP_TXT,
            "unknown" => "ls: unknown flag\nTry 'help ls' for more information.",
            _ => "Error parsing arguments. Try 'help ls' for more information.",
        }
        .to_string(),
    }
}
