use raylib::ffi::{ColorFromHSV, DrawTextEx, LoadFontEx, MeasureTextEx, Vector2};
use raylib::prelude::*;
use std::ffi::CString;
use std::os::raw::{c_char, c_int};
use std::time::{Duration, Instant};

const ABOUT_TEXT: &str = r#"
DEEMAK Shell - Version 1.0

A modern terminal shell built with Rust and Raylib

Features:
- Command parsing
- Directory navigation
- File operations
- Customizable interface

Created by: IISc Databased Club
License: MIT
"#;

pub fn show_about(rl: &mut RaylibHandle, thread: &RaylibThread) {
    let mut displayed_text = String::new();
    let mut char_index = 0;
    let mut last_char_time = Instant::now();
    let char_delay = Duration::from_millis(5);
    let mut should_exit = false;
    let mut input_ready = false;

    // Load font
    let font = unsafe {
        let path = CString::new("fontbook/fonts/ttf/JetBrainsMono-Medium.ttf").unwrap();
        LoadFontEx(
            path.as_ptr() as *const c_char,
            600.0 as c_int,
            0 as *mut c_int,
            0,
        )
    };

    while !rl.window_should_close() && !should_exit {
        // Wait before the next input
        if !input_ready {
            if !rl.is_key_down(KeyboardKey::KEY_ENTER)
                && !rl.is_key_down(KeyboardKey::KEY_SPACE)
                && !rl.is_key_down(KeyboardKey::KEY_ESCAPE)
            {
                input_ready = true;
            }
        } else if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
            if char_index < ABOUT_TEXT.len() {
                // Skip animation
                displayed_text = ABOUT_TEXT.to_string();
                char_index = ABOUT_TEXT.len();
            } else {
                // Exit after animation complete
                should_exit = true;
            }
        } else if char_index >= ABOUT_TEXT.len()
            && (rl.is_key_pressed(KeyboardKey::KEY_ENTER)
                || rl.is_key_pressed(KeyboardKey::KEY_ESCAPE))
        {
            // Exit after animation complete
            should_exit = true;
        }

        // Typewriter animation
        if char_index < ABOUT_TEXT.len() && last_char_time.elapsed() >= char_delay {
            displayed_text.push(ABOUT_TEXT.chars().nth(char_index).unwrap());
            char_index += 1;
            last_char_time = Instant::now();
        }

        // Draw
        let mut d = rl.begin_drawing(thread);
        d.clear_background(Color::BLACK);

        // Draw animated text
        let mut y_offset = 0.0;
        let line_height = 30.0;

        for line in displayed_text.lines() {
            unsafe {
                let content = CString::new(line).unwrap();
                let pos = Vector2 {
                    x: 50.0,
                    y: 50.0 + y_offset,
                };
                DrawTextEx(
                    font,
                    content.as_ptr() as *const c_char,
                    pos,
                    24.0,
                    1.0,
                    ColorFromHSV(0.0, 0.0, 1.0),
                );
            }
            y_offset += line_height;
        }

        // Draw prompt to continue and skip
        if char_index >= ABOUT_TEXT.len() {
            let prompt = "Press ESC/ENTER/SPACE to continue";
            // let prompt_width = measure_text_ex(&font, prompt, 20.0, 1.0).x as i32;
            unsafe {
                let c_prompt = CString::new(prompt).unwrap();
                let prompt_width = MeasureTextEx(font, c_prompt.as_ptr(), 20.0, 1.0).x;
                let content = CString::new(prompt).unwrap();
                let pos = Vector2 {
                    x: (800.0 - prompt_width) / 2.0,
                    y: 550.0,
                };
                DrawTextEx(
                    font,
                    content.as_ptr() as *const c_char,
                    pos,
                    20.0,
                    1.0,
                    ColorFromHSV(0.0, 0.0, 0.51),
                );
            }
        } else {
            let skip_prompt = "Press SPACE to skip";
            unsafe {
                let c_skip = CString::new(skip_prompt).unwrap();
                let skip_width = MeasureTextEx(font, c_skip.as_ptr(), 20.0, 1.0).x;
                let content = CString::new(skip_prompt).unwrap();
                let pos = Vector2 {
                    x: (600.0 - skip_width) / 2.0,
                    y: 550.0,
                };
                DrawTextEx(
                    font,
                    content.as_ptr() as *const c_char,
                    pos,
                    20.0,
                    1.0,
                    ColorFromHSV(0.0, 0.0, 0.51),
                );
            }
        }
    }
}
