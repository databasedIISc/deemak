#![allow(unused_variables, unused_mut, dead_code, unused_imports)]
mod gui_shell;
mod keys;
mod server;
mod utils;
use deemak::DEBUG_MODE;
use deemak::menu;
use raylib::ffi::{SetConfigFlags, SetTargetFPS};
use raylib::prelude::get_monitor_width;
use utils::{debug_mode, find_root, globals, log, restore_comp, valid_sekai};
use valid_sekai::validate_or_create_sekai;

pub const HELP_TXT: &str = r#"
Usage: deemak <sekai_directory> [--debug] [--web]

Options:
  <sekai_directory> [Required]  :   Path to the Sekai directory to parse.
  --debug [Optional]            :   Enable debug mode for more verbose logging.
  --web [Optional]              :   Run the application in web mode (requires a web server).
"#;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    // first argument is sekai name to parse
    DEBUG_MODE
        .set(args.iter().any(|arg| arg == "--debug"))
        .expect("DEBUG_MODE already set");
    log::log_info("Application", "Starting DEEMAK Shell");

    let sekai_dir = if args.len() > 1 {
        // get absolute path to the sekai directory
        let sekai_path = std::env::current_dir().unwrap().join(&args[1]);
        log::log_info(
            "SEKAI",
            &format!("Sekai directory provided: {:?}", sekai_path),
        );
        // If not valid, create .dir_info for each of them.
        if !validate_or_create_sekai(&sekai_path) {
            log::log_error(
                "SEKAI",
                &format!(
                    "Sekai directory is not valid even after creating default `.dir_info`. Sekai: {:?}",
                    sekai_path
                ),
            );
            eprintln!(
                "Error: Sekai directory is not valid even after creating default `.dir_info`. Please check the sekai validity. Sekai: {:?}",
                sekai_path
            );
            return;
        } else {
            // sekai is valid
            log::log_info("SEKAI", &format!("Sekai is Valid {:?}", sekai_path));
            let root_dir = find_root::find_home(&sekai_path);
            if root_dir.is_some() {
                log::log_info(
                    "SEKAI",
                    &format!("Found root directory for Sekai: {:?}", root_dir),
                );
            } else {
                log::log_error("SEKAI", "Failed to find root directory for Sekai. Exiting.");
                eprintln!("Error: Failed to find root directory for Sekai. Exiting.");
                return;
            }

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
                    log::log_error("SEKAI", &format!("Failed to create restore file: {}", e));
                    eprintln!(
                        "Error: Failed to create restore file: {}
Continuing...",
                        e
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
                    log::log_error("SEKAI", &format!("Failed to create save file: {}", e));
                    eprintln!(
                        "Error: Failed to create save file: {}
Continuing...",
                        e
                    );
                    return;
                }
                Ok(msg) => {
                    log::log_info("SEKAI", &msg);
                }
            }
        }
        Some(sekai_path)
    } else {
        // args.len() == 1
        log::log_error("Application", "Invalid arguments provided.");
        eprintln!("Error: At least one argument is required.");
        println!("{}", HELP_TXT);
        return;
    };

    // If `save_me` already exists, then the sekai will be restored from it.
    match restore_comp::restore_sekai("save", &sekai_dir.clone().unwrap()) {
        Err(err) => {
            log::log_error(
                "SEKAI",
                &format!("Failed to restore Sekai from save file: {}", err),
            );
            eprintln!(
                "Error: Failed to restore Sekai from save file at {:?}
Continuing...",
                sekai_dir
            );
        }
        Ok(_) => {
            log::log_info("SEKAI", "Sekai restored successfully from save file");
        }
    }

    globals::WORLD_DIR
        .set(sekai_dir.clone().unwrap())
        .expect("Failed to set world dir");

    // NOTE: All Directory operations and variables settings should be done before this point.
    //
    // We have 2 modes, the web and the raylib gui. The web argument runs it on the web, else
    // raylib gui is set by default.
    //
    // NOTE: #############    SERVER USAGE    #############
    //
    // Initialize the server if --web argument is provided
    if args.iter().any(|arg| arg == "--web") {
        log::log_info("Application", "Running in web mode");
        // server::launch_web(sekai_dir.clone().unwrap());
        let _ = server::server();
        return;
    }

    // NOTE: #############    RAYLIB GUI USAGE    #############
    //
    // Initialize Raylib window
    unsafe {
        SetConfigFlags(4);
        SetTargetFPS(60);
    }
    let loglevel = if !debug_mode() {
        raylib::consts::TraceLogLevel::LOG_ERROR
    } else {
        raylib::consts::TraceLogLevel::LOG_ALL
    };

    let (mut rl, thread) = raylib::init()
        .log_level(loglevel)
        .size(800, 600)
        .title("DEEMAK Shell")
        .build();
    let font_size = get_monitor_width(0) as f32 / 73.5;
    rl.set_trace_log(loglevel);
    log::log_info("Application", "DEEMAK initialized successfully");

    // Main menu loop
    loop {
        match menu::show_menu(&mut rl, &thread) {
            Some(0) => {
                // Shell mode
                let mut shell = gui_shell::ShellScreen::new_sekai(
                    rl,
                    thread,
                    sekai_dir.clone().unwrap(),
                    font_size,
                );
                shell.run();
                break; // Exit after shell closes
            }
            Some(1) => {
                // About screen
                menu::about::show_about(&mut rl, &thread);
            }
            Some(2) | None => {
                // Exit
                break;
            }
            _ => unreachable!(),
        }
    }
}
