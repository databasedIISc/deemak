use crate::utils::log;
use std::collections::HashSet;

pub struct ArgParser {
    flags: HashSet<String>,
    args: Vec<String>,
}

impl ArgParser {
    /// Create a new ArgParser with specified valid flags and help text
    pub fn new(valid_flags: &[&str]) -> Self {
        ArgParser {
            flags: valid_flags.iter().map(|s| s.to_string()).collect(),
            args: Vec::new(),
        }
    }

    /// Get the flag arguments
    pub fn parse(&mut self, input_args: &[String]) -> Result<(), String> {
        self.args.clear();

        log::log_debug("Argparse", &format!("Input arguments: {:?}", input_args));
        for arg in input_args {
            // Check for help flags first
            if arg == "-h".trim() || arg == "--help".trim() {
                return Err("help".to_string());
            }

            // Check if it's a valid flag (starts with - and in our flags list)
            if arg.starts_with('-') {
                if self.flags.contains(arg) {
                    self.args.push(arg.clone());
                } else {
                    Err("unknown".to_string())?;
                }
            } else {
                // Otherwise, treat it as a positional argument
                self.args.push(arg.clone());
            }
        }

        Ok(())
    }

    /// Check if a flag was provided
    pub fn has_flag(&self, flag: &str) -> bool {
        self.args.iter().any(|arg| arg == flag)
    }

    /// Get positional arguments (non-flag arguments)
    pub fn get_positional_args(&self) -> Vec<&String> {
        self.args
            .iter()
            .filter(|arg| !arg.starts_with('-'))
            .collect()
    }

    /// Get all arguments (including flags)
    pub fn get_all_args(&self) -> &Vec<String> {
        &self.args
    }
}
