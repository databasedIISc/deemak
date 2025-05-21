use crate::keys::key_to_char;
use commands::CommandResult;
use deemak::commands;
use deemak::utils;
use raylib::prelude::*;
use std::{path::PathBuf, mem::take, process::exit, os::raw::c_int};
use std::borrow::Cow;
use std::ffi::CString;
use std::os::raw::c_char;
use raylib::ffi::{DrawTextEx, DrawLineEx, LoadFontEx, MeasureTextEx, Vector2, ColorFromHSV, DrawRectangle};
use textwrap::wrap;

pub struct ShellScreen {
    rl: RaylibHandle,
    thread: RaylibThread,
    input_buffer: String,
    output_lines: Vec<String>,
    current_dir: PathBuf,
    root_dir: PathBuf,
    font: ffi::Font,
    window_width: i32,
    window_height: i32,
    char_width: f32,
    term_split_ratio: f32,
    font_size: f32,
    debug_mode: bool,
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
    pub fn new_world(rl: RaylibHandle, thread: RaylibThread, font_size: f32, debug_mode: bool) -> Self {
        // Loading Font
        let font = unsafe {
            let path = CString::new("JetBrainsMono-2/fonts/ttf/JetBrainsMono-Medium.ttf").unwrap();
            LoadFontEx(
                path.as_ptr() as *const c_char,
                600.0 as c_int,
                0 as *mut c_int,
                0
            )
        };

        let window_width = rl.get_screen_width();
        let window_height = rl.get_screen_height();
        let char_width = unsafe {
            let cstr = CString::new("W").unwrap();
            MeasureTextEx(font, cstr.as_ptr(), font_size, 1.2).x
        };
        let root_dir = utils::find_home().expect("Could not find sekai home directory");

        Self {
            rl,
            thread,
            input_buffer: String::new(),
            output_lines: vec![
                DEEMAK_BANNER.to_string(),
                "Type commands and press Enter. Try `help` for more info.".to_string(),
            ],
            root_dir: root_dir.clone(),
            current_dir: root_dir, // Both point to same path initially
            font,
            window_width,
            window_height,
            char_width,
            term_split_ratio: 2.0/3.0,
            font_size,
            debug_mode,
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
        let char_width = unsafe {
            let cstr = CString::new("W").unwrap();
            MeasureTextEx(self.font, cstr.as_ptr(), self.font_size, 1.2).x
        };
        let limit = ((self.window_width as f32 * (self.term_split_ratio - 0.12) ) / char_width).floor() as usize;
        let max_lines_on_screen = self.window_height / self.font_size as i32;

        let mut visible_lines = Vec::new();
        for line in self.output_lines.iter() {

            let lines = if line.len() > limit {
                wrap(line, limit)
            } else {
                vec![Cow::Borrowed(line.as_str())]
            };
            visible_lines.extend(lines);
        }

        if visible_lines.len() > max_lines_on_screen as usize {
            let neg_index = visible_lines.len() - max_lines_on_screen as usize + 5;
            visible_lines = visible_lines[neg_index..].to_owned();
        }

        for (i, line) in visible_lines.iter().enumerate() {
            unsafe {
                let pos: Vector2 = Vector2{x: 10.0, y: 10.0 + (i as f32 * self.font_size)};
                let content = CString::new(line.to_string()).unwrap();
                DrawTextEx(
                    self.font,
                    content.as_ptr() as *const c_char,
                    pos,
                    self.font_size,
                    1.2,
                    ColorFromHSV(0.0, 0.0, 1.0)
                );
            }
        }

        // '>' at the beginning of every line
        unsafe {
            let pos: Vector2 = Vector2{x: 10.0, y: 10.0 + (visible_lines.len() as f32 * self.font_size)};
            let content = CString::new(">").unwrap();

            DrawTextEx(
                self.font,
                content.as_ptr() as *const c_char,
                pos,
                self.font_size,
                1.2,
                ColorFromHSV(0.0, 0.0, 1.0)
            );
        }

        // Input
        let input_lines = if self.input_buffer.len() > limit {
            wrap(&self.input_buffer, limit)
        } else {
            vec![Cow::Borrowed(self.input_buffer.as_str())]
        };

        for (i, input_line) in input_lines.iter().enumerate() {
            unsafe {
                let pos: Vector2 = Vector2{x: 30.0, y: 10.0 + ((visible_lines.len() + i) as f32 * self.font_size)};
                let content = CString::new(input_line.to_string()).unwrap();

                DrawTextEx(
                    self.font,
                    content.as_ptr() as *const c_char,
                    pos,
                    self.font_size,
                    1.2,
                    ColorFromHSV(0.0, 0.0, 1.0)
                );
            }
        }

        // CURSOR
        let cursor_line = visible_lines.len() + input_lines.len() - 1;
        let cursor_x_offset = unsafe {
            let last_line = input_lines.last().unwrap();
            let c_string = CString::new(last_line.to_string()).unwrap();
            MeasureTextEx(self.font, c_string.as_ptr(), self.font_size, 1.2).x
        };

        unsafe {
            DrawRectangle(
                (30.0 + cursor_x_offset) as c_int,
                (10.0 + (cursor_line as f32 * self.font_size)) as c_int,
                char_width as c_int,
                self.font_size as c_int,
                ColorFromHSV(0.0, 0.0, 1.0),
            );
        }

        // DIVIDER
        let divider_pos = self.term_split_ratio;
        unsafe {
            DrawLineEx(
                Vector2{x: self.window_width as f32 * divider_pos, y: 0.0},
                Vector2{x: self.window_width as f32 * divider_pos, y: self.window_height as f32},
                2.0,
                ColorFromHSV(0.0, 0.0, 0.3)
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
