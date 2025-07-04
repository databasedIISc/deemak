use raylib::prelude::*;
use std::time::{Duration, Instant};

const MENU_OPTIONS: [&str; 4] = ["Start Shell", "About", "Settings", "Exit"];

pub fn show_menu(rl: &mut RaylibHandle, thread: &RaylibThread) -> Option<usize> {
    let mut selected: usize = 0;
    let mut last_change = Instant::now();
    let mut alpha = 0.0f32;
    let show_menu = true;

    // Animation variables
    let mut y_offset = 120.0;
    let target_y = 180.0;

    // Load font or fallback to default
    let font = rl.get_font_default();

    while !rl.window_should_close() && show_menu {
        // Handle input with wrapping around
        if last_change.elapsed() > Duration::from_millis(150) {
            if rl.is_key_pressed(KeyboardKey::KEY_UP) {
                selected = (selected + MENU_OPTIONS.len() - 1) % MENU_OPTIONS.len();
                last_change = Instant::now();
            } else if rl.is_key_pressed(KeyboardKey::KEY_DOWN) {
                selected = (selected + 1) % MENU_OPTIONS.len();
                last_change = Instant::now();
            } else if rl.is_key_pressed(KeyboardKey::KEY_ENTER) {
                return Some(selected);
            }
        }

        // Update animations
        alpha = (alpha + 0.02).min(1.0);
        y_offset = y_offset + (target_y - y_offset) * 0.1; // Manual lerp implementation

        // Draw
        let mut d = rl.begin_drawing(thread);
        d.clear_background(Color::BLACK);

        // Animated title
        d.draw_text_ex(
            &font,
            "DEEMAK SHELL",
            Vector2::new(200.0, y_offset),
            60.0,   
            2.0,
            Color::new(255, 255, 255, 255.0 as u8),
        );

        // Menu options
        for (i, option) in MENU_OPTIONS.iter().enumerate() {
            let color = if i == selected {
                Color::GOLD
            } else {
                Color::new(200, 200, 200, (alpha * 200.0) as u8)
            };

            d.draw_text_ex(
                &font,
                option,
                Vector2::new(200.0, 280.0 + (i as f32 * 50.0)),
                30.0,
                1.0,
                color,
            );
        }

        // Animated cursor
        d.draw_text(
            ">",
            175,
            (280 + selected as i32 * 50) as i32, // Convert usize to i32
            30,
            Color::new(255, 255, 255, ((alpha * 0.5).sin().abs() * 255.0) as u8),
        );

        // Footer text
        let initials  = "IISc DataBased Club";
        let size = d.measure_text(initials, 16);
        d.draw_text(
            initials,
            d.get_screen_width() - size - 10,
            d.get_screen_height() - 30,
            16,
            Color::fade(&Color::GRAY, 0.4),
        );

        let version = "Version 1.0";
        let version_size = d.measure_text(version, 16);
        d.draw_text(
            version,
            10, // 10 pixels from the left edge
            d.get_screen_height() - 30, // 30 pixels from the bottom
            16,
            Color::fade(&Color::GRAY, 0.4),
        );
    }

    

    None
}

