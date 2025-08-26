use crate::epr_log_error;
use crate::gui_shell::ShellScreen;
use crate::menu::{self, menu_options::MenuOption};
use crate::metainfo::valid_sekai::validate_or_create_sekai;
use crate::rns::restore_comp;
use crate::utils::{globals::get_sekai_dir, log};
use raylib::prelude::{RaylibHandle, RaylibThread};
use std::path::Path;

/// Validates the Sekai directory and sets it as the world directory
pub fn sekai_initialize(sekai_path: &Path) {
    log::log_info(
        "SEKAI",
        format!("Starting Sekai validation for: {}", sekai_path.display()).as_str(),
    );
    // Just check first for HOME directory validity and create if not.
    // If not valid, create .dir_info for each of them.
    if !validate_or_create_sekai(sekai_path, false) {
        epr_log_error!(
            "SEKAI",
            "Sekai directory is not valid even after creating default `.dir_info`. Sekai: {sekai_path:?}"
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
                sekai_path.join(".dir_info/restore_me.deemak")
            ),
        );
        // restore_me should be made initially if it doesnt exist, else it will not be created
        match restore_comp::backup_sekai("restore", sekai_path) {
            Err(e) => {
                log::log_error(
                    "SEKAI",
                    &format!("Failed to create restore file for: {sekai_path:?} Error: {e}"),
                );
                eprintln!(
                    "Error: Failed to create restore file for: {sekai_path:?} Error: {e}.\nContinuing..."
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
                sekai_path.join(".dir_info/save_me.deemak")
            ),
        );
        // Not copying the restore file to save file, since the password will be different.
        match restore_comp::backup_sekai("save", sekai_path) {
            Err(e) => {
                log::log_error(
                    "SEKAI",
                    &format!("Failed to create save file for: {sekai_path:?} Error: {e}"),
                );
                eprintln!(
                    "Error: Failed to create save file for: {sekai_path:?} Error: {e}.\nContinuing..."
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
                "Error: Failed to restore Sekai from save file at {sekai_path:?}. Error: {err}
Continuing..."
            );
        }
        Ok(_) => {
            log::log_info("SEKAI", "Sekai restored successfully from save file");
        }
    }
}

#[derive(Debug, Clone, Default)]
struct InitBools {
    tutorial_initialized: bool,
    sekai_initialized: bool,
}

/// Runs the main GUI loop for the Sekai shell
pub fn run_gui_loop(rl: &mut RaylibHandle, thread: &RaylibThread, font_size: f32) {
    let mut init_state = InitBools::default();
    // This will exist, since sekai directory is set in main.rs
    let sekai_path = &get_sekai_dir();
    let mut sekai_shell = ShellScreen::new_sekai(rl, thread, sekai_path.to_path_buf(), font_size);

    // Initialize the Tutorial directory
    let tutorial_dir = &Path::new(env!("CARGO_MANIFEST_DIR")).join("_tutorial");
    let mut tutorial_shell =
        ShellScreen::new_sekai(rl, thread, tutorial_dir.to_path_buf(), font_size);

    loop {
        // Show main menu and get user selection
        let menu_selection = menu::show_menu(rl, thread);
        match menu_selection {
            Some(MenuOption::StartShell) => {
                // Shell mode
                log::log_info(
                    "Deemak",
                    format!("Starting Shell: Sekai: {}", sekai_path.display()).as_str(),
                );
                if !init_state.sekai_initialized {
                    sekai_initialize(sekai_path);
                    init_state.sekai_initialized = true;
                }
                sekai_shell.run(rl, thread);
            }
            Some(MenuOption::About) => {
                // About screen
                menu::about::show_about(rl, thread);
                // After about screen closes, return to menu
                continue;
            }
            Some(MenuOption::Tutorial) => {
                // Tutorial screen
                log::log_info("Deemak", "Loading Tutorial");
                if !init_state.tutorial_initialized {
                    sekai_initialize(tutorial_dir);
                    tutorial_shell =
                        ShellScreen::new_sekai(rl, thread, tutorial_dir.to_path_buf(), font_size);
                    init_state.tutorial_initialized = true;
                }
                tutorial_shell.run(rl, thread);
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
                crate::utils::cleanup::exit_deemak(0);
            }
        }
    }
}
