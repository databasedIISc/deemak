use crate::utils::config::{self, FONT_OPTIONS};
use raylib::prelude::*;
use std::time::{Duration, Instant};

/// A trait for a UI screen, defining a common interface for running and managing screens.
trait Screen {
    /// Runs the screen, handling its event loop, updates, and drawing.
    fn run(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread);
}

#[derive(Debug, Clone, Copy, PartialEq)]
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
    // Load all fonts to be displayed for the preview.
    let loaded_fonts: Vec<Font> = FONT_OPTIONS
        .iter()
        .map(|(_, path)| {
            rl.load_font_ex(thread, path, 30, None)
                .unwrap_or_else(|e| panic!("Failed to load font: {}. Error: {}", path, e))
        })
        .collect();

    let default_font = rl.get_font_default();
    let mut last_change = Instant::now();

    // The total number of selectable items is the number of fonts plus the "Back" button.
    let options_count = FONT_OPTIONS.len() + 1;
    // The current selection can be any font index, or the index for the "Back" button.
    let mut current_selection = *selected_font;

    // Store the currently configured font to display the '*' marker.
    let mut saved_font_index = config::load_config().font_index;

    // Load the custom font for the footnote.
    let custom_font = rl
        .load_font_ex(
            thread,
            "fontbook/fonts/ttf/JetBrainsMono-Medium.ttf",
            18,
            None,
        )
        .expect("Failed to load custom font");

    while !rl.window_should_close() {
        if last_change.elapsed() > Duration::from_millis(100) {
            if rl.is_key_pressed(KeyboardKey::KEY_UP) {
                if current_selection > 0 {
                    current_selection -= 1;
                } else {
                    current_selection = options_count - 1; // Wrap around
                }
                last_change = Instant::now();
            } else if rl.is_key_pressed(KeyboardKey::KEY_DOWN) {
                if current_selection < options_count - 1 {
                    current_selection += 1;
                } else {
                    current_selection = 0; // Wrap around
                }
                last_change = Instant::now();
            } else if rl.is_key_pressed(KeyboardKey::KEY_ENTER) {
                // If the selection index is within the font list, it's a font change.
                if current_selection < FONT_OPTIONS.len() {
                    *selected_font = current_selection;
                    let mut cfg = config::load_config();
                    cfg.font_index = *selected_font;
                    config::save_config(&cfg);
                    saved_font_index = *selected_font; // Update saved index for immediate feedback
                } else {
                    // Otherwise, it's the "Back" button, so we exit.
                    return;
                }
            }
        }

        let mut d = rl.begin_drawing(thread);
        d.clear_background(Color::BLACK);
        d.draw_text_ex(
            &default_font,
            "Select Font",
            Vector2::new(200.0, 100.0),
            40.0,
            2.0,
            Color::WHITE,
        );

        for (i, (name, _)) in FONT_OPTIONS.iter().enumerate() {
            let color = if i == current_selection {
                Color::GOLD
            } else {
                Color::GRAY
            };

            // Create the display text with the "(default)" label for the first option.
            let display_text = if i == 0 {
                format!("{} (default)", name)
            } else {
                name.to_string()
            };

            // Draw a marker for the currently saved font.
            if i == saved_font_index {
                d.draw_text_ex(
                    &default_font,
                    "*", // Using a checkmark symbol for clarity
                    Vector2::new(170.0, 180.0 + (i as f32 * 30.0)),
                    25.0,
                    1.0,
                    Color::GREEN,
                );
            }
            // Draw a cursor for the currently hovered selection.
            else if i == current_selection {
                d.draw_text_ex(
                    &default_font,
                    ">",
                    Vector2::new(170.0, 180.0 + (i as f32 * 30.0)),
                    25.0,
                    1.0,
                    Color::GOLD,
                );
            }

            // Use the specific font for this option to draw its name.
            let option_font = &loaded_fonts[i];
            d.draw_text_ex(
                option_font,
                &display_text,
                Vector2::new(200.0, 180.0 + (i as f32 * 30.0)),
                25.0,
                1.0,
                color,
            );
        }

        // Determine if the "Back" option is currently selected.
        let is_back_selected = current_selection == FONT_OPTIONS.len();
        let back_color = if is_back_selected {
            Color::GOLD
        } else {
            Color::GRAY
        };
        d.draw_text_ex(
            &default_font,
            "Back",
            Vector2::new(200.0, 180.0 + (FONT_OPTIONS.len() as f32 * 30.0)),
            25.0,
            1.0,
            back_color,
        );

        // Draw the cursor for the "Back" option when it's selected.
        if is_back_selected {
            let cursor_y = 180.0 + (FONT_OPTIONS.len() as f32 * 30.0);
            d.draw_text_ex(
                &default_font,
                ">",
                Vector2::new(170.0, cursor_y),
                25.0,
                1.0,
                Color::GOLD,
            );
        }

        // Draw the footnote explaining the '*' symbol.
        let footnote = "* Represents currently selected font";
        let footnote_width = d.measure_text(footnote, 18);
        let footnote_pos = Vector2::new(
            (d.get_screen_width() as f32 - footnote_width as f32) / 2.0,
            d.get_screen_height() as f32 - 50.0,
        );
        d.draw_text_ex(&custom_font, footnote, footnote_pos, 18.0, 1.0, Color::GRAY);
    }
    // Fonts are automatically unloaded when `loaded_fonts` and `custom_font` go out of scope.
}

