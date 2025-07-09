pub struct DummyPrompter;
pub trait UserPrompter {
    /// Returns true if user confirms (yes), false otherwise.
    fn confirm(&mut self, message: &str) -> bool;
    /// Returns a prompt message for the user.
    fn input(&mut self, message: &str) -> String;
}
impl UserPrompter for DummyPrompter {
    fn confirm(&mut self, _message: &str) -> bool {
        // Always return true for dummy prompter
        true
    }
    fn input(&mut self, _message: &str) -> String {
        // Return an empty string for dummy prompter
        String::new()
    }
}
