use std::collections::VecDeque;
use std::{collections::HashMap, sync::RwLock};

use async_trait::async_trait;

use crate::sessions::{SessionManager, History};
use crate::sessions::ThawAction::*;

pub struct MemorySession {
    // Map of Username -> (Map of Key -> Value)
    // users: RwLock<HashMap<String, HashMap<String, String>>>,
    users: RwLock<HashMap<String, UserData>>,
}

#[derive(Clone)]
struct UserData {
    vars: HashMap<String, String>,
    frozen: HashMap<String, String>,
    history_input: VecDeque<String>,
    history_reply: VecDeque<String>,
}

impl UserData {
    pub fn new() -> Self {
        let mut input = VecDeque::with_capacity(rivescript_core::MAX_HISTORY);
        let mut reply = VecDeque::with_capacity(rivescript_core::MAX_HISTORY);

        for _ in 0..rivescript_core::MAX_HISTORY {
            input.push_back(rivescript_core::UNDEFINED.to_string());
            reply.push_back(rivescript_core::UNDEFINED.to_string());
        }

        Self {
            vars: HashMap::new(),
            frozen: HashMap::new(),
            history_input: input,
            history_reply: reply,
        }
    }
}

impl MemorySession {
    pub fn new() -> Self {
        Self {
            users: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl SessionManager for MemorySession {

    async fn set(&self, username: &str, vars: HashMap<String, String>) {
        let mut store = self.users.write().expect("RwLock poisoned");
        let user_entry = store.entry(username.to_string()).or_insert_with(|| UserData::new());
        user_entry.vars.extend(vars);
    }

    async fn add_history(&self, username: &str, input: &str, reply: &str) {
        let mut store = self.users.write().expect("RwLock poisoned");
        let user_entry = store.entry(username.to_string()).or_insert_with(|| UserData::new());

        // Remove the oldest entry (the back).
        user_entry.history_input.pop_back();
        user_entry.history_reply.pop_back();

        // Add the newest entry (the front).
        user_entry.history_input.push_front(input.trim().to_string());
        user_entry.history_reply.push_front(reply.trim().to_string());
    }

    async fn get_history(&self, username: &str) -> History {
        let store = self.users.read().expect("RwLock poisoned");
        store.get(username)
            .map(|user_data| History {
                input: user_data.history_input.iter().cloned().collect(),
                reply: user_data.history_reply.iter().cloned().collect(),
            }).unwrap_or_default()
    }

    async fn get(&self, username: &str, name: &str) -> String {
        let store = self.users.read().expect("RwLock poisoned");
        store.get(username)
            .and_then(|user_data| user_data.vars.get(name))
            .cloned()
            .unwrap_or_else(|| rivescript_core::UNDEFINED.to_string())
    }

    async fn get_any(&self, username: &str) -> HashMap<String, String> {
        let store = self.users.read().expect("RwLock poisoned");
        store.get(username)
            .map(|user_data| user_data.vars.clone())
            .unwrap_or_else(|| HashMap::new())
    }

    async fn get_all(&self) -> HashMap<String, HashMap<String, String>> {
        let store = self.users.read().expect("RwLock poisoned");
        store.iter()
            .map(|(username, data)| {
                (username.clone(), data.vars.clone())
            })
            .collect()
    }

    async fn clear(&self, username: &str) {
        let mut store = self.users.write().expect("RwLock poisoned");
        store.remove(username);
    }

    async fn clear_all(&self) {
        let mut store = self.users.write().expect("RwLock poisoned");
        store.clear();
    }

    async fn freeze(&self, username: &str) -> Result<bool, String> {
        let mut store = self.users.write().expect("RwLock poisoned");

        if let Some(user_data) = store.get_mut(username) {
            user_data.frozen.clear();
            user_data.frozen = user_data.vars.clone();
        } else {
            return Err("no user data found".to_string());
        }

        Ok(true)
    }

    async fn thaw(&self, username: &str, action: crate::sessions::ThawAction) -> Result<bool, String> {
        let mut store = self.users.write().expect("RwLock poisoned");

        if let Some(user_data) = store.get_mut(username) {

            // What are we doing with the frozen variables?
            match action {
                Discard => {
                    user_data.frozen.clear();
                }
                Thaw | Keep => {
                    user_data.vars.clear();
                    user_data.vars = user_data.frozen.clone();

                    if !matches!(action, Keep) {
                        user_data.frozen.clear();
                    }
                },
            }

        } else {
            return Err("no user data found".to_string());
        }

        Ok(true)
    }
}
