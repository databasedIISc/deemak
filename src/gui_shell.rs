use crate::globals;
use crate::keys::key_to_char;
use crate::utils::tab_completion::{TabCompletionResult, process_tab_completion};
use crate::utils::{find_root, shell_history, wrapit::wrapit};
use deemak::commands;
use deemak::commands::CommandResult;
use deemak::commands::list_directory_entries;
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
    working_buffer: Option<String>,
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
    history_index: Option<usize>,
}

impl UserPrompter for ShellScreen {
    fn confirm(&mut self, message: &str) -> bool {
        self.prompt_yes_no(message)
    }
}

static mut FIRST_RUN: bool = true;
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
        sekai_dir: PathBuf,
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
            find_root::find_home(&sekai_dir).expect("Could not find sekai home directory");

        Self {
            rl,
            thread,
            input_buffer: String::new(),
            output_lines: Vec::<String>::new(),
            working_buffer: None,
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
            history_index: None,
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
                if !input.is_empty() {
                    self.process_shell_input(&input);
                    self.scroll_offset = 0;
                    shell_history::add_to_history(&input);
                    self.history_index = None;
                    self.working_buffer = None; // Clear working buffer after command execution
                } else {
                    // If input is empty, just add a new line
                    if !unsafe { FIRST_RUN } {
                        self.output_lines.push("> ".to_string());
                    } else {
                        unsafe { FIRST_RUN = false };
                    }
                }
            }
            Some(KeyboardKey::KEY_BACKSPACE) => {
                if !self.input_buffer.is_empty() {
                    self.input_buffer.pop();
                }
            }

            Some(KeyboardKey::KEY_TAB) => {
                // Get current command parts
                let parts: Vec<&str> = self.input_buffer.split_whitespace().collect();

                if parts.len() > 1 {
                    // Get the last part (what we're trying to complete)
                    let last_part = parts.last().unwrap();

                    // List directory contents
                    let (files, dirs) = list_directory_entries(&self.current_dir, &self.root_dir);
                    let all_matches = [dirs, files].concat();

                    // Find matches
                    let matches: Vec<String> = all_matches
                        .iter()
                        .filter(|&name| name.starts_with(last_part))
                        .cloned()
                        .collect();

                    // Calculate terminal dimensions
                    let term_width = ((self.window_width as f32 * (self.term_split_ratio - 0.12))
                        / self.char_width)
                        .floor() as usize;
                    let term_height = (self.window_height / self.font_size as i32) as usize;

                    // Process tab completion
                    let result = process_tab_completion(
                        parts,
                        matches,
                        term_width,
                        term_height,
                        &self.input_buffer,
                        self.active_prompt.as_deref(),
                    );

                    match result {
                        TabCompletionResult::SingleMatch(new_input) => {
                            self.input_buffer = new_input;
                        }
                        TabCompletionResult::CommonPrefix(new_input) => {
                            self.input_buffer = new_input;
                        }
                        TabCompletionResult::DisplayCompletions {
                            current_line,
                            completion_lines,
                            should_display_all,
                        } => {
                            self.output_lines.push(current_line);
                            if should_display_all {
                                if self.prompt_yes_no(&format!(
                                    "Display all {} possibilities? (y or n)",
                                    completion_lines.len()
                                )) {
                                    self.output_lines.extend(completion_lines);
                                }
                            } else {
                                self.output_lines.extend(completion_lines);
                            }
                            self.scroll_offset = 0;
                        }
                        TabCompletionResult::NoAction => {}
                    }
                }
            }
            Some(KeyboardKey::KEY_UP) => {
                // Save current buffer if we're starting history navigation
                if self.history_index.is_none() && !self.input_buffer.is_empty() {
                    self.working_buffer = Some(self.input_buffer.clone());
                }

                let history = shell_history::get_history();
                if !history.is_empty() {
                    let new_index = match self.history_index {
                        Some(index) if index > 0 => index - 1,
                        Some(index) => index,      // already at first item
                        None => history.len() - 1, // start from most recent
                    };
                    self.input_buffer = history[new_index].clone();
                    self.history_index = Some(new_index);
                }
            }
            Some(KeyboardKey::KEY_DOWN) => {
                if let Some(index) = self.history_index {
                    let history = shell_history::get_history();
                    if index < history.len() - 1 {
                        // Move to next item in history
                        let new_index = index + 1;
                        self.input_buffer = history[new_index].clone();
                        self.history_index = Some(new_index);
                    } else {
                        // Reached the end of history - restore working buffer
                        self.input_buffer = self.working_buffer.take().unwrap_or_default();
                        self.history_index = None;
                    }
                }
            }
            Some(key) => {
                let shift = self.rl.is_key_down(KeyboardKey::KEY_LEFT_SHIFT)
                    || self.rl.is_key_down(KeyboardKey::KEY_RIGHT_SHIFT);

                if let Some(c) = key_to_char(key, shift) {
                    self.input_buffer.push(c);
                    self.history_index = None;
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
        visible_lines.extend(input_lines);

        let mut index: usize;

        index = min(
            max(
                0,
                visible_lines.len() as i32 - max_lines_on_screen + self.scroll_offset + 3,
            ),
            visible_lines.len() as i32 - 1,
        ) as usize;
        let display_lines = &visible_lines[index..];

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

    pub fn process_input(&mut self, mut input: &str, prefix: Option<&str>) -> Vec<String> {
        if input.is_empty() {
            return self.output_lines.clone();
        }

        // Add input to output
        self.output_lines
            .push(format!("{} {}", prefix.unwrap_or(""), input));

        self.output_lines.clone()
    }

    pub fn process_shell_input(&mut self, input: &str) {
        // If input is empty, do nothing
        if input.trim().is_empty() {
            return;
        }
        self.output_lines = self.process_input(input, Some(">"));

        // Parse and execute command
        let mut current_dir = self.current_dir.clone();
        let root_dir = self.root_dir.clone();
        let parts: Vec<&str> = input.split_whitespace().collect();
        match commands::cmd_manager(&parts, &current_dir, &root_dir, self) {
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
