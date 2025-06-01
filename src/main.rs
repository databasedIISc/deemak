mod keys;
mod screen;
mod server;
mod utils;
use deemak::DEBUG_MODE;
use deemak::menu;
use raylib::ffi::{SetConfigFlags, SetTargetFPS};
use raylib::prelude::get_monitor_width;
use utils::debug_mode;
use utils::log;

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
            &format!("sekai directory provided: {:?}", sekai_path),
        );
        Some(sekai_path)
    } else {
        log::log_error(
            "SEKAI",
            "No sekai directory provided. Please specify a sekai directory as the first argument.",
        );
        None
    };

    if sekai_dir.is_none() {
        log::log_error("SEKAI", "sekai directory is required. Exiting.");
        return;
    }

    // We have 2 modes, the web and the raylib gui. The web argument runs it on the web, else
    // raylib gui is set by default.
    if args.iter().any(|arg| arg == "--web") {
        server::launch_web();
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
                let mut shell = screen::ShellScreen::new_sekai(
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
