use crate::utils::config::{self, FONT_OPTIONS};
use raylib::prelude::*;
use std::ffi::{c_char, CString};
use std::os::raw::c_int;
use std::time::{Duration, Instant};

/// A trait for a UI screen, defining a common interface for running and managing screens.
trait Screen {
    /// Runs the screen, handling its event loop, updates, and drawing.
    fn run(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread);
}

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
    
    // Store the current font selection at the start
    let current_font_index = config::load_config().font_index;
    
    // Load the custom font for the footnote (same as keybindings)
    let custom_font = unsafe {
        let path = CString::new("fontbook/fonts/ttf/JetBrainsMono-Medium.ttf").unwrap();
        raylib::ffi::LoadFontEx(
            path.as_ptr() as *const c_char,
            600 as c_int,
            std::ptr::null_mut::<c_int>(),
            0,
        )
    };

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
            
            // Create the display text with default label if it's the first option
            let display_text = if i == 0 {
                format!("{} (default)", name)
            } else {
                name.to_string()
            };
            
            // Draw tick mark for currently selected font
            if i == current_font_index {
                d.draw_text_ex(
                    &font,
                    "*",
                    Vector2::new(170.0, 180.0 + (i as f32 * 30.0)),
                    30.0,
                    1.0,
                    Color::GREEN,
                );
            }
            // Draw cursor '>' only if this item is selected AND it's not the currently active font
            else if i == *selected_font && !is_back_selected {
                d.draw_text_ex(
                    &font,
                    ">",
                    Vector2::new(170.0, 180.0 + (i as f32 * 30.0)),
                    30.0,
                    1.0,
                    Color::GOLD,
                );
            }
            
            d.draw_text_ex(
                &font,
                &display_text,
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

        // Draw cursor for "Back" option only when it's selected
        if is_back_selected {
            let cursor_y = 180 + FONT_OPTIONS.len() as i32 * 30;
            d.draw_text_ex(
                &font,
                ">",
                Vector2::new(170.0, cursor_y as f32),
                30.0,
                1.0,
                Color::GOLD,
            );
        }

        // Draw footnote explaining the '*' symbol (same format as keybindings footer)
        let footnote = "* represents currently selected font";
        let footnote_content = CString::new(footnote).unwrap();
        let footnote_width =
            unsafe { raylib::ffi::MeasureTextEx(custom_font, footnote_content.as_ptr(), 18.0, 1.0).x };
        let footnote_pos = raylib::ffi::Vector2 {
            x: (d.get_screen_width() as f32 - footnote_width) / 2.0,
            y: d.get_screen_height() as f32 - 50.0,
        };
        unsafe {
            raylib::ffi::DrawTextEx(
                custom_font,
                footnote_content.as_ptr(),
                footnote_pos,
                18.0,
                1.0,
                Color::GRAY.into(),
            );
        }
    }
}

/// A screen to display the application's keybindings.
struct KeybindingsScreen {
    font: raylib::ffi::Font,
    keybindings: Vec<(String, String)>,
    last_change: Instant,
    alpha: f32,
    y_offset: f32,
    target_y: f32,
}

impl KeybindingsScreen {
    /// Creates a new `KeybindingsScreen`.
    fn new() -> Self {
        let font = unsafe {
            let path = CString::new("fontbook/fonts/ttf/JetBrainsMono-Medium.ttf").unwrap();
            raylib::ffi::LoadFontEx(
                path.as_ptr() as *const c_char,
                600 as c_int,
                std::ptr::null_mut::<c_int>(),
                0,
            )
        };
        let keybindings = [
            ("Keyboard characters", "Keyboard chars"),
            ("Ctrl+Shift+C ", "Copy (Linux/MacOS)"),
            ("Ctrl+Shift+V ", "Paste (Linux/MacOS)"),
            ("Ctrl+K", "Clear Line"),
            ("Ctrl+C", "Next prompt"),
            ("Tab", "File completion only till Current Working Directory"),
            ("Arrow keys", "Navigate through history"),
        ]
        .iter()
        .map(|(key, desc)| (key.to_string(), desc.to_string()))
        .collect();

        Self {
            font,
            keybindings,
            last_change: Instant::now(),
            alpha: 0.0,
            y_offset: 200.0,
            target_y: 120.0,
        }
    }

    /// Handles user input for the keybindings screen. Returns true if the screen should be closed.
    fn handle_input(&mut self, rl: &mut RaylibHandle) -> bool {
        if self.last_change.elapsed() > Duration::from_millis(150)
            && rl.is_key_pressed(KeyboardKey::KEY_ENTER)
        {
            return true; // Exit screen
        }
        false
    }

    /// Updates the screen's animation state.
    fn update(&mut self) {
        self.alpha = (self.alpha + 0.02).min(1.0);
        self.y_offset += (self.target_y - self.y_offset) * 0.1;
    }

