use crate::commands::cmds::{CommandResult, cmd_manager};
use crate::commands::ls::list_directory_entries;
use crate::keys::key_to_char;
use crate::menu;
use crate::menu::menu_options::MenuOption;
use crate::metainfo::info_reader::read_validate_info;
use crate::utils::config::{self, FONT_OPTIONS};
use crate::utils::tab_completion::{TabCompletionResult, process_tab_completion};
use crate::utils::{find_root, shell_history, wrapit::wrapit};
use crate::utils::{log, prompt::UserPrompter};
use raylib::ffi::{
    ColorFromHSV, DrawLineEx, DrawRectangle, DrawTextEx, LoadFontEx, MeasureTextEx, SetExitKey,
    Vector2,
};
use raylib::prelude::*;
use std::cmp::max;
use std::cmp::min;
use std::ffi::CString;
use std::os::raw::c_char;
use std::{
    mem::take,
    os::raw::c_int,
    path::{Path, PathBuf},
};
use textwrap::wrap;

pub struct ShellScreen<'a> {
    rl: &'a mut RaylibHandle,
    thread: &'a RaylibThread,
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
    cursor_pos: usize,
    selection_start: Option<(usize, usize)>, // (line_index, char_index)
    selection_end: Option<(usize, usize)>,
    mouse_dragging: bool,
}

