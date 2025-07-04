use crate::utils::globals::SHELL_HISTORY;

// To add to history:
pub fn add_to_history(input: &str) {
    if let Ok(mut history) = SHELL_HISTORY.lock() {
        // if the current input is same as the last one, do not add it again
        if let Some(last) = history.last() {
            if last == input {
                return;
            }
        }
        history.push(input.to_string());
    }
}

// To read history:
pub fn get_history() -> Vec<String> {
    SHELL_HISTORY
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone()
}
