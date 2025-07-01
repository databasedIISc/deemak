use crate::keys::key_to_char;
use crate::utils::find_root;
use crate::utils::wrapit::wrapit;
use deemak::commands;
use deemak::commands::CommandResult;
use deemak::utils::prompt::UserPrompter;
use raylib::ffi::{
    ColorFromHSV, DrawLineEx, DrawRectangle, DrawTextEx, LoadFontEx, MeasureTextEx, Vector2,
};
use raylib::prelude::*;
use std::cmp::max;
use std::cmp::min;
use std::ffi::CString;
use std::os::raw::c_char;
use std::{mem::take, os::raw::c_int, path::PathBuf, process::exit};
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
    scroll_offset: i32,
    active_prompt: Option<String>,
}

impl UserPrompter for ShellScreen {
    fn confirm(&mut self, message: &str) -> bool {
        self.prompt_yes_no(message)
    }
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
    pub fn new_sekai(
        rl: RaylibHandle,
        thread: RaylibThread,
        world_dir: PathBuf,
        font_size: f32,
    ) -> Self {
        // Loading Font
        let font = unsafe {
            let path = CString::new("fontbook/fonts/ttf/JetBrainsMono-Medium.ttf").unwrap();

            LoadFontEx(
                path.as_ptr() as *const c_char,
                600.0 as c_int,
                std::ptr::null_mut::<c_int>(),
                0,
            )
        };

        let window_width = rl.get_screen_width();
        let window_height = rl.get_screen_height();
        let char_width = unsafe {
            let cstr = CString::new("W").unwrap();
            MeasureTextEx(font, cstr.as_ptr(), font_size, 1.2).x
        };
        let root_dir =
            find_root::find_home(&world_dir).expect("Could not find sekai home directory");

        Self {
            rl,
            thread,
            input_buffer: String::new(),
            output_lines: Vec::<String>::new(),
            root_dir: root_dir.clone(),
            current_dir: root_dir, // Both point to same path initially
            font,
            font_size,
            window_width,
            window_height,
            char_width,
            term_split_ratio: 2.0 / 3.0,
            scroll_offset: 0,
            active_prompt: None,
        }
    }

    pub fn run(&mut self) {
        //add to output lines the banner
        let limit: usize = ((self.window_width as f32 * (self.term_split_ratio - 0.12))
            / self.char_width)
            .floor() as usize;
        let wrapped_banner = wrap(DEEMAK_BANNER, limit);
        let wrapped_initial = wrap(INITIAL_MSG, limit);
        self.output_lines
            .extend(wrapped_banner.into_iter().map(|c| c.into_owned()));
        self.output_lines
            .extend(wrapped_initial.into_iter().map(|c| c.into_owned()));

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
                self.scroll_offset = 0;
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

        // Handle scroll
        let scroll_y = self.rl.get_mouse_wheel_move();
        if scroll_y != 0.0 {
            self.scroll_offset -= (scroll_y / 2.00) as i32;
        }
    }