impl UserPrompter for ShellScreen<'_> {
    fn confirm(&mut self, message: &str) -> bool {
        self.prompt_yes_no(message)
    }
    fn input(&mut self, message: &str) -> String {
        self.prompt_input_text(message)
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

impl<'a> ShellScreen<'a> {
    pub fn new_sekai(
        rl: &'a mut RaylibHandle,
        thread: &'a RaylibThread,
        sekai_dir: PathBuf,
        font_size: f32,
    ) -> Self {
        // Load font index from config
        let font_index = config::load_config().font_index;
        let font_path = FONT_OPTIONS
            .get(font_index)
            .map(|(_, path)| *path)
            .unwrap_or("fontbook/fonts/ttf/JetBrainsMono-Medium.ttf");
        // Loading Font
        let font = unsafe {
            let path = CString::new(font_path).unwrap();

            LoadFontEx(
                path.as_ptr() as *const c_char,
                600.0 as c_int,
                std::ptr::null_mut::<c_int>(),
                0,
            )
        };
        unsafe {
            SetExitKey(0i32); // No exit key
        }
        let window_width = rl.get_screen_width();
        let window_height = rl.get_screen_height();
        let char_width = unsafe {
            let cstr = CString::new("W").unwrap();
            MeasureTextEx(font, cstr.as_ptr(), font_size, 1.2).x
        };
        let root_dir =
            find_root::get_home(&sekai_dir).expect("Could not find sekai home directory");
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
            cursor_pos: 0,
            selection_start: None,
            selection_end: None,
            mouse_dragging: false,
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

        if unsafe { FIRST_RUN } {
            let info_path = self.root_dir.join(".dir_info").join("info.json");
            let home_about = read_validate_info(&info_path).ok().map(|info| info.about);
            let mut home_about =
                home_about.unwrap_or_else(|| "Welcome User to Deemak!".to_string());
            home_about = "\nYou are in 'HOME'\n\nAbout:\n".to_string() + &home_about + "\n";
            let wrapped_home_about = wrap(&home_about, limit);
            unsafe { FIRST_RUN = false };
            self.output_lines
                .extend(wrapped_home_about.into_iter().map(|c| c.into_owned()));
        }

        while !self.window_should_close() {
            self.update();
            self.draw();
        }
    }

    pub fn window_should_close(&self) -> bool {
        self.rl.window_should_close()
    }

    pub fn update(&mut self) {
        // MOUSE START
        // Handle mouse input for text selection
        let mouse_pos = self.rl.get_mouse_position();

        // Check if mouse is in the text area
        let in_text_area = mouse_pos.x < self.window_width as f32 * self.term_split_ratio
            && mouse_pos.y < self.window_height as f32;

        if in_text_area
            && self
                .rl
                .is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT)
        {
            // Start new selection
            if let Some((line_idx, char_idx)) = self.get_char_index_at_pos(mouse_pos.into()) {
                // log::log_info(
                //     "Deemak",
                //     &format!("Mouse: line {}, char {}", line_idx, char_idx),
                // );
                self.selection_start = Some((line_idx, char_idx));
                self.selection_end = Some((line_idx, char_idx));
                self.mouse_dragging = true;
            }
        }

        if self.mouse_dragging && self.rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
            // Update selection end while dragging
            if let Some((line_idx, char_idx)) = self.get_char_index_at_pos(mouse_pos.into()) {
                self.selection_end = Some((line_idx, char_idx));
            }
        }

        // Stop dragging when mouse button is released
        if self
            .rl
            .is_mouse_button_released(MouseButton::MOUSE_BUTTON_LEFT)
        {
            self.mouse_dragging = false;
        }
        // MOUSE END

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
                self.cursor_pos = 0; // Reset cursor position
            }
            Some(KeyboardKey::KEY_BACKSPACE) => {
                if !self.input_buffer.is_empty() && self.cursor_pos > 0 {
                    self.input_buffer.remove(self.cursor_pos - 1);
                    self.cursor_pos -= 1;
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
                self.cursor_pos = self.input_buffer.len(); // Move cursor to end after tab
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
                self.cursor_pos = self.input_buffer.len(); //place at the end of the command 
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
                    self.cursor_pos = self.input_buffer.len();
                }
            }
            Some(KeyboardKey::KEY_LEFT) => {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                }
            }
            Some(KeyboardKey::KEY_RIGHT) => {
                if self.cursor_pos < self.input_buffer.len() {
                    self.cursor_pos += 1;
                }
            }
            Some(key) => {
                let ctrl_pressed = self.rl.is_key_down(KeyboardKey::KEY_LEFT_CONTROL)
                    || self.rl.is_key_down(KeyboardKey::KEY_RIGHT_CONTROL);

                if ctrl_pressed {
                    match key {
                        KeyboardKey::KEY_K => {
                            // Clear input buffer and reset history index
                            self.output_lines.clear();
                            self.output_lines.push(INITIAL_MSG.to_string());
                            self.working_buffer = None;
                            self.cursor_pos = 0;
                            // Clear selection
                            self.selection_start = None;
                            self.selection_end = None;
                        }
                        KeyboardKey::KEY_C => {
                            if let (Some(start), Some(end)) =
                                (self.selection_start, self.selection_end)
                            {
                                // Copy selected text
                                self.copy_selected_text(start, end);
                            } else {
                                // Next prompt (original behavior)
                                self.output_lines.push(format!("> {}", self.input_buffer));
                                self.working_buffer = None;
                                self.input_buffer.clear();
                                self.scroll_offset = 0;
                                self.cursor_pos = 0;
                            }
                        }
                        KeyboardKey::KEY_V => {
                            // Paste from clipboard
                            let clipboard_text = self.rl.get_clipboard_text().unwrap_or_default();
                            if !clipboard_text.is_empty() {
                                // Remove newlines and carriage returns
                                let filtered_text = clipboard_text.replace(['\n', '\r'], "");
                                self.input_buffer
                                    .insert_str(self.cursor_pos, &filtered_text);
                                self.cursor_pos += filtered_text.len();
                            }
                        }
                        _ => {}
                    }
                } else {
                    // Handle regular key input when Ctrl is not pressed
                    let shift = self.rl.is_key_down(KeyboardKey::KEY_LEFT_SHIFT)
                        || self.rl.is_key_down(KeyboardKey::KEY_RIGHT_SHIFT);

                    if let Some(c) = key_to_char(key, shift) {
                        self.input_buffer.insert(self.cursor_pos, c);
                        self.cursor_pos += 1;
                    }
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

    pub fn get_window_lines(&self) -> Vec<String> {
        let char_width = self.char_width; // Assuming char_width is stored in the struct
        let limit = ((self.window_width as f32 * (self.term_split_ratio - 0.12)) / char_width)
            .floor() as usize;

        let output_lines = &self.output_lines;
        let active_prompt = &self.active_prompt;
        let input_buffer = &self.input_buffer;

        // build the lines
        let mut all_lines = Vec::<String>::new();
        for line in output_lines.iter() {
            let lines = if line.len() > limit {
                wrapit(line, limit)
            } else {
                vec![line.to_string()]
            };
            all_lines.extend(lines);
        }

        // Add input lines
        let input_line = if let Some(prompt) = active_prompt {
            format!("{prompt} {input_buffer}")
        } else {
            format!("> {input_buffer}")
        };

        let input_lines: Vec<String> = wrapit(&input_line, limit)
            .into_iter()
            .map(|line| line.to_owned())
            .collect();

        all_lines.extend(input_lines);
        all_lines
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

        let mut d = self.rl.begin_drawing(self.thread);
        d.clear_background(Color::BLACK);

        // Input
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

        // MOUSE TEXT SELECTION START
        // This below is the same as get_window_lines function, but i wasnt able to use it directly
        if let (Some(start), Some(end)) = (self.selection_start, self.selection_end) {
            let (start, end) = if start <= end {
                (start, end)
            } else {
                (end, start)
            };

            let char_width = self.char_width;
            let font_size = self.font_size;

            // Get all lines (including wrapped ones)
            let limit = ((self.window_width as f32 * (self.term_split_ratio - 0.12)) / char_width)
                .floor() as usize;

            let mut all_lines = Vec::<String>::new();
            for line in self.output_lines.iter() {
                let lines = if line.len() > limit {
                    wrapit(line, limit)
                } else {
                    vec![line.to_string()]
                };
                all_lines.extend(lines);
            }

            // Add input lines
            let input_line = if let Some(ref prompt) = self.active_prompt {
                format!("{} {}", prompt, self.input_buffer)
            } else {
                format!("> {}", self.input_buffer)
            };

            let input_lines: Vec<String> = wrapit(&input_line, limit)
                .into_iter()
                .map(|line| line.to_owned())
                .collect();

            all_lines.extend(input_lines);

            // Calculate visible range based on scroll
            let visible_start = (-self.scroll_offset).max(0) as usize;
            let visible_end = (visible_start
                + (self.window_height / self.font_size as i32) as usize)
                .min(all_lines.len());

            const MOUSE_OFFSET: usize = 12;

            for line_idx in start.0..=end.0 {
                if line_idx >= all_lines.len()
                    || line_idx < visible_start
                    || line_idx >= visible_end
                {
                    continue;
                }

                let line = &all_lines[line_idx];
                let start_char = if line_idx == start.0 { start.1 } else { 0 };
                let end_char = if line_idx == end.0 {
                    end.1
                } else {
                    line.len() + MOUSE_OFFSET
                };

                if start_char < line.len() + MOUSE_OFFSET {
                    let start_x = 10.0 + (start_char as f32 * char_width);
                    let end_x = 10.0
                        + (end_char as f32 * char_width)
                            .min(10.0 + ((line.len()) as f32 * char_width))
                        + MOUSE_OFFSET as f32;
                    let y =
                        10.0 + ((line_idx as i32 - (-self.scroll_offset)) as f32 * self.font_size);

                    // Draw selection rectangle
                    unsafe {
                        DrawRectangle(
                            start_x as c_int,
                            y as c_int,
                            (end_x - start_x) as c_int,
                            self.font_size as c_int,
                            ColorFromHSV(210.0, 0.5, 0.5), // Blue selection color
                        );
                    }
                }
            }
        }
        // MOUSE TEXT SELECTION END

        // When drawing text, we need to ensure it appears above the selection
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
        let cursor_prefix = if let Some(ref prompt) = self.active_prompt {
            format!(">{prompt}")
        } else {
            "> ".to_string()
        };
        let cursor_text = if self.cursor_pos <= self.input_buffer.len() {
            format!("{}{}", cursor_prefix, &self.input_buffer[..self.cursor_pos])
        } else {
            format!("{}{}", cursor_prefix, &self.input_buffer)
        };
        let cursor_line =
            display_lines.len() - length_input + wrapit(&cursor_text, limit).len() - 1;
        let cursor_x_offset = unsafe {
            let c_string = CString::new(cursor_text).unwrap();
            (MeasureTextEx(self.font, c_string.as_ptr(), self.font_size, 1.2).x)
                % ((limit as f32 + 6.0) * char_width)
        };
        // Draw cursor
        unsafe {
            DrawRectangle(
                (10.6 + cursor_x_offset) as c_int,
                (10.0 + (cursor_line as f32 * self.font_size)) as c_int,
                (char_width as f32 * 1.2) as c_int,
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
        match cmd_manager(&parts, &current_dir, &root_dir, self) {
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
                run_gui_loop(self.rl, self.thread, self.root_dir.clone(), self.font_size);
            }
            CommandResult::NotFound => {
                self.output_lines
                    .push("Command not found. Try `help`.".to_string());
            }
        }
    }

    pub fn prompt_yes_no(&mut self, message: &str) -> bool {
        self.active_prompt = Some(format!("{message} [y/N]"));
        self.input_buffer.clear();
        self.draw();

        loop {
            self.update();
            self.draw();

            if self.rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_Y) {
                self.active_prompt = None;
                self.output_lines.push(format!("{message} [y/N] yes"));
                return true;
            }
            if self.rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_N)
                || self
                    .rl
                    .is_key_pressed(raylib::consts::KeyboardKey::KEY_ENTER)
            {
                self.active_prompt = None;
                self.output_lines.push(format!("{message} [y/N] no"));
                return false;
            }
        }
    }
    pub fn prompt_input_text(&mut self, message: &str) -> String {
        self.active_prompt = Some(message.to_string());
        self.input_buffer.clear();
        self.draw();
        let excess = self.cursor_pos;
        self.cursor_pos = 0;

        loop {
            self.update();
            self.draw();

            match self.rl.get_key_pressed() {
                Some(KeyboardKey::KEY_ENTER) => {
                    let input = take(&mut self.input_buffer);
                    self.active_prompt = None;
                    self.output_lines.push(format!("{message}: {input}"));
                    self.cursor_pos = 0;
                    return input;
                }
                Some(KeyboardKey::KEY_BACKSPACE) => {
                    if !self.input_buffer.is_empty() && self.cursor_pos > 0 {
                        self.input_buffer.remove(self.cursor_pos - 1);
                        self.cursor_pos -= 1;
                    }
                }
                Some(KeyboardKey::KEY_LEFT) => {
                    if self.cursor_pos > 0 {
                        self.cursor_pos -= 1;
                    }
                }
                Some(KeyboardKey::KEY_RIGHT) => {
                    if self.cursor_pos < self.input_buffer.len() {
                        self.cursor_pos += 1;
                    }
                }
                Some(key) => {
                    let shift = self.rl.is_key_down(KeyboardKey::KEY_LEFT_SHIFT)
                        || self.rl.is_key_down(KeyboardKey::KEY_RIGHT_SHIFT);

                    if let Some(c) = key_to_char(key, shift) {
                        self.input_buffer.insert(self.cursor_pos, c);
                        self.cursor_pos += 1;
                    }
                }
                None => {}
            }
        }
        // Reset cursor position after input
    }

    /// Helper method to get character index at screen position
    fn get_char_index_at_pos(&self, pos: Vector2) -> Option<(usize, usize)> {
        let all_lines = self.get_window_lines();
        // Calculate which line we're on (accounting for scroll offset)
        let line_index = ((pos.y - 10.0) / self.font_size).floor() as i32 + (-self.scroll_offset);
        if line_index < 0 || line_index >= all_lines.len() as i32 {
            return None;
        }
        let line_index = line_index as usize;

        // Calculate which character in the line
        let line = &all_lines[line_index];
        let char_index = ((pos.x - 10.0) / self.char_width).floor() as usize;
        let char_index = char_index.min(line.len());

        Some((line_index, char_index))
    }

    // Copy selected text to clipboard
    fn copy_selected_text(&mut self, start: (usize, usize), end: (usize, usize)) {
        let (start, end) = if start <= end {
            (start, end)
        } else {
            (end, start)
        };

        let all_lines = self.get_window_lines();
        // Build the selected text
        let mut selected_text = String::new();

        for line_idx in start.0..=end.0 {
            if line_idx >= all_lines.len() {
                break;
            }

            let line = &all_lines[line_idx];
            let start_char = if line_idx == start.0 { start.1 } else { 0 };
            let end_char = if line_idx == end.0 { end.1 } else { line.len() };

            if start_char < line.len() {
                let slice = &line[start_char..end_char.min(line.len())];
                selected_text.push_str(slice);

                if line_idx != end.0 {
                    selected_text.push('\n');
                }
            }
        }

        if !selected_text.is_empty() {
            let _ = self.rl.set_clipboard_text(&selected_text);
        }
    }
}

