mod gui_shell;
mod keys;
mod server;
mod utils;
use deemak::DEBUG_MODE;
use deemak::menu;
use raylib::ffi::{SetConfigFlags, SetTargetFPS};
use raylib::prelude::get_monitor_width;
use utils::{debug_mode, valid_sekai, globals, find_root, log, restore_comp, valid_sekai};
use valid_sekai::validate_sekai;
use std::path::PathBuf;
use once_cell::sync::OnceCell;

static WORLD_DIR: OnceCell<PathBuf> = OnceCell::new();

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
        if !validate_sekai(&sekai_path) {
            log::log_error(
                "SEKAI",
                &format!("sekai directory does not exist: {:?}", sekai_path),
            );
            eprintln!("Error: sekai directory does not exist: {:?}", sekai_path);
            return;
        } else {
            log::log_info("SEKAI", &format!("Sekai is Valid {:?}", sekai_path));

            // Create the restore file if it does/doesn't exist, because anyways that world only
            // will be played, so make the restore file of that
            log::log_info(
                "SEKAI",
                &format!(
                    "Creating restore file for Sekai at {:?}",
                    sekai_path.join(".dir_info/restore_me")
                ),
            );
            let root_dir =
                find_root::find_home(&sekai_path).expect("Failed to find root directory");
            if let Err(e) = restore_comp::backup_sekai("restore", &root_dir) {
                log::log_error("SEKAI", &format!("Failed to create restore file: {}", e));
                eprintln!("Error: Failed to create restore file: {}\nContinuing...", e);
                return;
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

    globals::WORLD_DIR.set(sekai_dir.clone().unwrap()).expect("Failed to set world dir");
    
    // We have 2 modes, the web and the raylib gui. The web argument runs it on the web, else
    // raylib gui is set by default.
    if args.iter().any(|arg| arg == "--web") {
        log::log_info("Application", "Running in web mode");
        // server::launch_web(sekai_dir.clone().unwrap());
        server::server();
        return;
    }

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
