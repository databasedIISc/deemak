use crate::keys::key_to_char;
use crate::utils::globals::{UserInfo, set_user_info};
use raylib::ffi::{DrawTextEx, LoadFontEx, MeasureTextEx, SetExitKey, Vector2};
use raylib::prelude::*;
use std::ffi::CString;
use std::os::raw::{c_char, c_int};
use std::time::{Duration, Instant};

// Login/Register UI configuration
struct LoginConfig {
    pub top_y: f32,
    pub base_x: f32,
    pub tab_spacing: f32,
    pub field_spacing: f32,
    pub box_width: f32,
    pub box_height: f32,
    pub tab_width: f32,
    pub tab_height: f32,
    pub divider_margin: f32,
    pub footer_margin: f32,
}

impl Default for LoginConfig {
    fn default() -> Self {
        Self {
            top_y: 100.0,
            base_x: 200.0,
            tab_spacing: 125.0,
            field_spacing: 180.0,
            box_width: 320.0,
            box_height: 40.0,
            tab_width: 160.0,
            tab_height: 40.0,
            divider_margin: 8.0,
            footer_margin: 50.0,
        }
    }
}

// Animation state manager
struct LoginAnimation {
    pub y_offset: f32,
    pub target_y: f32,
    pub alpha: f32,
    pub stream_index: usize,
    pub displayed_text: String,
    pub last_stream_time: Instant,
    pub animation_done: bool,
    pub show_input: bool,
    pub pause_start: Option<Instant>,
}

impl LoginAnimation {
    fn new(config: &LoginConfig) -> Self {
        Self {
            y_offset: config.top_y + 120.0,
            target_y: config.top_y + 20.0,
            alpha: 0.0,
            stream_index: 0,
            displayed_text: String::new(),
            last_stream_time: Instant::now(),
            animation_done: false,
            show_input: false,
            pause_start: None,
        }
    }

    fn update(&mut self) {
        let full_text = "WELCOME TO";
        let stream_delay = Duration::from_millis(80);

        if self.stream_index >= full_text.len() {
            self.alpha = (self.alpha + 0.02).min(1.0);
            self.y_offset += (self.target_y - self.y_offset) * 0.1;
            let _y: f32 = self.y_offset - self.target_y;
            if _y.abs() < 1.0 && !self.animation_done {
                self.animation_done = true;
                self.pause_start = Some(Instant::now());
            }
        }

        if self.animation_done && !self.show_input {
            if let Some(start) = self.pause_start {
                if start.elapsed() >= Duration::from_secs(0) {
                    self.show_input = true;
                }
            }
        }

        if self.stream_index < full_text.len() && self.last_stream_time.elapsed() >= stream_delay {
            self.stream_index += 1;
            self.displayed_text = full_text[..self.stream_index].to_string();
            self.last_stream_time = Instant::now();
        }
    }
}

// Tab management
#[derive(Debug, Clone, Copy, PartialEq)]
enum TabType {
    Login = 0,
    Register = 1,
}

struct TabManager {
    pub active_tab: TabType,
    pub login_fields: FieldPair,
    pub register_fields: FieldPair,
}

impl TabManager {
    fn new(users_empty: bool) -> Self {
        Self {
            active_tab: if users_empty {
                TabType::Register
            } else {
                TabType::Login
            },
            login_fields: FieldPair::new(),
            register_fields: FieldPair::new(),
        }
    }

    fn switch_tab(&mut self, new_tab: TabType) {
        if self.active_tab != new_tab {
            self.active_tab = new_tab;
            self.clear_all_fields();
            match new_tab {
                TabType::Login => {
                    self.login_fields.username.entering = true;
                    self.login_fields.password.entering = false;
                }
                TabType::Register => {
                    self.register_fields.username.entering = true;
                    self.register_fields.password.entering = false;
                }
            }
        }
    }

    fn clear_all_fields(&mut self) {
        self.login_fields.clear();
        self.register_fields.clear();
    }

    fn get_current_fields(&mut self) -> &mut FieldPair {
        match self.active_tab {
            TabType::Login => &mut self.login_fields,
            TabType::Register => &mut self.register_fields,
        }
    }

    fn get_footer_text(&self) -> &'static str {
        match self.active_tab {
            TabType::Login => LOGIN_FOOTER,
            TabType::Register => REGISTER_FOOTER,
        }
    }
}

// Field pair management
struct FieldPair {
    pub username: InputField,
    pub password: InputField,
}

impl FieldPair {
    fn new() -> Self {
        Self {
            username: InputField::new("Username :", true),
            password: InputField::new("Password :", false),
        }
    }

