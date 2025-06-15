pub struct DummyPrompter;
pub trait UserPrompter {
    /// Returns true if user confirms (yes), false otherwise.
    fn confirm(&mut self, message: &str) -> bool;
}
impl UserPrompter for DummyPrompter {
    fn confirm(&mut self, _message: &str) -> bool {
        // Always return true for dummy prompter
        true
    }
}