use raylib::prelude::*;
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

pub fn show_about(rl: &mut RaylibHandle, thread: &RaylibThread, debug_mode: bool) {
    let mut displayed_text = String::new();
    let mut char_index = 0;
    let mut last_char_time = Instant::now();
    let char_delay = Duration::from_millis(5);
    let mut should_exit = false;
    let mut input_ready = false;

    // Load font
    let font = rl.get_font_default();

    while !rl.window_should_close() && !should_exit {

        // Wait before the next input
        if !input_ready {
            if !rl.is_key_down(KeyboardKey::KEY_ENTER)
                && !rl.is_key_down(KeyboardKey::KEY_SPACE)
                && !rl.is_key_down(KeyboardKey::KEY_ESCAPE)
            {
                input_ready = true;
            }
        } else {
            if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
                if char_index < ABOUT_TEXT.len() {
                    // Skip animation
                    displayed_text = ABOUT_TEXT.to_string();
                    char_index = ABOUT_TEXT.len();
                } else {
                    // Exit after animation complete
                    should_exit = true;
                }
            } else if char_index >= ABOUT_TEXT.len()
                && (rl.is_key_pressed(KeyboardKey::KEY_ENTER) || rl.is_key_pressed(KeyboardKey::KEY_ESCAPE))
            {
                // Exit after animation complete
                should_exit = true;
            }
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
        let text_pos = Vector2::new(50.0, 50.0);
        let mut y_offset = 0.0;
        let line_height = 30.0;

        for line in displayed_text.lines() {
            d.draw_text_ex(
                &font,
                line,
                text_pos + Vector2::new(0.0, y_offset),
                24.0,
                1.0,
                Color::WHITE,
            );
            y_offset += line_height;
        }

        // Draw prompt to continue and skip
        if char_index >= ABOUT_TEXT.len() {
            let prompt = "Press ESC/ENTER/SPACE to continue";
            // let prompt_width = measure_text_ex(&font, prompt, 20.0, 1.0).x as i32;
            let prompt_width = d.measure_text(prompt, 24);
            d.draw_text(prompt, (800 - prompt_width) / 2, 550, 20, Color::GRAY);
        } else {
            let skip_prompt = "Press SPACE to skip";
            let skip_width = d.measure_text(skip_prompt, 20);
            d.draw_text(skip_prompt, (800 - skip_width) / 2, 550, 20, Color::GRAY);
        }
    }
}