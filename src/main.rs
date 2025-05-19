mod keys;
mod screen;
mod server;
use deemak::menu;
mod log;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let debug_mode: bool = args.iter().any(|arg| arg == "debug");
    log::log_info("Starting application", debug_mode);

    // We have 2 modes, the web and the raylib gui. The web argument runs it on the web, else
    // raylib gui is set by default.
    if args.iter().any(|arg| arg == "web") {
        server::launch_web();
        return;
    }

    // Initialize Raylib window
    let loglevel = if !debug_mode {
        raylib::consts::TraceLogLevel::LOG_ERROR
    } else {
        raylib::consts::TraceLogLevel::LOG_ALL
    };
    let (mut rl, thread) = raylib::init().log_level(loglevel).size(800, 600).title("DEEMAK Shell").build();
    rl.set_trace_log(loglevel);
    log::log_info("Raylib initialized successfully", debug_mode);

    // Main menu loop
    loop {
        match menu::show_menu(&mut rl, &thread) {
            Some(0) => {
                // Shell mode
                let mut shell = screen::ShellScreen::new_world(rl, thread, debug_mode);
                shell.run();
                break; // Exit after shell closes
            }
            Some(1) => {
                // About screen
                menu::about::show_about(&mut rl, &thread, debug_mode);
            }
            Some(2) | None => {
                // Exit
                break;
            }
            _ => unreachable!(),
        }
    }
}
