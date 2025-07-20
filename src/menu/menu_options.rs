use raylib::prelude::*;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy)]
pub enum MenuOption {
    StartShell,
    About,
    Tutorial,
    Settings,
    Exit,
}

impl MenuOption {
    pub fn opts() -> &'static [Self] {
        &[
            Self::StartShell,
            Self::About,
            Self::Tutorial,
            Self::Settings,
            Self::Exit,
        ]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::StartShell => "Start Shell",
            Self::About => "About",
            Self::Tutorial => "Tutorial",
            Self::Settings => "Settings",
            Self::Exit => "Exit",
        }
    }
}

// Constants for layout and styling
struct MenuConfig {
    pub title_x: f32,
    pub menu_x: f32,
    pub menu_start_y: f32,
    pub menu_item_height: f32,
    pub cursor_x: f32,
    pub title_font_size: f32,
    pub menu_font_size: f32,
    pub cursor_font_size: i32,
    pub footer_font_size: i32,
    pub input_delay: Duration,
    pub animation_speed: f32,
    pub alpha_increment: f32,
}

impl Default for MenuConfig {
    fn default() -> Self {
        Self {
            title_x: 200.0,
            menu_x: 200.0,
            menu_start_y: 280.0,
            menu_item_height: 50.0,
            cursor_x: 175.0,
            title_font_size: 60.0,
            menu_font_size: 30.0,
            cursor_font_size: 30,
            footer_font_size: 16,
            input_delay: Duration::from_millis(150),
            animation_speed: 0.1,
            alpha_increment: 0.02,
        }
    }
}

// Animation state manager
struct MenuAnimation {
    pub alpha: f32,
    pub y_offset: f32,
    pub target_y: f32,
}

impl MenuAnimation {
    fn new(start_y: f32, target_y: f32) -> Self {
        Self {
            alpha: 0.0,
            y_offset: start_y,
            target_y,
        }
    }

    fn update(&mut self, config: &MenuConfig) {
        self.alpha = (self.alpha + config.alpha_increment).min(1.0);
        self.y_offset = self.y_offset + (self.target_y - self.y_offset) * config.animation_speed;
    }
}

// Input handler for menu navigation
struct MenuInput {
    pub selected: usize,
    pub last_change: Instant,
}

impl MenuInput {
    fn new() -> Self {
        Self {
            selected: 0,
            last_change: Instant::now(),
        }
    }

    fn handle_input(&mut self, rl: &mut RaylibHandle, config: &MenuConfig) -> Option<MenuOption> {
        if self.last_change.elapsed() > config.input_delay {
            if let Some(key) = rl.get_key_pressed() {
                match key {
                    KeyboardKey::KEY_UP => {
                        self.selected = (self.selected + MenuOption::opts().len() - 1)
                            % MenuOption::opts().len();
                        self.last_change = Instant::now();
                    }
                    KeyboardKey::KEY_DOWN => {
                        self.selected = (self.selected + 1) % MenuOption::opts().len();
                        self.last_change = Instant::now();
                    }
                    KeyboardKey::KEY_ENTER => {
                        return Some(MenuOption::opts()[self.selected]);
                    }
                    _ => {}
                }
            }
        }
        None
    }
}

pub fn show_menu(rl: &mut RaylibHandle, thread: &RaylibThread) -> Option<MenuOption> {
    let config = MenuConfig::default();
    let mut animation = MenuAnimation::new(120.0, 180.0);
    let mut input = MenuInput::new();
    let font = rl.get_font_default();

    while !rl.window_should_close() {
        // Handle input and check for menu selection
        if let Some(selected_option) = input.handle_input(rl, &config) {
            return Some(selected_option);
        }

        // Update animations
        animation.update(&config);

        // Draw everything
        let mut d = rl.begin_drawing(thread);
        d.clear_background(Color::BLACK);

        // Draw title
        d.draw_text_ex(
            &font,
            "DEEMAK SHELL",
            Vector2::new(config.title_x, animation.y_offset),
            config.title_font_size,
            2.0,
            Color::new(255, 255, 255, 255),
        );

        // Draw menu options
        for (i, option) in MenuOption::opts().iter().enumerate() {
            let color = if i == input.selected {
                Color::GOLD
            } else {
                Color::new(200, 200, 200, (animation.alpha * 200.0) as u8)
            };

            d.draw_text_ex(
                &font,
                option.as_str(),
                Vector2::new(
                    config.menu_x,
                    config.menu_start_y + (i as f32 * config.menu_item_height),
                ),
                config.menu_font_size,
                1.0,
                color,
            );
        }

        // Draw animated cursor
        d.draw_text(
            ">",
            config.cursor_x as i32,
            (config.menu_start_y + input.selected as f32 * config.menu_item_height) as i32,
            config.cursor_font_size,
            Color::new(
                255,
                255,
                255,
                ((animation.alpha * 0.5).sin().abs() * 255.0) as u8,
            ),
        );

        // Draw footer
        let initials = "IISc DataBased Club";
        let size = d.measure_text(initials, config.footer_font_size);
        d.draw_text(
            initials,
            d.get_screen_width() - size - 10,
            d.get_screen_height() - 30,
            config.footer_font_size,
            Color::alpha(&Color::GRAY, 0.4),
        );

        let version = "Version 1.0";
        d.draw_text(
            version,
            10,
            d.get_screen_height() - 30,
            config.footer_font_size,
            Color::alpha(&Color::GRAY, 0.4),
        );
    }

    None
}
