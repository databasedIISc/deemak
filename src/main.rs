mod keys;
mod screen;
mod server;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 && args[1] == "web" {
        // Launch Rocket web server ()
        server::launch_web();
    } else {
        // Launch terminal shell
        let mut shell = screen::ShellScreen::new();

        while !shell.window_should_close() {
            shell.update();
            shell.draw();
        }
    }
}
