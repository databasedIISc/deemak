use crate::keys::key_to_char;
use crate::utils::auth::verify_password;
use crate::utils::globals::{USER_NAME, USER_PASSWORD};
use deemak::utils::auth::load_users;
use raylib::ffi::{DrawTextEx, LoadFontEx, MeasureTextEx, SetExitKey, Vector2};
use raylib::prelude::*;
use std::ffi::CString;
use std::time::{Duration, Instant};

pub fn show_login(rl: &mut RaylibHandle, thread: &RaylibThread, _font_size: f32) -> bool {
    let mut username = String::new();
    let mut password = String::new();
    let mut entering_username = true;
    let mut warning_text = String::new();
    let mut warning = false;
    let mut alpha = 0.0f32;
    let top_y = 100.0;
    let mut y_offset = top_y + 120.0;
    let target_y = top_y + 20.0;
    let full_text = "WELCOME TO";
    let mut displayed_text = String::new();
    let mut stream_index = 0;
    let stream_delay = Duration::from_millis(80);
    let mut last_stream_time = Instant::now();
    let mut animation_done = false;
    let mut show_input = false;
    let mut pause_start = None;

    unsafe {
        SetExitKey(0i32); // Disable exit key (ESC) to prevent accidental exit during login
    }

    let font = unsafe {
        let path = CString::new("fontbook/fonts/ttf/JetBrainsMono-Medium.ttf").unwrap();
        LoadFontEx(path.as_ptr(), 600, std::ptr::null_mut(), 0)
    };

    let font_d = rl.get_font_default();

    while !rl.window_should_close() {
        let input = rl.get_key_pressed();

        // Animate welcome text
        if stream_index >= full_text.len() {
            alpha = (alpha + 0.02).min(1.0);
            y_offset += (target_y - y_offset) * 0.1;
            let _y: f32 = y_offset - target_y;
            if _y.abs() < 1.0 && !animation_done {
                animation_done = true;
                pause_start = Some(Instant::now());
            }
        }

        // Pause before login input
        if animation_done && !show_input {
            if let Some(start) = pause_start {
                if start.elapsed() >= Duration::from_secs(0) {
                    show_input = true;
                }
            }
        }

        // Input Handling
        if show_input {
            if let Some(key) = input {
                match key {
                    KeyboardKey::KEY_ENTER => {
                        if entering_username {
                            if !username.is_empty() {
                                entering_username = false;
                                warning = false;
                            }
                        } else if !password.is_empty() {
                            USER_NAME.set(username.clone()).ok();
                            USER_PASSWORD.set(password.clone()).ok();
                            let users: Vec<deemak::utils::auth::User> = load_users();
                            let username: String = username.trim().to_string();
                            let password: String = password.trim().to_string();
                            if let Some(user) = users.iter().find(|u| u.username == username) {
                                if verify_password(&password, &user.salt, &user.password_hash) {
                                    return true;
                                } else {
                                    warning = true;
                                    warning_text = "Invalid password!".to_string();
                                }
                            } else {
                                warning = true;
                                warning_text = "Username not found!".to_string();
                            }
                        }
                    }
                    KeyboardKey::KEY_BACKSPACE => {
                        if entering_username {
                            username.pop();
                        } else {
                            password.pop();
                        }
                    }
                    KeyboardKey::KEY_SPACE => {
                        if entering_username {
                            username.push(' ');
                        } else {
                            password.push(' ');
                        }
                    }
                    KeyboardKey::KEY_DOWN | KeyboardKey::KEY_UP | KeyboardKey::KEY_TAB => {
                        entering_username = !entering_username;
                    }
                    _ => {
                        let shift = rl.is_key_down(KeyboardKey::KEY_LEFT_SHIFT)
                            || rl.is_key_down(KeyboardKey::KEY_RIGHT_SHIFT);
                        if let Some(c) = key_to_char(key, shift) {
                            if entering_username {
                                username.push(c);
                            } else {
                                password.push(c);
                            }
                            // warning = false;
                        }
                    }
                }
            }
        }

        // Drawing
        {
            let mut d = rl.begin_drawing(thread);
            d.clear_background(Color::BLACK);

            let highlight_color = Color::GOLD;

            // Streamed welcome text
            if stream_index < full_text.len() && last_stream_time.elapsed() >= stream_delay {
                stream_index += 1;
                displayed_text = full_text[..stream_index].to_string();
                last_stream_time = Instant::now();
            }

            if stream_index > 0 {
                d.draw_text(
                    &displayed_text,
                    200,
                    top_y as i32,
                    20,
                    Color::alpha(&Color::GRAY, 1.0),
                );
            }

            // Shell Title
            if stream_index >= full_text.len() {
                d.draw_text_ex(
                    &font_d,
                    "DEEMAK SHELL",
                    Vector2 {
                        x: 200.0,
                        y: y_offset,
                    },
                    60.0,
                    2.0,
                    Color::WHITE,
                );
            }

            if show_input {
                let base_x = 200.0;
                let user_y = top_y + 120.0;
                let pass_y = user_y + 100.0;
                let box_width = 320.0;
                let box_height = 40.0;

                // Draw warning if any
                if warning {
                    d.draw_text(
                        &warning_text,
                        base_x as i32,
                        (pass_y + 90.0) as i32,
                        20,
                        Color::RED,
                    );
                }

                // Username
                d.draw_text(
                    "Username :",
                    base_x as i32,
                    (user_y + 5.0) as i32,
                    30,
                    Color::alpha(&Color::WHITE, 0.9),
                );
                d.draw_rectangle_lines(
                    (base_x) as i32,
                    (user_y + 40.0) as i32,
                    box_width as i32,
                    box_height as i32,
                    if entering_username {
                        highlight_color
                    } else {
                        Color::GRAY
                    },
                );

                // Draw username
                let mut total_width = 0.0;
                let mut visible = String::new();
                for ch in username.chars().rev() {
                    let s = CString::new(ch.to_string()).unwrap();
                    let w = unsafe { MeasureTextEx(font, s.as_ptr(), 30.0, 1.0).x };
                    if total_width + w + 10.0 > box_width {
                        break;
                    }
                    total_width += w;
                    visible.insert(0, ch);
                }
                let user_display = if entering_username {
                    format!("{visible}|")
                } else {
                    visible.clone()
                };
                let user_c = CString::new(user_display).unwrap();
                unsafe {
                    DrawTextEx(
                        font,
                        user_c.as_ptr(),
                        Vector2 {
                            x: base_x + 5.0,
                            y: user_y + 45.0,
                        },
                        30.0,
                        0.1,
                        highlight_color.into(),
                    );
                }

                // Password
                d.draw_text(
                    "Password :",
                    base_x as i32,
                    (pass_y + 5.0) as i32,
                    30,
                    Color::alpha(&Color::WHITE, 0.9),
                );
                d.draw_rectangle_lines(
                    base_x as i32,
                    (pass_y + 40.0) as i32,
                    box_width as i32,
                    box_height as i32,
                    if !entering_username {
                        highlight_color
                    } else {
                        Color::GRAY
                    },
                );

                // Draw masked password
                let masked = "*".repeat(password.len());
                let mut total_width = 0.0;
                let mut visible_masked = String::new();
                for ch in masked.chars().rev() {
                    let s = CString::new(ch.to_string()).unwrap();
                    let w = unsafe { MeasureTextEx(font, s.as_ptr(), 30.0, 1.0).x };
                    if total_width + w + 10.0 > box_width {
                        break;
                    }
                    total_width += w;
                    visible_masked.insert(0, ch);
                }
                let pass_display = if !entering_username {
                    format!("{visible_masked}|")
                } else {
                    visible_masked.clone()
                };
                let pass_c = CString::new(pass_display).unwrap();
                unsafe {
                    DrawTextEx(
                        font,
                        pass_c.as_ptr(),
                        Vector2 {
                            x: base_x + 5.0,
                            y: pass_y + 45.0,
                        },
                        30.0,
                        0.1,
                        highlight_color.into(),
                    );
                }

                // Divider line
                let screen_width = d.get_screen_width();
                let divider_y = pass_y + 150.0;
                d.draw_line(
                    30,
                    divider_y as i32,
                    screen_width - 30,
                    divider_y as i32,
                    Color::alpha(&Color::GRAY, 0.5),
                );

                // Footer note aligned to
                let footer_note = "Created by DataBased Club, IISc Bengaluru.
                Enter your username and password to log in. Use up/down keys to switch input.";
                let max_width = screen_width as f32 - 40.0;
                let font_size = 20.0;
                let spacing = 0.1;
                let mut x = 40.0;
                let mut y = divider_y + 10.0;

                let words: Vec<&str> = footer_note.split_whitespace().collect();
                let mut line = String::new();

                for word in words {
                    let trial = if line.is_empty() {
                        word.to_string()
                    } else {
                        format!("{line} {word}")
                    };
                    let trial_c = CString::new(trial.clone()).unwrap();
                    let width =
                        unsafe { MeasureTextEx(font, trial_c.as_ptr(), font_size, spacing).x };

                    if width > max_width {
                        // Draw current line
                        let line_c = CString::new(line.clone()).unwrap();
                        unsafe {
                            DrawTextEx(
                                font,
                                line_c.as_ptr(),
                                Vector2 { x, y },
                                font_size,
                                spacing,
                                Color::alpha(&Color::GRAY, 0.5).into(),
                            );
                        }
                        // Start new line
                        line = word.to_string();
                        y += font_size + 5.0; // line spacing
                    } else {
                        line = trial;
                    }
                }

                // Draw last line
                if !line.is_empty() {
                    let line_c = CString::new(line).unwrap();
                    unsafe {
                        DrawTextEx(
                            font,
                            line_c.as_ptr(),
                            Vector2 { x, y },
                            font_size,
                            spacing,
                            Color::alpha(&Color::GRAY, 0.5).into(),
                        );
                    }
                }

                // Version text
                let version = "Version 1.0";
                d.draw_text(
                    version,
                    10,
                    d.get_screen_height() - 30,
                    16,
                    Color::alpha(&Color::GRAY, 0.4),
                );
            }
        }
    }

    false
}
