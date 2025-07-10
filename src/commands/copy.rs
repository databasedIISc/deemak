use super::argparser::ArgParser;
use super::cmds::normalize_path;
use super::display_relative_path;
use crate::commands::cmds::check_dir_info;
use crate::metainfo::info_reader::*;
use crate::metainfo::lock_perm;
use crate::metainfo::valid_sekai::create_dir_info;
use crate::utils::{log, prompt::UserPrompter};
use std::fs;
use std::io::{self, Error};
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
- copy -f /path/to/existing_file.txt /path/to/new_file.txt  # Force overwrite existing file
"#;

fn _print_dir_contents(path: &Path) {
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.filter_map(Result::ok) {
            let entry_path = entry.path();
            if entry_path.is_dir() {
                println!("{} (directory)", entry_path.display());
            } else {
                println!("{}", entry_path.display());
            }
        }
    } else {
        println!("Failed to read directory: {}", path.display());
    }
}

/// Validates paths for copy/move operations
fn validate_paths(
    src: &Path,
    dest: &Path,
    current_dir: &Path,
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

/// Moves object metadata from source to destination in their respective info.json files
fn move_obj_info(src: &Path, dest: &Path, cut: bool) -> Result<(), String> {
    // Get source and destination parent directories
    let src_parent = src.parent().ok_or("Source has no parent directory")?;
    let dest_parent = dest.parent().ok_or("Destination has no parent directory")?;

    // Get source and destination info.json paths
    let src_info_path = src_parent.join(".dir_info/info.json");
    let dest_info_path = dest_parent.join(".dir_info/info.json");

    // Get source object name
    let src_obj_name = src
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or("Invalid source object name")?;

    // Get destination object name
    let dest_obj_name = dest
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or("Invalid destination object name")?;

    // Read source object info (if exists)
    let src_obj_info = match read_get_obj_info(&src_info_path, src_obj_name) {
        Ok(info) => info,
        Err(_) => return Ok(()), // No metadata to move
    };

    // Remove source object info
    if cut {
        del_obj_from_info(src, src_obj_name)
            .map_err(|e| format!("Failed to remove source metadata: {}", e))?;
    }

    // Add destination object info with same properties
    add_obj_to_info(dest, dest_obj_name, Some(src_obj_info.properties))
        .map_err(|e| format!("Failed to add destination metadata: {}", e))?;

    Ok(())
}

/// Copies a file from source to destination
fn copy_file(
    src: &Path,
    dest: &Path,
    root_dir: &Path,
    force: bool,
    cut: bool,
) -> io::Result<String> {
    if dest.exists() {
        if !force {
            return Err(Error::new(
                io::ErrorKind::AlreadyExists,
                "Destination file exists (use -f to overwrite)",
            ));
        }
        fs::remove_file(dest)?;
    }
    fs::copy(src, dest)?;

    // Copy metadata
    if let Err(e) = move_obj_info(src, dest, false) {
        // false = copy operation
        log::log_warning(
            "copy",
            &format!("Failed to copy metadata for {}: {}", dest.display(), e),
        );
    }

    Ok(format!(
        "{} {} to {}",
        if cut { "Moved" } else { "Copied" },
        display_relative_path(src, root_dir),
        display_relative_path(dest, root_dir)
    ))
}

/// Recursively copies a directory
fn copy_directory(
    src: &Path,
    dest: &Path,
    root_dir: &Path,
    force: bool,
    cut: bool,
) -> io::Result<String> {
    if dest.exists() {
        if !force {
            return Err(Error::new(
                io::ErrorKind::AlreadyExists,
                "Destination directory exists (use -f to overwrite)",
            ));
        }
        fs::remove_dir_all(dest)?;
    }
    fs::create_dir_all(dest)?;

    // First copy directory metadata
    if let Err(e) = move_obj_info(src, dest, false) {
        // false = copy operation
        log::log_warning(
            "copy",
            &format!("Failed to copy metadata for {}: {}", dest.display(), e),
        );
    }

    // Then copy contents
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let entry_path = entry.path();
        let entry_name = entry.file_name();

        if entry_name == ".dir_info" {
            continue;
        }

        let new_path = dest.join(entry_name);
        if entry_path.is_dir() {
            copy_directory(&entry_path, &new_path, root_dir, force, cut)?;
        } else {
            copy_file(&entry_path, &new_path, root_dir, force, cut)?;
        }
    }

    Ok(format!(
        "{} directory: {} → {}",
        if cut { "Moved" } else { "Copied" },
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
    if src.ends_with(".dir_info") {
        return Err(Error::new(
            io::ErrorKind::InvalidInput,
            "Cannot move a restricted file/directory. Operation not allowed",
        ));
    }

    if src.is_dir() {
        if !recursive {
            return Err(Error::other("Use -r for directories"));
        }

        if dest.exists() && !force {
            return Err(Error::new(
                io::ErrorKind::AlreadyExists,
                "Use -f to overwrite",
            ));
        }
        fs::create_dir_all(dest)?;

        // Create .dir_info for destination first
        if !create_dir_info(dest, false) {
            return Err(Error::other(format!(
                "Failed to create .dir_info in {}",
                display_relative_path(dest, root_dir)
            )));
        }

        // First move the directory metadata
        if let Err(e) = move_obj_info(src, dest, true) {
            log::log_warning(
                "move",
                &format!(
                    "Failed to move directory metadata for {}: {}",
                    dest.display(),
                    e
                ),
            );
        }

        // Process all contents
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let entry_path = entry.path();
            let entry_name = entry.file_name();

            if entry_name == ".dir_info" {
                continue;
            }

            let new_path = dest.join(entry_name);

            // Handle metadata first for each item
            if let Err(e) = move_obj_info(&entry_path, &new_path, true) {
                log::log_warning(
                    "move",
                    &format!("Failed to move metadata for {}: {}", new_path.display(), e),
                );
            }

            // Then handle the actual file/directory move
            if entry_path.is_dir() {
                fs::create_dir_all(&new_path)?;
                if !create_dir_info(&new_path, false) {
                    return Err(Error::other(format!(
                        "Failed to create .dir_info in {}",
                        display_relative_path(&new_path, root_dir)
                    )));
                }

                // Recursively move subdirectory
                move_item(&entry_path, &new_path, root_dir, true, force)?;
                fs::remove_dir_all(&entry_path)?;
            } else {
                fs::rename(&entry_path, &new_path).or_else(|_| {
                    copy_file(&entry_path, &new_path, root_dir, force, true)?;
                    fs::remove_file(&entry_path)
                })?;
            }
        }

        // Clean up source directory
        fs::remove_dir_all(src)?;
    } else {
        // Handle file move
        if dest.exists() && !force {
            return Err(Error::new(
                io::ErrorKind::AlreadyExists,
                "Use -f to overwrite",
            ));
        }

        // Move metadata first
        if let Err(e) = move_obj_info(src, dest, true) {
            log::log_warning(
                "move",
                &format!("Failed to move file metadata for {}: {}", dest.display(), e),
            );
        }

        // Then move the file
        fs::rename(src, dest).or_else(|_| {
            copy_file(src, dest, root_dir, force, true)?;
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
    current_dir: &Path,
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

    match parser.parse(&args_string, "copy") {
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

            // If restricted file/directory used in paths, return error
            for pth in [&src, &dest] {
                if check_dir_info(pth) {
                    return "copy: Cannot copy/refer restricted file or directory. Operation Not Allowed."
                        .to_string();
                }
            }

            // Prompt for force confirmation
            if force && !prompter.confirm("Are you sure you want to force overwrite files?") {
                return "Operation of force overwriting cancelled. No files copied/moved."
                    .to_string();
            }

            // Validate paths and perform operations
            match validate_paths(src, dest, current_dir, root_dir) {
                Ok((src_path, dest_path)) => {
                    // Operation allowed only if paths are not locked
                    for pth in [&src_path, &dest_path] {
                        if let Err(e) = lock_perm::operation_locked_perm(
                            pth,
                            "copy",
                            "Cannot copy/move locked file/directory. Unlock it first.",
                        ) {
                            return e;
                        }
                    }
                    let result = if cut {
                        move_item(&src_path, &dest_path, root_dir, recursive, force)
                    } else if src_path.is_dir() && !recursive {
                        Err(Error::other("Cannot copy directory without -r flag"))
                    } else if src_path.is_dir() {
                        copy_directory(&src_path, &dest_path, root_dir, force, false)
                    } else {
                        copy_file(&src_path, &dest_path, root_dir, force, false).map(|_| {
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
