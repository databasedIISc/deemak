use crate::keys::key_to_char;
use crate::utils::auth::verify_password;
use crate::utils::globals::{USER_NAME, USER_PASSWORD};
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
    let mut reg_username = String::new();
    let mut reg_password = String::new();
    let mut reg_warning_text = String::new();
    let mut reg_warning = false;
    let mut entering_reg_username = true;
    let mut users = deemak::utils::auth::load_users();
    let mut active_tab = if users.is_empty() { 1 } else { 0 }; // 0: Login, 1: Register

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
    let mut alpha = 0.0f32;

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
                    KeyboardKey::KEY_TAB => {
                        active_tab = 1 - active_tab;
                        warning = false;
                        warning_text.clear();
                        reg_warning = false;
                        reg_warning_text.clear();
                        username.clear();
                        password.clear();
                        reg_username.clear();
                        reg_password.clear();
                        entering_username = active_tab == 0;
                        entering_reg_username = active_tab == 1;
                    }
                    KeyboardKey::KEY_BACKSPACE => {
                        if active_tab == 0 {
                            if entering_username {
                                username.pop();
                            } else {
                                password.pop();
                            }
                        } else {
                            if entering_reg_username {
                                reg_username.pop();
                                reg_warning_text.clear();
                                reg_warning = false;
                            } else {
                                reg_password.pop();
                                reg_warning_text.clear();
                                reg_warning = false;
                            }
                        }
                    }
                    KeyboardKey::KEY_SPACE => {
                        if active_tab == 0 {
                            if entering_username {
                                username.push(' ');
                            } else {
                                password.push(' ');
                            }
                        } else {
                            if entering_reg_username {
                                reg_username.push(' ');
                                reg_warning_text.clear();
                                reg_warning = false;
                            } else {
                                reg_password.push(' ');
                                reg_warning_text.clear();
                                reg_warning = false;
                            }
                        }
                    }
                    KeyboardKey::KEY_DOWN | KeyboardKey::KEY_UP => {
                        if active_tab == 0 {
                            entering_username = !entering_username;
                        } else {
                            entering_reg_username = !entering_reg_username;
                        }
                    }
                    _ => {
                        let shift = rl.is_key_down(KeyboardKey::KEY_LEFT_SHIFT)
                            || rl.is_key_down(KeyboardKey::KEY_RIGHT_SHIFT);
                        if let Some(c) = key_to_char(key, shift) {
                            if active_tab == 0 {
                                if entering_username {
                                    username.push(c);
                                } else {
                                    password.push(c);
                                }
                            } else {
                                if entering_reg_username {
                                    reg_username.push(c);
                                    reg_warning_text.clear();
                                    reg_warning = false;
                                } else {
                                    reg_password.push(c);
                                    reg_warning_text.clear();
                                    reg_warning = false;
                                }
                            }
                        } else if key == KeyboardKey::KEY_ENTER {
                            if active_tab == 0 {
                                if entering_username {
                                    if !username.is_empty() {
                                        entering_username = false;
                                        warning = false;
                                    }
                                } else if !password.is_empty() {
                                    USER_NAME.set(username.clone()).ok();
                                    USER_PASSWORD.set(password.clone()).ok();
                                    let users: Vec<deemak::utils::auth::User> =
                                        deemak::utils::auth::load_users();
                                    let username = username.trim().to_string();
                                    let password = password.trim().to_string();
                                    if let Some(user) =
                                        users.iter().find(|u| u.username == username)
                                    {
                                        if verify_password(
                                            &password,
                                            &user.salt,
                                            &user.password_hash,
                                        ) {
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
                            } else {
                                if entering_reg_username {
                                    if !reg_username.is_empty() {
                                        entering_reg_username = false;
                                        reg_warning = false;
                                    }
                                } else if !reg_password.is_empty() {
                                    let username = reg_username.trim();
                                    let password = reg_password.trim();
                                    if users.iter().any(|u| u.username == username) {
                                        reg_warning = true;
                                        reg_warning_text = "Username already exists!".to_string();
                                    } else {
                                        match deemak::utils::auth::hash_password(password) {
                                            Ok((salt, hash)) => {
                                                users.push(deemak::utils::auth::User {
                                                    username: username.to_string(),
                                                    salt,
                                                    password_hash: hash,
                                                });
                                                deemak::utils::auth::save_users(&users);
                                                USER_NAME.set(username.to_string()).ok();
                                                USER_PASSWORD.set(password.to_string()).ok();
                                                return true;
                                            }
                                            Err(_) => {
                                                reg_warning = true;
                                                reg_warning_text =
                                                    "Failed to hash password!".to_string();
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        let mut d = rl.begin_drawing(thread);
        d.clear_background(Color::BLACK);
        let highlight_color = Color::GOLD;

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
            let tab_x = 200.0;
            let tab_y = top_y + 85.0;
            let tab_w = 160.0;
            let tab_h = 40.0;
            d.draw_rectangle(
                tab_x as i32,
                tab_y as i32,
                tab_w as i32,
                tab_h as i32,
                if active_tab == 0 {
                    highlight_color
                } else {
                    Color::alpha(&Color::GRAY, 0.3)
                },
            );
            d.draw_rectangle(
                (tab_x + tab_w) as i32,
                tab_y as i32,
                tab_w as i32,
                tab_h as i32,
                if active_tab == 1 {
                    highlight_color
                } else {
                    Color::alpha(&Color::GRAY, 0.3)
                },
            );
            d.draw_text_ex(
                &font_d,
                "Login",
                Vector2 {
                    x: tab_x + 40.0,
                    y: tab_y + 8.0,
                },
                24.0,
                1.0,
                if active_tab == 0 {
                    Color::BLACK
                } else {
                    Color::WHITE
                },
            );
            d.draw_text_ex(
                &font_d,
                "Register",
                Vector2 {
                    x: tab_x + tab_w + 40.0,
                    y: tab_y + 8.0,
                },
                24.0,
                1.0,
                if active_tab == 1 {
                    Color::BLACK
                } else {
                    Color::WHITE
                },
            );
            if active_tab == 0 {
                let base_x = 200.0;
                let user_y = top_y + 140.0;
                let pass_y = user_y + 100.0;
                let box_width = 320.0;
                let box_height = 40.0;
                if warning {
                    d.draw_text(
                        &warning_text,
                        base_x as i32,
                        (pass_y + 90.0) as i32,
                        20,
                        Color::RED,
                    );
                }
                d.draw_text(
                    "Username :",
                    base_x as i32,
                    (user_y + 5.0) as i32,
                    30,
                    Color::alpha(&Color::WHITE, 0.9),
                );
                d.draw_rectangle_lines(
                    base_x as i32,
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
                // Footer note
                let footer_note = "Welcome to Deemak by DBD! Use up/down keys to switch focus. Press Enter to continue. Not registered yet? Press Tab to switch from login to register.";
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
                        line = word.to_string();
                        y += font_size + 5.0;
                    } else {
                        line = trial;
                    }
                }
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
                let version = "Version 1.0";
                d.draw_text(
                    version,
                    10,
                    d.get_screen_height() - 30,
                    16,
                    Color::alpha(&Color::GRAY, 0.4),
                );
            } else {
                // Registration UI
                let base_x = 200.0;
                let user_y = top_y + 140.0;
                let pass_y = user_y + 100.0;
                let box_width = 320.0;
                let box_height = 40.0;
                if reg_warning {
                    d.draw_text(
                        &reg_warning_text,
                        base_x as i32,
                        (pass_y + 90.0) as i32,
                        20,
                        Color::RED,
                    );
                } else if !reg_warning_text.is_empty() {
                    d.draw_text(
                        &reg_warning_text,
                        base_x as i32,
                        (pass_y + 90.0) as i32,
                        20,
                        Color::GREEN,
                    );
                }
                d.draw_text(
                    "Username :",
                    base_x as i32,
                    (user_y + 5.0) as i32,
                    30,
                    Color::alpha(&Color::WHITE, 0.9),
                );
                d.draw_rectangle_lines(
                    base_x as i32,
                    (user_y + 40.0) as i32,
                    box_width as i32,
                    box_height as i32,
                    if entering_reg_username {
                        highlight_color
                    } else {
                        Color::GRAY
                    },
                );
                // Draw reg username
                let mut total_width = 0.0;
                let mut visible = String::new();
                for ch in reg_username.chars().rev() {
                    let s = CString::new(ch.to_string()).unwrap();
                    let w = unsafe { MeasureTextEx(font, s.as_ptr(), 30.0, 1.0).x };
                    if total_width + w + 10.0 > box_width {
                        break;
                    }
                    total_width += w;
                    visible.insert(0, ch);
                }
                let user_display = if entering_reg_username {
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
                    if !entering_reg_username {
                        highlight_color
                    } else {
                        Color::GRAY
                    },
                );
                // Draw masked reg password
                let masked = "*".repeat(reg_password.len());
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
                let pass_display = if !entering_reg_username {
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
                // Footer note
                let footer_note = "Welcome to Deemak by DBD! Use up/down keys to switch focus. Press Enter to submit. Already registered? Press Tab to switch from register to login.";
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
                        line = word.to_string();
                        y += font_size + 5.0;
                    } else {
                        line = trial;
                    }
                }
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
