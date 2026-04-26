use async_trait::async_trait;
use std::collections::HashMap;

pub mod memory;

/// What to do with frozen user variables after a thaw?
pub enum ThawAction {
    // Thaw means to restore the variables and erase the frozen copy.
    Thaw,

    // Discard means to cancel the frozen copy and not restore them.
    Discard,

    // Keep means to restore the variables but still keep the frozen copy.
    Keep,
}

#[async_trait]
pub trait SessionManager: Send + Sync {
    // Set one or more user variables from a hashmap.
    async fn set(&self, username: &str, vars: HashMap<String, String>);

    // Add a message exchange to the user's history.
    async fn add_history(&self, username: &str, input: &str, reply: &str);

    // Get the user's recent 9 inputs and replies.
    async fn get_history(&self, username: &str) -> History;

    // Get a user variable, or return "undefined" if not set.
    async fn get(&self, username: &str, name: &str) -> String;

    // Get all variables for a user.
    async fn get_any(&self, username: &str) -> HashMap<String, String>;

    // Get all variables about all users.
    async fn get_all(&self) -> HashMap<String, HashMap<String, String>>;

    // Clear all variables for a given user.
    async fn clear(&self, username: &str);

    // Clear all variables about all users.
    async fn clear_all(&self);

    // Freeze a snapshot of user variables that can later be thawed.
    async fn freeze(&self, username: &str) -> Result<bool, String>;

    // Thaw frozen user variables, putting them back in place.
    async fn thaw(&self, username: &str, action: ThawAction) -> Result<bool, String>;
}

#[derive(Debug, Clone)]
pub struct History {
    pub input: Vec<String>,
    pub reply: Vec<String>,
}

impl Default for History {
    fn default() -> Self {
        Self {
            input: vec!["undefined".to_string(); rivescript_core::MAX_HISTORY],
            reply: vec!["undefined".to_string(); rivescript_core::MAX_HISTORY],
        }
    }
}