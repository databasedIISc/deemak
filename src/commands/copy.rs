use super::argparser::ArgParser;
use super::cmds::normalize_path;
use super::display_relative_path;
use crate::utils::log;
use crate::utils::prompt::UserPrompter;
use crate::utils::valid_sekai::create_dir_info;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub const HELP_TXT: &str = r#"
Usage: copy [OPTIONS] <SOURCE> <DESTINATION>

Copy or move files/directories. By default, performs a copy operation.
Options:
    -x, --cut       Move instead of copy (cut/paste)
    -r, --recursive Copy directories recursively
    -f, --force     Overwrite existing files
    -h, --help      Display this help message

Examples:
- copy file.txt new_file.txt         # Copy file
- copy -r dir/ new_dir/              # Recursive directory copy
- copy -x old.txt new.txt            # Move (cut/paste) file
- copy -x -r old_dir/ new_location/    # Move directory recursively
- copy -f existing_file.txt new_file.txt  # Force overwrite existing file
"#;

/// Validates paths for copy/move operations
fn validate_paths(
    src: &Path,
    dest: &Path,
    current_dir: &PathBuf,
    root_dir: &Path,
) -> Result<(PathBuf, PathBuf), String> {
    let src_path = current_dir.join(src);
    let dest_path = current_dir.join(dest);

    let src_normalized = normalize_path(&src_path);
    let dest_normalized = normalize_path(&dest_path);

    log::log_debug(
        "copy",
        &format!(
            "Source: {}, Destination: {}",
            src_normalized.display(),
            dest_normalized.display()
        ),
    );

    // Verify paths are within root directory
    if !src_normalized.starts_with(root_dir) {
        return Err(format!(
            "copy: {}: Cannot operate outside of root directory",
            src.display()
        ));
    }

    if !dest_normalized.starts_with(root_dir) {
        return Err(format!(
            "copy: {}: Cannot operate outside of root directory",
            dest.display()
        ));
    }

    // Check if source exists
    if !src_normalized.exists() {
        return Err(format!(
            "copy: {}: No such file or directory",
            src.display()
        ));
    }

    Ok((src_normalized, dest_normalized))
}

/// Copies a file from source to destination
fn copy_file(src: &Path, dest: &Path, force: bool) -> io::Result<()> {
    if dest.exists() {
        if !force {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "Destination file exists (use -f to overwrite)",
            ));
        }
        fs::remove_file(dest)?;
    }
    fs::copy(src, dest)?;
    Ok(())
}

/// Recursively copies a directory
fn copy_directory(src: &Path, dest: &Path, root_dir: &Path, force: bool) -> io::Result<String> {
    // Create destination directory (without .dir_info initially)
    if dest.exists() {
        if !force {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "Destination directory exists (use -f to overwrite)",
            ));
        }
        fs::remove_dir_all(dest)?;
    }
    fs::create_dir_all(dest)?;

    // Copy contents except .dir_info
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let entry_path = entry.path();
        let entry_name = entry.file_name();

        // Skip .dir_info
        if entry_name == ".dir_info" {
            continue;
        }

        let new_path = dest.join(entry_name);

        if entry_path.is_dir() {
            copy_directory(&entry_path, &new_path, root_dir, force)?;
        } else {
            copy_file(&entry_path, &new_path, force)?;
        }
    }

    // Create new .dir_info in destination
    if !create_dir_info(&dest.to_path_buf(), false) {
        return Err(io::Error::other(format!(
            "Failed to create .dir_info in {}",
            display_relative_path(dest, root_dir)
        )));
    }

    Ok(format!(
        "Copied directory: {} → {}",
        display_relative_path(src, root_dir),
        display_relative_path(dest, root_dir)
    ))
}

