use raylib::prelude::*;
use std::time::{Duration, Instant};

const SETTINGS_OPTIONS: [&str; 1] = ["Back"];

pub fn show_settings(rl: &mut RaylibHandle, thread: &RaylibThread) {
    let mut selected: usize = 0;
    let mut last_change = Instant::now();
    let mut alpha = 0.0f32;
    let mut y_offset = 300.0;
    let target_y = 200.0;

    let font = rl.get_font_default();

    while !rl.window_should_close() {
        // Handle input
        if last_change.elapsed() > Duration::from_millis(150) {
            if let Some(key) = rl.get_key_pressed() {
                if key == KeyboardKey::KEY_UP {
                    selected = selected.saturating_sub(1);
                    last_change = Instant::now();
                } else if key == KeyboardKey::KEY_DOWN {
                    selected = (selected + 1).min(SETTINGS_OPTIONS.len() - 1);
                    last_change = Instant::now();
                } else if key == KeyboardKey::KEY_ENTER && selected == 0 {
                    return; // Exit settings
                } else {
                    continue; // Ignore other keys
                }
            }
        }

        // Animate
        alpha = (alpha + 0.02).min(1.0);
        y_offset += (target_y - y_offset) * 0.1;

        // Draw
        let mut d = rl.begin_drawing(thread);
        d.clear_background(Color::BLACK);

        // Animated title
        d.draw_text_ex(
            &font,
            "Settings",
            Vector2::new(200.0, y_offset),
            50.0,
            2.0,
            Color::new(255, 255, 255, (alpha * 255.0) as u8),
        );

        // Draw settings options
        for (i, option) in SETTINGS_OPTIONS.iter().enumerate() {
            let color = if i == selected {
                Color::GOLD
            } else {
                Color::new(200, 200, 200, (alpha * 200.0) as u8)
            };

            d.draw_text_ex(
                &font,
                option,
                Vector2::new(200.0, 300.0 + (i as f32 * 50.0)),
                30.0,
                1.0,
                color,
            );
        }

        // Cursor
        d.draw_text(
            ">",
            175,
            300 + selected as i32 * 50,
            30,
            Color::new(255, 255, 255, ((alpha * 0.5).sin().abs() * 255.0) as u8),
        );
    }
}