/// A screen to display the application's keybindings.
struct KeybindingsScreen {
    font: Font,
    keybindings: Vec<(String, String)>,
    last_change: Instant,
    alpha: f32,
    y_offset: f32,
    target_y: f32,
}

impl KeybindingsScreen {
    /// Creates a new `KeybindingsScreen`.
    fn new(rl: &mut RaylibHandle, thread: &RaylibThread) -> Self {
        let font = rl
            .load_font_ex(
                thread,
                "fontbook/fonts/ttf/JetBrainsMono-Medium.ttf",
                20,
                None,
            )
            .expect("Failed to load keybindings font");

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
        d.draw_text_ex(
            &self.font,
            "Keybinding",
            Vector2::new(100.0, header_y),
            20.0,
            1.0,
            Color::LIGHTGRAY,
        );
        d.draw_text_ex(
            &self.font,
            "Function",
            Vector2::new(400.0, header_y),
            20.0,
            1.0,
            Color::LIGHTGRAY,
        );

        // Draw separator line
        let separator_y = header_y + 25.0;
        d.draw_line(
            100,
            separator_y as i32,
            650,
            separator_y as i32,
            Color::GRAY,
        );

        // Draw keybindings with wrapping for descriptions
        let mut y = separator_y + 20.0;
        let max_desc_width = d.get_screen_width() as f32 - 400.0 - 20.0; // 20px padding

        for (key, desc) in &self.keybindings {
            d.draw_text_ex(
                &self.font,
                key,
                Vector2::new(100.0, y),
                20.0,
                1.0,
                Color::WHITE,
            );

            let wrapped_lines = wrap_text(&self.font, desc, 20.0, 1.0, max_desc_width);
            let mut line_y = y;
            for line in &wrapped_lines {
                d.draw_text_ex(
                    &self.font,
                    line,
                    Vector2::new(400.0, line_y),
                    20.0,
                    1.0,
                    Color::WHITE,
                );
                line_y += 25.0; // Line height
            }

            y += (wrapped_lines.len() as f32 * 25.0).max(30.0); // Move to the next entry
        }

        // Draw "Press Enter to go back" message
        let back_msg = "Press Enter to go back";
        let back_msg_width = d.measure_text(back_msg, 20);
        d.draw_text_ex(
            &font_heading,
            back_msg,
            Vector2::new(
                (d.get_screen_width() as f32 - back_msg_width as f32) / 2.0,
                d.get_screen_height() as f32 - 50.0,
            ),
            20.0,
            1.0,
            Color::GRAY,
        );
    }
}

impl Drop for KeybindingsScreen {
    fn drop(&mut self) {
        // Unload font to prevent memory leaks.
        // This is handled automatically by raylib-rs wrapper's Drop implementation.
    }
}

/// Wraps text to fit within a maximum width.
fn wrap_text(font: &Font, text: &str, font_size: f32, spacing: f32, max_width: f32) -> Vec<String> {
    let mut lines = Vec::new();
    if text.is_empty() {
        return lines;
    }

    let mut current_line = String::new();
    let words: Vec<&str> = text.split_whitespace().collect();

    for word in words {
        let test_line = if current_line.is_empty() {
            word.to_string()
        } else {
            format!("{} {}", current_line, word)
        };

        let text_size = font.measure_text(&test_line, font_size, spacing);

        if text_size.x > max_width && !current_line.is_empty() {
            lines.push(current_line);
            current_line = word.to_string();
        } else {
            current_line = test_line;
        }
    }
    lines.push(current_line);

    lines
}

impl Screen for KeybindingsScreen {
    fn run(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread) {
        while !rl.window_should_close() {
            if self.handle_input(rl) {
                break;
            }
            self.update();

            let mut d = rl.begin_drawing(thread);
            self.draw(&mut d);
        }
    }
}

pub fn show_keybindings(rl: &mut RaylibHandle, thread: &RaylibThread) {
    let mut screen = KeybindingsScreen::new(rl, thread);
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
