use std::sync::Mutex;
use once_cell::sync::OnceCell;

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