/// Moves a file or directory (cut/paste)
fn move_item(
    src: &Path,
    dest: &Path,
    root_dir: &Path,
    recursive: bool,
    force: bool,
) -> io::Result<String> {
    // Prevent moving .dir_info directly
    if src.ends_with(".dir_info") {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Cannot move .dir_info directly",
        ));
    }

    if src.is_dir() {
        if !recursive {
            return Err(io::Error::other("Use -r for directories"));
        }

        // Prepare destination
        if dest.exists() && !force {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "Use -f to overwrite",
            ));
        }
        fs::create_dir_all(dest)?;

        // Move contents except .dir_info
        fs::read_dir(src)?
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name() != ".dir_info")
            .try_for_each(|e| {
                let new_path = dest.join(e.file_name());
                fs::rename(e.path(), &new_path).or_else(|_| {
                    if e.path().is_dir() {
                        copy_directory(&e.path(), &new_path, root_dir, force)?;
                    } else {
                        copy_file(&e.path(), &new_path, force)?;
                    }
                    fs::remove_dir_all(e.path()).or_else(|_| fs::remove_file(e.path()))
                })
            })?;

        fs::remove_dir(src)?;
        if !create_dir_info(&dest.to_path_buf(), false) {
            return Err(io::Error::other(format!(
                "Failed to create .dir_info in {}",
                display_relative_path(dest, root_dir)
            )));
        }
    } else {
        // Handle file move
        if dest.exists() && !force {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "Use -f to overwrite",
            ));
        }
        fs::rename(src, dest).or_else(|_| {
            copy_file(src, dest, force)?;
            fs::remove_file(src)
        })?;
    }

    Ok(format!(
        "Moved {} → {}",
        display_relative_path(src, root_dir),
        display_relative_path(dest, root_dir)
    ))
}

/// Helper to delete directory contents except .dir_info
fn delete_directory_contents(path: &Path) -> io::Result<()> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let entry_path = entry.path();
        let entry_name = entry.file_name();

        // Skip .dir_info
        if entry_name == ".dir_info" {
            continue;
        }

        if entry_path.is_dir() {
            fs::remove_dir_all(&entry_path)?;
        } else {
            fs::remove_file(&entry_path)?;
        }
    }
    Ok(())
}

/// Main copy command function
pub fn copy(
    args: &[&str],
    current_dir: &PathBuf,
    root_dir: &Path,
    prompter: &mut dyn UserPrompter,
) -> String {
    let valid_flags = vec![
        "-x",
        "--cut",
        "-r",
        "--recursive",
        "-f",
        "--force",
        "-h",
        "--help",
    ];
    let mut parser = ArgParser::new(&valid_flags);

    let args_string: Vec<String> = args.iter().map(|s| s.to_string()).collect();

    log::log_debug(
        "copy",
        &format!(
            "Parsing arguments: {:?}, Current Directory: {}",
            args_string,
            current_dir.display(),
        ),
    );

    match parser.parse(&args_string) {
        Ok(_) => {
            // Get source and destination arguments
            let paths: Vec<&str> = args
                .iter()
                .filter(|&&arg| !valid_flags.contains(&arg))
                .copied()
                .collect();

            if paths.len() != 2 {
                return "copy: Requires exactly two paths (source and destination)".to_string();
            }

            // handle source and destination paths, and flags
            let (src, dest) = (Path::new(paths[0]), Path::new(paths[1]));
            let cut = args.contains(&"-x") || args.contains(&"--cut");
            let recursive = args.contains(&"-r") || args.contains(&"--recursive");
            let force = args.contains(&"-f") || args.contains(&"--force");

            // Prompt for force confirmation
            if force && !prompter.confirm("Are you sure you want to force overwrite files?") {
                return "Operation of force overwriting cancelled. No files copied/moved."
                    .to_string();
            }

            // Validate paths and perform operations
            match validate_paths(src, dest, current_dir, root_dir) {
                Ok((src_path, dest_path)) => {
                    let result = if cut {
                        move_item(&src_path, &dest_path, root_dir, recursive, force)
                    } else if src_path.is_dir() && !recursive {
                        Err(io::Error::other("Cannot copy directory without -r flag"))
                    } else if src_path.is_dir() {
                        copy_directory(&src_path, &dest_path, root_dir, force)
                    } else {
                        copy_file(&src_path, &dest_path, force).map(|_| {
                            format!(
                                "Copied {} to {}",
                                display_relative_path(&src_path, root_dir),
                                display_relative_path(&dest_path, root_dir)
                            )
                        })
                    };

                    match result {
                        Ok(msg) => msg,
                        Err(e) => format!("copy: {}", e),
                    }
                }
                Err(e) => e,
            }
        }
        Err(e) => match &e[..] {
            "help" => HELP_TXT.to_string(),
            "unknown" => "copy: unknown flag\nTry 'help copy' for more information.".to_string(),
            _ => "Error parsing arguments. Try 'help copy' for more information.".to_string(),
        },
    }
}
