use crate::gui_shell::{FIRST_RUN, ShellScreen};
use crate::menu::{self, menu_options::MenuOption};
use crate::metainfo::valid_sekai::validate_or_create_sekai;
use crate::rns::restore_comp;
use crate::utils::globals::get_world_dir;
use crate::utils::log;
use crate::utils::{find_root, globals::set_world_dir};
use raylib::prelude::{RaylibHandle, RaylibThread};
use std::path::Path;

/// Validates the Sekai directory and sets it as the world directory
pub fn sekai_no_hajimari(sekai_path: &Path) {
    // First Validate Home
    if !validate_or_create_sekai(sekai_path, true) {
        log::log_error(
            "SEKAI",
            &format!(
                "Sekai directory HOME is not valid. Creating default `.dir_info` for HOME at {sekai_path:?}"
            ),
        );
    }
    // Just check first for HOME directory validity and create if not.
    let root_dir;
    match find_root::find_home(sekai_path) {
        Ok(Some(sekai_dir)) => {
            log::log_info(
                "SEKAI",
                &format!("Found root directory for Sekai: {}", sekai_dir.display()),
            );
            // Set the global Sekai directory
            root_dir = Some(sekai_dir.clone());
            set_world_dir(sekai_dir);
        }
        Ok(None) => {
            log::log_error(
                "SEKAI",
                "Failed to find root directory for Sekai. No HOME location found. Exiting.",
            );
            eprintln!("Error: Failed to find root directory for Sekai. Exiting.");
            return;
        }
        Err(e) => {
            log::log_error(
                "SEKAI",
                &format!("Process failed while finding Sekai HOME. Error: {e}. Exiting."),
            );
            eprintln!("Process failed while finding Sekai HOME. Error: {e}. Exiting.");
            return;
        }
    }
    // If not valid, create .dir_info for each of them.
    if !validate_or_create_sekai(sekai_path, false) {
        log::log_error(
            "SEKAI",
            &format!(
                "Sekai directory is not valid even after creating default `.dir_info`. Sekai: {sekai_path:?}"
            ),
        );
        eprintln!(
            "Error: Sekai directory is not valid even after creating default `.dir_info`. Please check the sekai validity. Sekai: {sekai_path:?}"
        );
        return;
    } else {
        // sekai is valid
        log::log_info("SEKAI", &format!("Sekai is Valid {sekai_path:?}"));

        // Create the restore file if it doesn't exist, since it is required for restoring. The
        // progress will be saved as `save_me` and will be recreated every run.
        log::log_info(
            "SEKAI",
            &format!(
                "Creating restore file for Sekai at {:?}",
                sekai_path.join(".dir_info/restore_me")
            ),
        );
        // restore_me should be made initially if it doesnt exist, else it will not be created
        match restore_comp::backup_sekai("restore", root_dir.as_ref().unwrap()) {
            Err(e) => {
                log::log_error("SEKAI", &format!("Failed to create restore file: {e}"));
                eprintln!(
                    "Error: Failed to create restore file: {e}
Continuing..."
                );
                return;
            }
            Ok(msg) => {
                log::log_info("SEKAI", &msg);
            }
        }

        // save_me should be made initially if it doesnt exist, it will be created every run
        log::log_info(
            "SEKAI",
            &format!(
                "Creating save file for Sekai at {:?}",
                sekai_path.join(".dir_info/save_me")
            ),
        );
        match restore_comp::backup_sekai("save", root_dir.as_ref().unwrap()) {
            Err(e) => {
                log::log_error("SEKAI", &format!("Failed to create save file: {e}"));
                eprintln!(
                    "Error: Failed to create save file: {e}
Continuing..."
                );
                return;
            }
            Ok(msg) => {
                log::log_info("SEKAI", &msg);
            }
        }
    }

    // If `save_me` already exists, then the sekai will be restored from it.
    match restore_comp::restore_sekai("save", sekai_path) {
        Err(err) => {
            log::log_error(
                "SEKAI",
                &format!("Failed to restore Sekai from save file: {err}"),
            );
            eprintln!(
                "Error: Failed to restore Sekai from save file at {sekai_path:?}
Continuing..."
            );
        }
        Ok(_) => {
            log::log_info("SEKAI", "Sekai restored successfully from save file");
        }
    }
}

/// Runs the main GUI loop for the Sekai shell
pub fn run_gui_loop(
    rl: &mut RaylibHandle,
    thread: &RaylibThread,
    font_size: f32,
    sekai_path: &Path,
) {
    loop {
        // Show main menu and get user selection
        let menu_selection = menu::show_menu(rl, thread);

        match menu_selection {
            Some(MenuOption::StartShell) => {
                // Shell mode
                unsafe { FIRST_RUN = true }; // Reset first run flag
                let sekai_dir = get_world_dir();
                log::log_info(
                    "Deemak",
                    format!("Starting Shell: Sekai: {}", sekai_dir.display()).as_str(),
                );
                let mut shell = ShellScreen::new_sekai(rl, thread, sekai_dir.clone(), font_size);
                shell.run();
            }
            Some(MenuOption::About) => {
                // About screen
                menu::about::show_about(rl, thread);
                // After about screen closes, return to menu
                continue;
            }
            Some(MenuOption::Tutorial) => {
                // Tutorial screen
                let tutorial_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("_tutorial");
                log::log_info("Deemak", "Loading Tutorial");
                unsafe { FIRST_RUN = true }; // Reset first run flag
                let mut tutorial_shell =
                    ShellScreen::new_sekai(rl, thread, tutorial_dir, font_size);
                tutorial_shell.run();
                continue;
            }
            Some(MenuOption::Settings) => {
                // Settings screen
                menu::settings::show_settings(rl, thread);
                // After settings screen closes, return to menu
                continue;
            }
            Some(MenuOption::Exit) | None => {
                // Exit
                std::process::exit(0); // Exit the application
            }
        }
    }
}