    fn clear(&mut self) {
        self.username.value.clear();
        self.password.value.clear();
        self.username.warning = false;
        self.password.warning = false;
        self.username.warning_text.clear();
        self.password.warning_text.clear();
    }

    fn toggle_focus(&mut self) {
        let user_entering = self.username.entering;
        self.username.entering = !user_entering;
        self.password.entering = user_entering;
    }

    fn has_warning(&self) -> bool {
        self.username.warning || self.password.warning
    }

    fn get_warning_text(&self) -> &str {
        if self.username.warning {
            &self.username.warning_text
        } else {
            &self.password.warning_text
        }
    }

    fn draw_fields(
        &self,
        d: &mut RaylibDrawHandle,
        font: raylib::ffi::Font,
        config: &LoginConfig,
        highlight_color: Color,
    ) {
        let user_y = config.top_y + config.field_spacing;
        let pass_y = user_y + 100.0;

        self.username.draw(
            d,
            font,
            config.base_x,
            user_y,
            config.box_width,
            config.box_height,
            highlight_color,
            false,
        );
        self.password.draw(
            d,
            font,
            config.base_x,
            pass_y,
            config.box_width,
            config.box_height,
            highlight_color,
            true,
        );

        if self.has_warning() {
            d.draw_text(
                self.get_warning_text(),
                config.base_x as i32,
                (pass_y + 90.0) as i32,
                20,
                Color::RED,
            );
        }
    }
}

// Mouse input handler
struct MouseHandler;

impl MouseHandler {
    fn handle_clicks(rl: &mut RaylibHandle, tab_manager: &mut TabManager, config: &LoginConfig) {
        if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
            let mouse_pos = rl.get_mouse_position();

            if Self::handle_tab_clicks(mouse_pos, tab_manager, config) {
                return;
            }

            Self::handle_field_clicks(mouse_pos, tab_manager, config);
        }
    }

    fn handle_tab_clicks(
        mouse_pos: raylib::prelude::Vector2,
        tab_manager: &mut TabManager,
        config: &LoginConfig,
    ) -> bool {
        let tab_x = config.base_x;
        let tab_y = config.top_y + config.tab_spacing;

        // Login tab
        if Self::point_in_rect(mouse_pos, tab_x, tab_y, config.tab_width, config.tab_height) {
            tab_manager.switch_tab(TabType::Login);
            return true;
        }

        // Register tab
        if Self::point_in_rect(
            mouse_pos,
            tab_x + config.tab_width,
            tab_y,
            config.tab_width,
            config.tab_height,
        ) {
            tab_manager.switch_tab(TabType::Register);
            return true;
        }

        false
    }

    fn handle_field_clicks(
        mouse_pos: raylib::prelude::Vector2,
        tab_manager: &mut TabManager,
        config: &LoginConfig,
    ) {
        let user_y = config.top_y + config.field_spacing;
        let pass_y = user_y + 100.0;

        let fields = tab_manager.get_current_fields();

        // Username field
        if Self::point_in_rect(
            mouse_pos,
            config.base_x,
            user_y,
            config.box_width,
            config.box_height,
        ) {
            fields.username.entering = true;
            fields.password.entering = false;
        }
        // Password field
        else if Self::point_in_rect(
            mouse_pos,
            config.base_x,
            pass_y,
            config.box_width,
            config.box_height,
        ) {
            fields.username.entering = false;
            fields.password.entering = true;
        }
    }

    fn point_in_rect(
        point: raylib::prelude::Vector2,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) -> bool {
        point.x >= x && point.x <= x + width && point.y >= y && point.y <= y + height
    }
}

// Handles user input fields for login/register
struct InputField {
    label: &'static str,  // Field label
    value: String,        // User input value
    entering: bool,       // Is this field currently active?
    warning_text: String, // Warning message
    warning: bool,        // Warning flag
}

impl InputField {
    // Create a new input field
    fn new(label: &'static str, entering: bool) -> Self {
        Self {
            label,
            value: String::new(),
            entering,
            warning_text: String::new(),
            warning: false,
        }
    }

    // Handle keyboard input for this field
    fn handle_input(&mut self, key: KeyboardKey, shift: bool) {
        match key {
            KeyboardKey::KEY_BACKSPACE => {
                self.value.pop();
                self.warning_text.clear();
                self.warning = false;
            }
            KeyboardKey::KEY_SPACE => {
                self.value.push(' ');
                self.warning_text.clear();
                self.warning = false;
            }
            _ => {
                if let Some(c) = key_to_char(key, shift) {
                    self.value.push(c);
                    self.warning_text.clear();
                    self.warning = false;
                }
            }
        }
    }

