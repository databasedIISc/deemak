use crate::keys::key_to_char;
use commands::CommandResult;
use deemak::commands;
use deemak::utils;
use raylib::prelude::*;
use std::{path::PathBuf, mem::take, process::exit, os::raw::c_int};
use std::ffi::CString;
use std::os::raw::c_char;
use raylib::ffi::{DrawTextEx, LoadFontEx, MeasureTextEx, SetConfigFlags, Vector2, ColorFromHSV, DrawRectangle};
use textwrap::wrap;

// BUGGY: Wrap text to new lines,
// DONE: Make Deemak resizeable,
// DONE: Make the font size relative to the device.
// DONE: Change the font completely.
// BUGGY: Add a cursor

pub struct ShellScreen {
    rl: RaylibHandle,
    thread: RaylibThread,
    input_buffer: String,
    output_lines: Vec<String>,
    current_dir: PathBuf,
    root_dir: PathBuf,
    cursor_position: Vec<i32>,
    font: ffi::Font,
    grid_width: f32,
    window_width: i32,
}

pub const FONT_SIZE: f32 = 20.0;

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
    pub fn new() -> Self {
        unsafe {
            SetConfigFlags(4);
        }
        // Loading Font
        let font = unsafe {
            let path = CString::new("JetBrainsMono-2/fonts/ttf/JetBrainsMonoNL-Medium.ttf").unwrap();
            LoadFontEx(
                path.as_ptr() as *const c_char,
                FONT_SIZE as c_int,
                0 as *mut c_int,
                0
            )
        };

        let grid_width = unsafe {
            let a_char = CString::new("A").unwrap();
            MeasureTextEx(
                font,
                a_char.as_ptr() as *const c_char,
                FONT_SIZE,
                0.0
            ).x
        };

        let (rl, thread) = init()
            .size(800, 600)
            .title("DBD Deemak Shell")
            .build();

        let window_width = rl.get_screen_width();

        let root_dir = utils::find_home().expect("Sekai root directory not found");

        Self {
            rl,
            thread,
            input_buffer: String::new(),
            // output_lines: vec![DEEMAK_BANNER.to_string(), INITIAL_MSG.to_string()],
            output_lines: vec![INITIAL_MSG.to_string()],
            root_dir: root_dir.clone(),
            current_dir: root_dir, // Both point to same path initially
            cursor_position: vec![0, 0],
            font,
            grid_width,
            window_width,
        }
    }

    pub fn window_should_close(&self) -> bool {
        self.rl.window_should_close()
    }

    pub fn update(&mut self) {
        // Handle keyboard input
        match self.rl.get_key_pressed() {
            Some(KeyboardKey::KEY_ENTER) => {
                let input = take(&mut self.input_buffer);
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

        // Handle window re-size
        if self.rl.is_window_resized() {
            self.window_width = self.rl.get_screen_width();
        }
    }

    pub fn draw(&mut self) {
        let mut d = self.rl.begin_drawing(&self.thread);

        d.clear_background(Color::BLACK);

        // Draw output lines
        let mut extra_lines = 0;
        for (i, line) in self.output_lines.iter().enumerate() {
            let limit = (self.window_width / self.grid_width as i32) - 5;
            if line.len() as i32 > limit {
                let lines = wrap(line, limit as usize);
                for wrapped_line in lines {
                    unsafe {
                        let pos: Vector2 = Vector2{x: 10.0, y: 10.0 + ((i+extra_lines) as f32 * FONT_SIZE)};
                        let content = CString::new(wrapped_line.to_string()).unwrap();
                        DrawTextEx(
                            self.font,
                            content.as_ptr() as *const c_char,
                            pos,
                            FONT_SIZE,
                            1.0,
                            ColorFromHSV(0.0, 0.0, 1.0)
                        );
                    }
                    extra_lines += 1;
                }
            } else {
                unsafe {
                    let pos: Vector2 = Vector2{x: 10.0, y: 10.0 + ((i+extra_lines) as f32 * FONT_SIZE)};
                    let content = CString::new(line.as_str()).unwrap();
                    DrawTextEx(
                        self.font,
                        content.as_ptr() as *const c_char,
                        pos,
                        FONT_SIZE,
                        1.0,
                        ColorFromHSV(0.0, 0.0, 1.0)
                    );
                }
            }
        }

        // '>' at the beginning of every line
        unsafe {
            let pos: Vector2 = Vector2{x: 10.0, y: 10.0 + ((self.output_lines.len()+extra_lines) as f32 * FONT_SIZE)};
            let content = CString::new(">").unwrap();
            DrawTextEx(
                self.font,
                content.as_ptr() as *const c_char,
                pos,
                FONT_SIZE,
                1.0,
                ColorFromHSV(0.0, 0.0, 1.0)
            );
        }

        // Input
        for (i, char) in self.input_buffer.as_str().chars().enumerate() {
            unsafe {
                let pos: Vector2 = Vector2{x: 30.0 + (i as f32 * (2.5 + self.grid_width)), y: 10.0 + ((self.output_lines.len()+extra_lines) as f32 * FONT_SIZE)};
                let content = CString::new(char.to_string()).unwrap();
                DrawTextEx(
                    self.font,
                    content.as_ptr() as *const c_char,
                    pos,
                    FONT_SIZE,
                    1.0,
                    ColorFromHSV(0.0, 0.0, 1.0)
                );
            }
        }

        // CURSOR
        unsafe {
            DrawRectangle(
                (30.0 + (self.input_buffer.len() as f32 * (2.5 + self.grid_width))) as c_int,
                (10.0 + ((self.output_lines.len()+extra_lines) as f32 * FONT_SIZE)) as c_int,
                self.grid_width as c_int,
                (self.grid_width*2.5) as c_int,
                ColorFromHSV(10.0, 10.0, 1.0),
            );
        }
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
                exit(1);
            }
            CommandResult::NotFound => {
                self.output_lines
                    .push("Command not found. Try `help`.".to_string());
            }
        }
    }
}
