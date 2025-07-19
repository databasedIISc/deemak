use crate::utils::config;
use raylib::prelude::*;
use std::ffi::c_char;
use std::os::raw::c_int;
use std::time::{Duration, Instant};

const FONT_OPTIONS: [(&str, &str); 11] = [
    ("Hack Nerd", "fontbook/fonts/ttf/HackNerdFont-Regular.ttf"),
    (
        "Hack Nerd Mono",
        "fontbook/fonts/ttf/HackNerdFontMono-Regular.ttf",
    ),
    (
        "Hack Nerd Propo",
        "fontbook/fonts/ttf/HackNerdFontPropo-Regular.ttf",
    ),
    (
        "JetBrains Mono Medium",
        "fontbook/fonts/ttf/JetBrainsMono-Medium.ttf",
    ),
    (
        "JetBrains Mono Regular",
        "fontbook/fonts/ttf/JetBrainsMono-Regular.ttf",
    ),
    (
        "JetBrains Mono NL Light",
        "fontbook/fonts/ttf/JetBrainsMonoNL-Light.ttf",
    ),
    (
        "JetBrains Mono NL Medium",
        "fontbook/fonts/ttf/JetBrainsMonoNL-Medium.ttf",
    ),
    (
        "JetBrains Mono NL Regular",
        "fontbook/fonts/ttf/JetBrainsMonoNL-Regular.ttf",
    ),
    (
        "JetBrains Mono NL Thin",
        "fontbook/fonts/ttf/JetBrainsMonoNL-Thin.ttf",
    ),
    (
        "JetBrains Mono NL Thin Italic",
        "fontbook/fonts/ttf/JetBrainsMonoNL-ThinItalic.ttf",
    ),
    (
        "Meslo LGS NF Regular",
        "fontbook/fonts/ttf/MesloLGS NF Regular.ttf",
    ),
];

#[derive(Debug, Clone, Copy)]
pub enum SettingsOption {
    Font,
    Keybindings,
    Back,
}

impl SettingsOption {
    pub fn opts() -> &'static [Self] {
        &[Self::Font, Self::Keybindings, Self::Back]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Font => "Font",
            Self::Keybindings => "Keybindings",
            Self::Back => "Back",
        }
    }
}

pub fn show_font_selection(
    rl: &mut RaylibHandle,
    thread: &RaylibThread,
    selected_font: &mut usize,
) {
    let font = rl.get_font_default();
    let mut last_change = Instant::now();
    let mut is_back_selected = false;

    while !rl.window_should_close() {
        if last_change.elapsed() > Duration::from_millis(150) {
            if rl.is_key_pressed(KeyboardKey::KEY_UP) {
                if is_back_selected {
                    is_back_selected = false;
                } else {
                    *selected_font = selected_font.saturating_sub(1);
                }
                last_change = Instant::now();
            } else if rl.is_key_pressed(KeyboardKey::KEY_DOWN) {
                if *selected_font == FONT_OPTIONS.len() - 1 && !is_back_selected {
                    is_back_selected = true;
                } else if !is_back_selected {
                    *selected_font = (*selected_font + 1).min(FONT_OPTIONS.len() - 1);
                }
                last_change = Instant::now();
            } else if rl.is_key_pressed(KeyboardKey::KEY_ENTER) {
                if is_back_selected {
                    return;
                } else {
                    let mut cfg = config::load_config();
                    cfg.font_index = *selected_font;
                    config::save_config(&cfg);
                    return;
                }
            }
        }

        let mut d = rl.begin_drawing(thread);
        d.clear_background(Color::BLACK);
        d.draw_text_ex(
            &font,
            "Select Font",
            Vector2::new(200.0, 100.0),
            40.0,
            2.0,
            Color::WHITE,
        );

        for (i, (name, _)) in FONT_OPTIONS.iter().enumerate() {
            let color = if i == *selected_font && !is_back_selected {
                Color::GOLD
            } else {
                Color::GRAY
            };
            d.draw_text_ex(
                &font,
                name,
                Vector2::new(200.0, 180.0 + (i as f32 * 30.0)),
                30.0,
                1.0,
                color,
            );
        }

        let back_color = if is_back_selected {
            Color::GOLD
        } else {
            Color::GRAY
        };
        d.draw_text_ex(
            &font,
            "Back",
            Vector2::new(200.0, 180.0 + (FONT_OPTIONS.len() as f32 * 30.0)),
            30.0,
            1.0,
            back_color,
        );

        let cursor_y = if is_back_selected {
            180 + FONT_OPTIONS.len() as i32 * 30
        } else {
            180 + *selected_font as i32 * 30
        };
        d.draw_text(">", 180, cursor_y, 30, Color::GOLD);
    }
}