    // Draw the input field UI, with optional masking for password fields
    fn draw(
        &self,
        d: &mut RaylibDrawHandle,
        font: raylib::ffi::Font,
        base_x: f32,
        base_y: f32,
        box_width: f32,
        box_height: f32,
        highlight_color: Color,
        mask: bool, // true for password fields
    ) {
        d.draw_text(
            self.label,
            base_x as i32,
            (base_y + 5.0) as i32,
            30,
            Color::alpha(&Color::WHITE, 0.9),
        );
        d.draw_rectangle_lines(
            base_x as i32,
            (base_y + 40.0) as i32,
            box_width as i32,
            box_height as i32,
            if self.entering {
                highlight_color
            } else {
                Color::GRAY
            },
        );
        // Show masked or plain text
        let text = if mask {
            "*".repeat(self.value.len())
        } else {
            self.value.clone()
        };

        // Find the starting byte index for the substring that fits in the box.
        let mut start_byte_index = 0;
        let mut current_width = 0.0;
        for (i, c) in text.char_indices().rev() {
            let s = CString::new(c.to_string()).unwrap();
            let char_width = unsafe { MeasureTextEx(font, s.as_ptr(), 30.0, 1.0).x };
            if current_width + char_width + 10.0 > box_width {
                start_byte_index = i + c.len_utf8();
                break;
            }
            current_width += char_width;
        }

        // Get the visible part of the string using the safe byte index.
        let visible = &text[start_byte_index..];

        let display = if self.entering {
            format!("{visible}|")
        } else {
            visible.to_string()
        };
        let display_c = CString::new(display).unwrap();
        let text_color = if self.entering {
            highlight_color
        } else {
            Color::alpha(&Color::WHITE, 0.7)
        };
        unsafe {
            DrawTextEx(
                font,
                display_c.as_ptr(),
                Vector2 {
                    x: base_x + 5.0,
                    y: base_y + 45.0,
                },
                30.0,
                0.1,
                text_color.into(),
            );
        }
    }
}

// Footer notes for login and register
const LOGIN_FOOTER: &str = "Welcome to Deemak by DBD! Use up/down keys to switch focus. Press Enter to continue. Not registered yet? Press Tab to switch from login to register.";
const REGISTER_FOOTER: &str = "Welcome to Deemak by DBD! Use up/down keys to switch focus. Press Enter to submit. Already registered? Press Tab to switch from register to login.";

// Authentication handler
struct AuthHandler;

impl AuthHandler {
    fn handle_login(fields: &mut FieldPair, users: &[crate::utils::auth::User]) -> Option<bool> {
        if fields.username.entering {
            if !fields.username.value.is_empty() {
                fields.username.entering = false;
                fields.password.entering = true;
                fields.username.warning = false;
            }
        } else if fields.password.entering {
            if !fields.password.value.is_empty() {
                let username = fields.username.value.trim();
                let password = fields.password.value.trim();
                if let Some(user) = users.iter().find(|u| u.username == username) {
                    if crate::utils::auth::verify_password(
                        &password.to_string(),
                        &user.salt,
                        &user.password_hash,
                    ) {
                        // Create and authenticate UserInfo
                        let mut user_info = UserInfo::new(
                            username.to_string(),
                            user.salt.clone(),
                            user.password_hash.clone(),
                        );
                        user_info.authenticate(); // Mark as authenticated with timestamp

                        // Set global user info
                        set_user_info(user_info).ok();
                        return Some(true);
                    } else {
                        fields.password.warning = true;
                        fields.password.warning_text = "Invalid password!".to_string();
                    }
                } else {
                    fields.username.warning = true;
                    fields.username.warning_text = "Username not found!".to_string();
                }
            }
        }
        None
    }

