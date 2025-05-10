use crate::keys::key_to_char;
use commands::CommandResult;
use deemak::commands;
use deemak::utils;
use raylib::prelude::*;
use std::path::PathBuf;

pub struct ShellScreen {
    rl: RaylibHandle,
    thread: RaylibThread,
    input_buffer: String,
    output_lines: Vec<String>,
    current_dir: PathBuf,
    root_dir: PathBuf,
}

pub const DEEMAK_BANNER: &str = r#"
 _____                            _    
|  __ \                          | |   
| |  | | ___  ___ _ __ ___   __ _| | __
| |  | |/ _ \/ _ \ '_ ` _ \ / _` | |/ /
| |__| |  __/  __/ | | | | | (_| |   < 
|_____/ \___|\___|_| |_| |_|\__,_|_|\_\
                                       
Developed by Databased Club, Indian Institute of Science, Bangalore.
Official Github Repo: https://github.com/databasedIISc/deemak
"#;

pub const INITIAL_MSG: &str = "Type commands and press Enter. Try `help` for more info.";

impl ShellScreen {
    pub fn new_world(rl: RaylibHandle, thread: RaylibThread) -> Self {
        let root_dir = utils::find_home().expect("Could not find sekai home directory");

        Self {
            rl,
            thread,
            input_buffer: String::new(),
            output_lines: vec![
                "Type commands and press Enter. Try `help` for more info.".to_string(),
            ],
            root_dir: root_dir.clone(),
            current_dir: root_dir,
        }
    }

    pub fn run(&mut self) {
        while !self.window_should_close() {
            self.update();
            self.draw();
        }
    }

    pub fn window_should_close(&self) -> bool {
        self.rl.window_should_close()
    }

    pub fn update(&mut self) {
        // Handle keyboard input
        match self.rl.get_key_pressed() {
            Some(KeyboardKey::KEY_ENTER) => {
                let input = std::mem::take(&mut self.input_buffer);
                self.process_input(&input);
            }
            Some(KeyboardKey::KEY_BACKSPACE) => {
                if !self.input_buffer.is_empty() {
                    self.input_buffer.pop();
                }
            }
            Some(key) => {
                // Only accept printable ASCII characters
                let shift = self.rl.is_key_down(KeyboardKey::KEY_LEFT_SHIFT)
                    || self.rl.is_key_down(KeyboardKey::KEY_RIGHT_SHIFT);

                if let Some(c) = key_to_char(key, shift) {
                    self.input_buffer.push(c);
                }
            }
            None => {}
        }
    }

    pub fn draw(&mut self) {
        let mut d = self.rl.begin_drawing(&self.thread);

        d.clear_background(Color::BLACK);

        // Draw output lines
        for (i, line) in self.output_lines.iter().enumerate() {
            d.draw_text(line, 10, 10 + (i as i32 * 20), 20, Color::WHITE);
        }

        // Draw input prompt and buffer
        d.draw_text(
            "> ",
            10,
            10 + (self.output_lines.len() as i32 * 20),
            20,
            Color::GREEN,
        );
        d.draw_text(
            &self.input_buffer,
            30,
            10 + (self.output_lines.len() as i32 * 20),
            20,
            Color::WHITE,
        );
    }

    pub fn process_input(&mut self, input: &str) {
        if input.is_empty() {
            return;
        }

        // Add input to output
        self.output_lines.push(format!("> {}", input));

        // Parse and execute command
        let parts: Vec<&str> = input.split_whitespace().collect();
        match commands::cmd_manager(&parts, &mut self.current_dir, &self.root_dir) {
            CommandResult::ChangeDirectory(new_dir, message) => {
                self.current_dir = new_dir;
                self.output_lines
                    .extend(message.split("\n").map(|s| s.to_string()));
            }
            CommandResult::Output(output) => {
                self.output_lines
                    .extend(output.split("\n").map(|s| s.to_string()));
            }
            CommandResult::Clear => {
                self.output_lines.clear();
                self.output_lines.push(INITIAL_MSG.to_string());
            }
            CommandResult::Exit => {
                std::process::exit(1);
            }
            CommandResult::NotFound => {
                self.output_lines
                    .push("Command not found. Try `help`.".to_string());
            }
        }
    }
}