pub fn show_keybindings(rl: &mut RaylibHandle, thread: &RaylibThread) {
    let font = unsafe {
        let path = std::ffi::CString::new("fontbook/fonts/ttf/JetBrainsMono-Medium.ttf").unwrap();
        raylib::ffi::LoadFontEx(
            path.as_ptr() as *const c_char,
            600.0 as c_int,
            std::ptr::null_mut::<c_int>(),
            0,
        )
    };

    let keybindings = [
        "Keyboard characters - Keyboard chars",
        "Ctrl+Shift+C (Linux/MacOS) - Copy",
        "Ctrl+Shift+V (Linux/MacOS) - Paste",
        "Ctrl+K - Clear Line",
        "Ctrl+C - Next prompt",
        "TAB - File completion till Current Working Directory",
    ];

    let mut last_change = Instant::now();
    let mut alpha = 0.0f32;
    let mut y_offset = 300.0;
    let target_y = 200.0;

    while !rl.window_should_close() {
        if last_change.elapsed() > Duration::from_millis(150) {
            if rl.is_key_pressed(KeyboardKey::KEY_ENTER) {
                return;
            }
        }

        alpha = (alpha + 0.02).min(1.0);
        y_offset += (target_y - y_offset) * 0.1;

        {
            let font_heading = rl.get_font_default();
            let mut d = rl.begin_drawing(thread);
            d.clear_background(Color::BLACK);
            // Draw heading
            d.draw_text_ex(
                &font_heading,
                "KEYBINDINGS",
                Vector2::new(200.0, 200.0),
                45.0,
                2.0,
                Color::new(255, 255, 255, (alpha * 255.0) as u8),
            );

            // Draw keybindings
            let mut y_offset = 0.0;
            let line_height = 30.0;
            for binding in keybindings.iter() {
                unsafe {
                    let binding_content = std::ffi::CString::new(*binding).unwrap();
                    let binding_pos = raylib::ffi::Vector2 {
                        x: 100.0,
                        y: 250.0 + y_offset,
                    };
                    raylib::ffi::DrawTextEx(
                        font,
                        binding_content.as_ptr() as *const c_char,
                        binding_pos,
                        24.0,
                        1.0,
                        raylib::ffi::ColorFromHSV(0.0, 0.0, 1.0),
                    );
                }
                y_offset += line_height;
            }

            // Draw footer
            unsafe {
                let footer = "Press ENTER to go back";
                let footer_content = std::ffi::CString::new(footer).unwrap();
                let footer_width =
                    raylib::ffi::MeasureTextEx(font, footer_content.as_ptr(), 20.0, 1.0).x;
                let footer_pos = raylib::ffi::Vector2 {
                    x: (raylib::ffi::GetScreenWidth() as f32 - footer_width) / 2.0,
                    y: raylib::ffi::GetScreenHeight() as f32 - 50.0,
                };
                raylib::ffi::DrawTextEx(
                    font,
                    footer_content.as_ptr() as *const c_char,
                    footer_pos,
                    20.0,
                    1.0,
                    raylib::ffi::ColorFromHSV(0.0, 0.0, 0.51),
                );
            }
        }
    }
}

pub fn show_settings(rl: &mut RaylibHandle, thread: &RaylibThread) {
    let mut selected: usize = 0;
    let mut last_change = Instant::now();
    let mut alpha = 0.0f32;
    let mut y_offset = 300.0;
    let target_y = 200.0;
    let font = rl.get_font_default();
    let mut selected_font: usize = config::load_config().font_index;

    while !rl.window_should_close() {
        if last_change.elapsed() > Duration::from_millis(150) {
            if rl.is_key_pressed(KeyboardKey::KEY_UP) {
                selected = selected.saturating_sub(1);
                last_change = Instant::now();
            } else if rl.is_key_pressed(KeyboardKey::KEY_DOWN) {
                selected = (selected + 1).min(SettingsOption::opts().len() - 1);
                last_change = Instant::now();
            } else if rl.is_key_pressed(KeyboardKey::KEY_ENTER) {
                match SettingsOption::opts()[selected] {
                    SettingsOption::Font => show_font_selection(rl, thread, &mut selected_font),
                    SettingsOption::Keybindings => show_keybindings(rl, thread),
                    SettingsOption::Back => return,
                }
                last_change = Instant::now();
            }
        }

        alpha = (alpha + 0.02).min(1.0);
        y_offset += (target_y - y_offset) * 0.1;

        let mut d = rl.begin_drawing(thread);
        d.clear_background(Color::BLACK);

        d.draw_text_ex(
            &font,
            "Settings",
            Vector2::new(200.0, y_offset),
            50.0,
            2.0,
            Color::new(255, 255, 255, (alpha * 255.0) as u8),
        );

        for (i, option) in SettingsOption::opts().iter().enumerate() {
            let color = if i == selected {
                Color::GOLD
            } else {
                Color::new(200, 200, 200, (alpha * 200.0) as u8)
            };

            d.draw_text_ex(
                &font,
                option.as_str(),
                Vector2::new(200.0, 300.0 + (i as f32 * 50.0)),
                30.0,
                1.0,
                color,
            );
        }

        d.draw_text(
            ">",
            175,
            300 + selected as i32 * 50,
            30,
            Color::new(255, 255, 255, ((alpha * 0.5).sin().abs() * 255.0) as u8),
        );
    }
}