    fn handle_register(
        fields: &mut FieldPair,
        users: &mut Vec<crate::utils::auth::User>,
    ) -> Option<bool> {
        if fields.username.entering {
            if !fields.username.value.is_empty() {
                fields.username.entering = false;
                fields.password.entering = true;
                fields.username.warning = false;
            }
        } else if fields.password.entering {
            if !fields.password.value.is_empty() {
                let username = fields.username.value.trim();
                let password = fields.password.value.trim();
                if users.iter().any(|u| u.username == username) {
                    fields.username.warning = true;
                    fields.username.warning_text = "Username already exists!".to_string();
                } else {
                    match crate::utils::auth::hash_password(password) {
                        Ok((salt, hash)) => {
                            users.push(crate::utils::auth::User {
                                username: username.to_string(),
                                salt: salt.clone(),
                                password_hash: hash.clone(),
                            });
                            if let Err(_) =
                                std::panic::catch_unwind(|| crate::utils::auth::save_users(users))
                            {
                                fields.username.warning = true;
                                fields.username.warning_text = "Failed to save user!".to_string();
                            } else {
                                // Create and authenticate UserInfo
                                let mut user_info =
                                    UserInfo::new(username.to_string(), salt.clone(), hash.clone());
                                user_info.authenticate(); // Mark as authenticated with timestamp

                                // Set global user info
                                set_user_info(user_info).ok();
                                return Some(true);
                            }
                        }
                        Err(_) => {
                            fields.username.warning = true;
                            fields.username.warning_text = "Failed to hash password!".to_string();
                        }
                    }
                }
            }
        }
        None
    }
}

