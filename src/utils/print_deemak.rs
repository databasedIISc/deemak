use std::ffi::{c_char, CString};
use std::sync::Mutex;
use once_cell::sync::OnceCell;
use raylib::ffi::{ColorFromHSV, DrawTextEx, Vector2};
use crate::utils::global_font::{GLOBAL_FONT, GLOBAL_FONT_SIZE};

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
    let font = &GLOBAL_FONT.get().unwrap().0;
    let font_size = GLOBAL_FONT_SIZE.get().unwrap();

    unsafe {
        DrawTextEx(
            *font,
            content.as_ptr() as *const c_char,
            pos,
            *font_size,
            1.2,
            ColorFromHSV(0.0, 0.0, 1.0),
        )
    }
}