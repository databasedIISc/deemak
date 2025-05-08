mod commands;
mod keys;
mod screen;

fn main() {
    // Initialize the shell screen
    let mut shell = screen::ShellScreen::new();

    // Main game loop
    while !shell.window_should_close() {
        shell.update();
        shell.draw();
    }
}