// Main login/register UI loop
pub fn show_login(rl: &mut RaylibHandle, thread: &RaylibThread, _font_size: f32) -> bool {
    // Load custom font
    let font = unsafe {
        let path = CString::new("fontbook/fonts/ttf/JetBrainsMono-Medium.ttf").unwrap();
        LoadFontEx(
            path.as_ptr() as *const c_char,
            600 as c_int,
            std::ptr::null_mut::<c_int>(),
            0,
        )
    };
    let font_d = rl.get_font_default();

    // Load users and initialize components
    let users_result = std::panic::catch_unwind(|| crate::utils::auth::load_users());
    let mut users = match users_result {
        Ok(u) => u,
        Err(_) => {
            eprintln!("Failed to load users. User database may be corrupted.");
            Vec::new()
        }
    };

    let config = LoginConfig::default();
    let mut animation = LoginAnimation::new(&config);
    let mut tab_manager = TabManager::new(users.is_empty());

    unsafe {
        SetExitKey(0i32); // Disable exit key (ESC) to prevent accidental exit during login
    }

    // Main event loop
    while !rl.window_should_close() {
        let input = rl.get_key_pressed();
        animation.update();

        // Handle input for login/register fields
        if animation.show_input {
            // Handle mouse input
            MouseHandler::handle_clicks(rl, &mut tab_manager, &config);

            // Handle keyboard input
            if let Some(key) = input {
                let shift = rl.is_key_down(KeyboardKey::KEY_LEFT_SHIFT)
                    || rl.is_key_down(KeyboardKey::KEY_RIGHT_SHIFT);
                match key {
                    KeyboardKey::KEY_TAB => {
                        tab_manager.switch_tab(match tab_manager.active_tab {
                            TabType::Login => TabType::Register,
                            TabType::Register => TabType::Login,
                        });
                    }
                    KeyboardKey::KEY_DOWN | KeyboardKey::KEY_UP => {
                        tab_manager.get_current_fields().toggle_focus();
                    }
                    KeyboardKey::KEY_ENTER => {
                        let result = match tab_manager.active_tab {
                            TabType::Login => {
                                AuthHandler::handle_login(&mut tab_manager.login_fields, &users)
                            }
                            TabType::Register => AuthHandler::handle_register(
                                &mut tab_manager.register_fields,
                                &mut users,
                            ),
                        };
                        if let Some(success) = result {
                            return success;
                        }
                    }
                    _ => {
                        // Handle character input
                        let fields = tab_manager.get_current_fields();
                        if fields.username.entering {
                            fields.username.handle_input(key, shift);
                        } else {
                            fields.password.handle_input(key, shift);
                        }
                    }
                }
            }
        }

        // Begin drawing UI
        let mut d = rl.begin_drawing(thread);
        d.clear_background(Color::BLACK);
        let highlight_color = Color::GOLD;

        // Draw welcome text
        if animation.stream_index > 0 {
            d.draw_text(
                &animation.displayed_text,
                200,
                config.top_y as i32,
                20,
                Color::alpha(&Color::GRAY, 1.0),
            );
        }

        // Draw title
        if animation.stream_index >= "WELCOME TO".len() {
            d.draw_text_ex(
                &font_d,
                "DEEMAK SHELL",
                raylib::prelude::Vector2 {
                    x: 200.0,
                    y: animation.y_offset,
                },
                60.0,
                2.0,
                Color::WHITE,
            );
        }

        if animation.show_input {
            let screen_width = d.get_screen_width();
            let screen_height = d.get_screen_height();

            // Draw tabs
            let tab_x = config.base_x;
            let tab_y = config.top_y + config.tab_spacing;

            d.draw_rectangle(
                tab_x as i32,
                tab_y as i32,
                config.tab_width as i32,
                config.tab_height as i32,
                if tab_manager.active_tab == TabType::Login {
                    highlight_color
                } else {
                    Color::alpha(&Color::GRAY, 0.3)
                },
            );
            d.draw_rectangle(
                (tab_x + config.tab_width) as i32,
                tab_y as i32,
                config.tab_width as i32,
                config.tab_height as i32,
                if tab_manager.active_tab == TabType::Register {
                    highlight_color
                } else {
                    Color::alpha(&Color::GRAY, 0.3)
                },
            );

            d.draw_text_ex(
                &font_d,
                "Login",
                raylib::prelude::Vector2 {
                    x: tab_x + 40.0,
                    y: tab_y + 8.0,
                },
                24.0,
                1.0,
                if tab_manager.active_tab == TabType::Login {
                    Color::BLACK
                } else {
                    Color::WHITE
                },
            );
            d.draw_text_ex(
                &font_d,
                "Register",
                raylib::prelude::Vector2 {
                    x: tab_x + config.tab_width + 40.0,
                    y: tab_y + 8.0,
                },
                24.0,
                1.0,
                if tab_manager.active_tab == TabType::Register {
                    Color::BLACK
                } else {
                    Color::WHITE
                },
            );

            // Draw current tab's fields
            let fields = tab_manager.get_current_fields();
            fields.draw_fields(&mut d, font, &config, highlight_color);

            // Draw divider and footer
            let footer_text = tab_manager.get_footer_text();
            let footer_height = calculate_footer_height(font, footer_text, screen_width);
            let divider_y = screen_height as f32 - footer_height - config.divider_margin;
            d.draw_line(
                30,
                divider_y as i32,
                screen_width - 30,
                divider_y as i32,
                Color::alpha(&Color::GRAY, 0.5),
            );
            draw_footer_legacy(
                &mut d,
                font,
                footer_text,
                screen_width,
                screen_height as f32,
            );

            // Draw version
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

    false // Login failed or window closed
}

// Helper function to calculate the total height needed for the footer
fn calculate_footer_height(font: raylib::ffi::Font, note: &str, screen_width: i32) -> f32 {
    let max_width = screen_width as f32 - 80.0;
    let font_size = 20.0;
    let spacing = 0.1;
    let words: Vec<&str> = note.split_whitespace().collect();
    let mut lines = Vec::new();
    let mut current_line = String::new();

    // Calculate all lines to know total height
    for word in words {
        let trial = if current_line.is_empty() {
            word.to_string()
        } else {
            format!("{current_line} {word}")
        };
        let trial_c = CString::new(trial.clone()).unwrap();
        let width = unsafe { MeasureTextEx(font, trial_c.as_ptr(), font_size, spacing).x };
        if width > max_width && !current_line.is_empty() {
            lines.push(current_line.clone());
            current_line = word.to_string();
        } else {
            current_line = trial;
        }
    }
    if !current_line.is_empty() {
        lines.push(current_line);
    }

    let line_height = font_size + 5.0;
    let total_height = lines.len() as f32 * line_height;
    total_height + 50.0 // Include the 50px bottom margin
}

// Draws a multi-line footer note, wrapping text to fit the screen width, positioned at bottom of screen
fn draw_footer_legacy(
    d: &mut RaylibDrawHandle,
    font: raylib::ffi::Font,
    note: &str,
    screen_width: i32,
    screen_height: f32,
) {
    let max_width = screen_width as f32 - 80.0;
    let font_size = 20.0;
    let spacing = 0.1;
    let x = 40.0;
    let words: Vec<&str> = note.split_whitespace().collect();
    let mut lines = Vec::new();
    let mut current_line = String::new();

    // Calculate all lines first to know total height
    for word in words {
        let trial = if current_line.is_empty() {
            word.to_string()
        } else {
            format!("{current_line} {word}")
        };
        let trial_c = CString::new(trial.clone()).unwrap();
        let width = unsafe { MeasureTextEx(font, trial_c.as_ptr(), font_size, spacing).x };
        if width > max_width && !current_line.is_empty() {
            lines.push(current_line.clone());
            current_line = word.to_string();
        } else {
            current_line = trial;
        }
    }
    if !current_line.is_empty() {
        lines.push(current_line);
    }

    // Calculate starting y position to align footer to bottom
    let line_height = font_size + 5.0;
    let total_height = lines.len() as f32 * line_height;
    let start_y = screen_height - total_height - 50.0; // 50px margin from bottom

    // Draw all lines
    for (i, line) in lines.iter().enumerate() {
        let y = start_y + (i as f32 * line_height);
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
    }
}
