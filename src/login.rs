use raylib::ffi::{ColorFromHSV, DrawTextEx, LoadFontEx, MeasureTextEx, Vector2};
use raylib::prelude::*;
use std::ffi::CString;
use std::os::raw::{c_char, c_int};
use std::time::{Duration, Instant};

use crate::utils::globals::USER_ID;

pub fn show_login(rl: &mut RaylibHandle, thread: &RaylibThread, _font_size: f32) -> bool {
    let mut username = String::new();
    let mut warning = false;
    let mut alpha = 0.0f32;
    let mut y_offset = 300.0;
    let target_y = 200.0;

    // Load JetBrainsMono for the input box only
    let font = unsafe {
        let path = CString::new("fontbook/fonts/ttf/JetBrainsMono-Medium.ttf").unwrap();
        LoadFontEx(path.as_ptr(), 600, std::ptr::null_mut(), 0)
    };

    while !rl.window_should_close() {
        let input = rl.get_key_pressed();

        // Animate title position and fade-in
        alpha = (alpha + 0.02).min(1.0);
        y_offset += (target_y - y_offset) * 0.1;

        {
            let mut d = rl.begin_drawing(thread);
            d.clear_background(Color::BLACK);

            let highlight_color = Color::GOLD;
            let base_color = Color::new(200, 200, 200, (alpha * 200.0) as u8);
            let subtle_warning = Color::new(180, 100, 100, 120);

            // Draw animated title with default font
            d.draw_text(
                "DEEMAK SHELL",
                200,
                y_offset as i32,
                60,
                Color::fade(&Color::WHITE, alpha),
            );

            // Prompt and box
            let prompt = "Enter Username :";
            let prompt_x = 200.0;
            let prompt_y = 300.0;
            let box_x = prompt_x;
            let box_y = prompt_y + 50.0;
            let box_width = 300.0;
            let box_height = 40.0;

            d.draw_text(prompt, prompt_x as i32, (prompt_y + 5.0) as i32, 30, Color::fade(&Color::WHITE, 0.9));

            d.draw_rectangle_lines(
                box_x as i32,
                box_y as i32,
                box_width as i32,
                box_height as i32,
                highlight_color,
            );

            // Display cropped input using JetBrainsMono
            let mut visible = String::new();
            let mut total_width = 0.0;
            for ch in username.chars().rev() {
                let s = CString::new(ch.to_string()).unwrap();
                let w = unsafe { MeasureTextEx(font, s.as_ptr(), 30.0, 1.0).x };
                if total_width + w + 10.0 > box_width {
                    break;
                }
                total_width += w;
                visible.insert(0, ch);
            }

            unsafe {
                let visible_c = CString::new(visible).unwrap();
                DrawTextEx(
                    font,
                    visible_c.as_ptr(),
                    Vector2 { x: box_x + 5.0, y: box_y + 5.0 },
                    30.0,
                    0.1,
                    highlight_color.into(),
                );
            }

            // Show warning if spaces are entered
            // if warning {
            //     d.draw_text(
            //         "Spaces not allowed",
            //         prompt_x as i32,
            //         (box_y + 45.0) as i32,
            //         14,
            //         subtle_warning,
            //     );
            // }
        }

        // Handle input
        if let Some(key) = input {
            match key {
                KeyboardKey::KEY_ENTER => {
                    if !username.is_empty() {
                        USER_ID.set(username.clone()).ok();
                        return true;
                    }
                }
                KeyboardKey::KEY_BACKSPACE => {
                    username.pop();
                }
                KeyboardKey::KEY_SPACE => {
                    warning = true;
                }
                _ => {
                    let shift = rl.is_key_down(KeyboardKey::KEY_LEFT_SHIFT)
                        || rl.is_key_down(KeyboardKey::KEY_RIGHT_SHIFT);
                    if let Some(c) = key_to_char(key, shift) {
                        username.push(c);
                        warning = false;
                    }
                }
            }
        }
    }

    false
}

fn key_to_char(key: KeyboardKey, shift: bool) -> Option<char> {
    use KeyboardKey::*;
    let ch = match key {
        KEY_A => 'a', KEY_B => 'b', KEY_C => 'c', KEY_D => 'd',
        KEY_E => 'e', KEY_F => 'f', KEY_G => 'g', KEY_H => 'h',
        KEY_I => 'i', KEY_J => 'j', KEY_K => 'k', KEY_L => 'l',
        KEY_M => 'm', KEY_N => 'n', KEY_O => 'o', KEY_P => 'p',
        KEY_Q => 'q', KEY_R => 'r', KEY_S => 's', KEY_T => 't',
        KEY_U => 'u', KEY_V => 'v', KEY_W => 'w', KEY_X => 'x',
        KEY_Y => 'y', KEY_Z => 'z',
        KEY_ZERO => '0', KEY_ONE => '1', KEY_TWO => '2', KEY_THREE => '3',
        KEY_FOUR => '4', KEY_FIVE => '5', KEY_SIX => '6', KEY_SEVEN => '7',
        KEY_EIGHT => '8', KEY_NINE => '9',
        KEY_PERIOD => '.',
        KEY_MINUS => if shift { '_' } else { '-' },
        _ => return None,
    };

    Some(if shift && ch.is_ascii_alphabetic() {
        ch.to_ascii_uppercase()
    } else {
        ch
    })
}
