use crate::utils::prompt::UserPrompter;

pub fn exit(prompter: &mut dyn UserPrompter) -> (bool, String) {
    if !prompter.confirm(
        "Are you sure you want to exit? Make sure you have saved your progress before exiting.",
    ) {
        (false, "Exit cancelled by user.".to_string())
    } else {
        // Perform any necessary cleanup here
        // For example, saving state, closing files, etc.
        // This is a placeholder for any cleanup logic you might need.
        (true, "Exiting the application. Goodbye!".to_string())
    }
}
