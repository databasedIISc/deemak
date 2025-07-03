use std::ffi::{c_char, CString, c_int};
use std::sync::Mutex;
use once_cell::sync::OnceCell;
use raylib::ffi::{ColorFromHSV, DrawTextEx, Vector2, LoadFontEx};
use raylib::prelude::get_monitor_width;

pub static GLOBAL_OUTPUT_LINES: OnceCell<Mutex<Vec<String>>> = OnceCell::new();

pub fn init_output_lines(buffer: Vec<String>) {
    GLOBAL_OUTPUT_LINES.set(Mutex::new(buffer)).expect("Already initialized!");
}

pub fn print_deemak(line: String) {
    if let Some(output) = GLOBAL_OUTPUT_LINES.get() {
        let mut lines = output.lock().unwrap();
        lines.push(line);
    }
}

pub fn print_deemak_extend<I>(iter: I)
where
    I: IntoIterator<Item = String>,
{
    if let Some(output) = GLOBAL_OUTPUT_LINES.get() {
        let mut lines = output.lock().unwrap();
        lines.extend(iter);
    }
}

pub fn deemak_clear() {
    if let Some(output) = GLOBAL_OUTPUT_LINES.get() {
        let mut lines = output.lock().unwrap();
        lines.clear();
    }
}

pub fn print_deemak_at(x: f32, y: f32, text: &str) {
    let content = CString::new(text).unwrap();
    let pos = Vector2 { x, y };
    let font = unsafe {
        let path = CString::new("fontbook/fonts/ttf/JetBrainsMono-Medium.ttf").unwrap();

        LoadFontEx(
            path.as_ptr() as *const c_char,
            600.0 as c_int,
            std::ptr::null_mut::<c_int>(),
            0,
        )
    };
    let font_size = get_monitor_width(0) as f32 / 73.5;

    unsafe {
        DrawTextEx(
            font,
            content.as_ptr() as *const c_char,
            pos,
            font_size,
            1.2,
            ColorFromHSV(0.0, 0.0, 1.0),
        )
    }
}