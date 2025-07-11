use crate::commands::cmds::normalize_path;
use std::path::Path;

// Helper types and functions
pub enum TabCompletionResult {
    SingleMatch(String),
    CommonPrefix(String),
    DisplayCompletions {
        current_line: String,
        completion_lines: Vec<String>,
        should_display_all: bool,
    },
    NoAction,
}

pub fn process_tab_completion(
    parts: Vec<&str>,
    matches: Vec<String>,
    term_width: usize,
    term_height: usize,
    current_input: &str,
    prompt: Option<&str>,
) -> TabCompletionResult {
    let last_part_str = parts.last().unwrap();

    // Convert to normalized path and back to String
    let last_part = normalize_path(Path::new(last_part_str))
        .to_string_lossy()
        .into_owned();

    if matches.len() == 1 {
        // Single match - complete it
        let mut new_input = parts[..parts.len() - 1].join(" ");
        if !new_input.is_empty() {
            new_input.push(' ');
        }
        new_input.push_str(&matches[0]);
        TabCompletionResult::SingleMatch(new_input)
    } else if !matches.is_empty() {
        // Multiple matches - find common prefix
        let mut common_prefix = matches[0].clone();
        for m in &matches[1..] {
            common_prefix = common_prefix
                .chars()
                .zip(m.chars())
                .take_while(|(a, b)| a == b)
                .map(|(c, _)| c)
                .collect();
            if common_prefix.is_empty() {
                break;
            }
        }

        if common_prefix.len() > last_part.len() {
            let mut new_input = parts[..parts.len() - 1].join(" ");
            if !new_input.is_empty() {
                new_input.push(' ');
            }
            new_input.push_str(&common_prefix);
            TabCompletionResult::CommonPrefix(new_input)
        } else {
            // Prepare to display completions
            let current_line = if let Some(prompt) = prompt {
                format!("{prompt} {current_input}")
            } else {
                format!("> {current_input}")
            };

            // Calculate optimal column display
            let max_len = matches.iter().map(|s| s.len()).max().unwrap_or(0) + 2;
            let cols = (term_width / max_len).max(1);
            let rows = matches.len().div_ceil(cols);

            // Format completion lines
            let mut completion_lines = Vec::new();
            if rows > term_height.saturating_sub(4) {
                // Too many items - just return the matches and let the caller handle pagination
                TabCompletionResult::DisplayCompletions {
                    current_line,
                    completion_lines: matches,
                    should_display_all: true,
                }
            } else {
                // Format all completions at once
                for row in 0..rows {
                    let mut line = String::new();
                    for col in 0..cols {
                        let idx = row * cols + col;
                        if idx < matches.len() {
                            line.push_str(&format!("{:width$}", matches[idx], width = max_len));
                        }
                    }
                    completion_lines.push(line);
                }
                TabCompletionResult::DisplayCompletions {
                    current_line,
                    completion_lines,
                    should_display_all: false,
                }
            }
        }
    } else {
        TabCompletionResult::NoAction
    }
}