    pub fn draw(&mut self) {
        // Draw output lines
        let char_width = unsafe {
            let cstr = CString::new("W").unwrap();
            MeasureTextEx(self.font, cstr.as_ptr(), self.font_size, 1.2).x
        };
        let limit = ((self.window_width as f32 * (self.term_split_ratio - 0.12)) / char_width)
            .floor() as usize;
        let max_lines_on_screen = self.window_height / self.font_size as i32;

        let mut visible_lines = Vec::<String>::new();
        for line in self.output_lines.iter() {
            let lines = if line.len() > limit {
                wrapit(line, limit)
            } else {
                vec![line.to_string()]
            };
            visible_lines.extend(lines);
        }

        // Scroll offset is negative or zero. Clamp it to valid range.
        let min_scroll_offset = -max(0, visible_lines.len() as i32 - max_lines_on_screen + 3);
        self.scroll_offset = max(self.scroll_offset, min_scroll_offset);
        self.scroll_offset = min(self.scroll_offset, 0); // Never go below bottom

        let mut d = self.rl.begin_drawing(&self.thread);
        d.clear_background(Color::BLACK);

        // Input
        // let input_lines = if self.input_buffer.len()+1 > limit {
        //     wrap(&self.input_buffer, limit)
        // } else {
        //     vec![Cow::Borrowed(self.input_buffer.as_str())]
        // };
        let input_line = if let Some(ref prompt) = self.active_prompt {
            format!("{} {}", prompt, self.input_buffer)
        } else {
            format!("> {}", self.input_buffer)
        };

        let input_lines: Vec<String> = {
            wrapit(&input_line, limit)
                .into_iter()
                .map(|line| line.to_owned())
                .collect()
        };

        let length_input: usize = input_lines.len();
        visible_lines.extend(input_lines.into_iter());

        let mut index: usize = 0;

        index = min(
            max(
                0,
                visible_lines.len() as i32 - max_lines_on_screen + self.scroll_offset + 3,
            ),
            visible_lines.len() as i32 - 1,
        ) as usize;
        let display_lines = &visible_lines[index as usize..];

        for (i, line) in display_lines.iter().enumerate() {
            unsafe {
                let pos: Vector2 = Vector2 {
                    x: 10.0,
                    y: 10.0 + (i as f32 * self.font_size),
                };
                let content = CString::new(line.to_string()).unwrap();
                DrawTextEx(
                    self.font,
                    content.as_ptr() as *const c_char,
                    pos,
                    self.font_size,
                    1.2,
                    ColorFromHSV(0.0, 0.0, 1.0),
                );
            }
        }
        //promt

        // '>' at the beginning of every line
        unsafe {
            let pos_cursr: Vector2 = Vector2 {
                x: 10.0,
                y: 10.0 + ((display_lines.len() - length_input) as f32 * self.font_size),
            };
            let content = CString::new(">").unwrap();

            DrawTextEx(
                self.font,
                content.as_ptr() as *const c_char,
                pos_cursr,
                self.font_size,
                1.2,
                ColorFromHSV(0.0, 0.0, 1.0),
            );
        }

        // CURSOR
        let cursor_line = display_lines.len() - 1;
        let cursor_x_offset = unsafe {
            let last_line = display_lines.last().unwrap();
            let c_string = CString::new(last_line.to_string()).unwrap();
            MeasureTextEx(self.font, c_string.as_ptr(), self.font_size, 1.2).x
        };

        unsafe {
            DrawRectangle(
                (10.0 + cursor_x_offset) as c_int,
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
                Vector2 {
                    x: self.window_width as f32 * divider_pos,
                    y: 0.0,
                },
                Vector2 {
                    x: self.window_width as f32 * divider_pos,
                    y: self.window_height as f32,
                },
                2.0,
                ColorFromHSV(0.0, 0.0, 0.3),
            );
        }
    }

    pub fn process_input(&mut self, mut input: &str) {
        if input.is_empty() {
            return;
        }

        // Add input to output
        self.output_lines.push(format!("> {}", input));

        // Parse and execute command
        let mut current_dir = self.current_dir.clone();
        let root_dir = self.root_dir.clone();
        let parts: Vec<&str> = input.split_whitespace().collect();
        match commands::cmd_manager(&parts, &mut current_dir, &root_dir, self) {
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

    pub fn prompt_yes_no(&mut self, message: &str) -> bool {
        self.active_prompt = Some(format!("{} [y/N]", message));
        self.input_buffer.clear();
        self.draw();

        loop {
            self.update();
            self.draw();

            if self.rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_Y) {
                self.active_prompt = None;
                self.output_lines.push(format!("{} [y/N] yes", message));
                return true;
            }
            if self.rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_N)
                || self
                    .rl
                    .is_key_pressed(raylib::consts::KeyboardKey::KEY_ENTER)
            {
                self.active_prompt = None;
                self.output_lines.push(format!("{} [y/N] no", message));
                return false;
            }
        }
    }
}