    /// Draws the keybindings screen.
    fn draw(&self, d: &mut RaylibDrawHandle) {
        let font_heading = d.get_font_default();
        d.clear_background(Color::BLACK);

        // Draw heading with animation
        d.draw_text_ex(
            &font_heading,
            "KEYBINDINGS",
            Vector2::new(200.0, self.y_offset),
            50.0,
            2.0,
            Color::new(255, 255, 255, (self.alpha * 255.0) as u8),
        );

        // Draw column headers
        let header_y = 180.0;
        let key_header = CString::new("Keybinding").unwrap();
        let desc_header = CString::new("Function").unwrap();
        
        unsafe {
            raylib::ffi::DrawTextEx(
                self.font,
                key_header.as_ptr(),
                raylib::ffi::Vector2 { x: 100.0, y: header_y },
                20.0,
                1.0,
                Color::GOLD.into(),
            );
            raylib::ffi::DrawTextEx(
                self.font,
                desc_header.as_ptr(),
                raylib::ffi::Vector2 { x: 320.0, y: header_y },
                20.0,
                1.0,
                Color::GOLD.into(),
            );
        }

        // Draw separator line
        let separator_y = header_y + 25.0;
        let separator = CString::new("-------------------------------------------------------").unwrap();
        unsafe {
            raylib::ffi::DrawTextEx(
                self.font,
                separator.as_ptr(),
                raylib::ffi::Vector2 { x: 100.0, y: separator_y },
                16.0,
                1.0,
                Color::GRAY.into(),
            );
        }

        // Draw keybindings list with proper formatting and wrapping
        let mut y_pos = separator_y + 35.0;
        let line_height = 25.0;
        let key_column_width = 200.0;
        let desc_column_width = d.get_screen_width() as f32 - 350.0; // Leave margin for wrapping
        
        for (key, description) in &self.keybindings {
            // Draw the keybinding (left column)
            let key_content = CString::new(key.as_str()).unwrap();
            unsafe {
                raylib::ffi::DrawTextEx(
                    self.font,
                    key_content.as_ptr(),
                    raylib::ffi::Vector2 { x: 100.0, y: y_pos },
                    18.0,
                    1.0,
                    Color::WHITE.into(),
                );
            }

            // Handle text wrapping for description (right column)
            let words: Vec<&str> = description.split_whitespace().collect();
            let mut current_line = String::new();
            let mut desc_y = y_pos;
            
            for word in words {
                let test_line = if current_line.is_empty() {
                    word.to_string()
                } else {
                    format!("{} {}", current_line, word)
                };
                
                let test_content = CString::new(test_line.as_str()).unwrap();
                let text_width = unsafe {
                    raylib::ffi::MeasureTextEx(self.font, test_content.as_ptr(), 18.0, 1.0).x
                };
                
                if text_width > desc_column_width && !current_line.is_empty() {
                    // Draw current line and start new one
                    let line_content = CString::new(current_line.as_str()).unwrap();
                    unsafe {
                        raylib::ffi::DrawTextEx(
                            self.font,
                            line_content.as_ptr(),
                            raylib::ffi::Vector2 { x: 320.0, y: desc_y },
                            18.0,
                            1.0,
                            Color::LIGHTGRAY.into(),
                        );
                    }
                    desc_y += line_height;
                    current_line = word.to_string();
                } else {
                    current_line = test_line;
                }
            }
            
            // Draw the last line
            if !current_line.is_empty() {
                let line_content = CString::new(current_line.as_str()).unwrap();
                unsafe {
                    raylib::ffi::DrawTextEx(
                        self.font,
                        line_content.as_ptr(),
                        raylib::ffi::Vector2 { x: 320.0, y: desc_y },
                        18.0,
                        1.0,
                        Color::LIGHTGRAY.into(),
                    );
                }
            }
            
            // Move to next keybinding (ensure proper spacing)
            y_pos = desc_y + line_height + 5.0;
        }

        // Draw footer
        let footer = "Press ENTER to go back";
        let footer_content = CString::new(footer).unwrap();
        let footer_width =
            unsafe { raylib::ffi::MeasureTextEx(self.font, footer_content.as_ptr(), 20.0, 1.0).x };
        let footer_pos = raylib::ffi::Vector2 {
            x: (d.get_screen_width() as f32 - footer_width) / 2.0,
            y: d.get_screen_height() as f32 - 50.0,
        };
        unsafe {
            raylib::ffi::DrawTextEx(
                self.font,
                footer_content.as_ptr(),
                footer_pos,
                20.0,
                1.0,
                Color::GRAY.into(),
            );
        }
    }
}

impl Screen for KeybindingsScreen {
    /// Runs the main loop for the keybindings screen.
    fn run(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread) {
        while !rl.window_should_close() {
            if self.handle_input(rl) {
                return;
            }
            self.update();

            let mut d = rl.begin_drawing(thread);
            self.draw(&mut d);
        }
    }
}

pub fn show_keybindings(rl: &mut RaylibHandle, thread: &RaylibThread) {
    let mut screen = KeybindingsScreen::new();
    screen.run(rl, thread);
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
