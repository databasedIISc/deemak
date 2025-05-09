mod keys;
mod menu;
mod screen;
mod server;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 && args[1] == "web" {
        // Launch Rocket web server ()
        server::launch_web();
    } else {
        // Launch terminal shell
        let (mut rl, thread) = raylib::init().size(800, 600).title("DEEMAK Shell").build();

        // Show menu and get selection
        let selection = menu::show_menu(&mut rl, &thread);

        match selection {
            Some(0) => {
                // Create shell using existing Raylib instance
                let mut shell = screen::ShellScreen::new_with_context(rl, thread);
                shell.run();
            }
            Some(1) => println!("Settings would go here"),
            _ => {} // Exit
        }
    }
}