/// Runs the main GUI loop for the Sekai shell
pub fn run_gui_loop(
    rl: &mut RaylibHandle,
    thread: &RaylibThread,
    sekai_dir: PathBuf,
    font_size: f32,
) {
    loop {
        // Show main menu and get user selection
        let menu_selection = menu::show_menu(rl, thread);

        match menu_selection {
            Some(MenuOption::StartShell) => {
                // Shell mode
                unsafe { FIRST_RUN = true }; // Reset first run flag
                let mut shell = ShellScreen::new_sekai(rl, thread, sekai_dir.clone(), font_size);
                shell.run();
            }
            Some(MenuOption::About) => {
                // About screen
                menu::about::show_about(rl, thread);
                // After about screen closes, return to menu
                continue;
            }
            Some(MenuOption::Tutorial) => {
                // Tutorial screen
                let tutorial_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("_tutorial");
                log::log_info("Deemak", "Loading Tutorial");
                unsafe { FIRST_RUN = true }; // Reset first run flag
                let mut tutorial_shell =
                    ShellScreen::new_sekai(rl, thread, tutorial_dir, font_size);
                tutorial_shell.run();
                continue;
            }
            Some(MenuOption::Settings) => {
                // Settings screen
                menu::settings::show_settings(rl, thread);
                // After settings screen closes, return to menu
                continue;
            }
            Some(MenuOption::Exit) | None => {
                // Exit
                std::process::exit(0); // Exit the application
            }
        }
    }
}
