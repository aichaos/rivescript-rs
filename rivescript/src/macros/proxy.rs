use std::collections::HashMap;

use crate::RiveScript;

/// RiveScript Proxy for calling object macro subroutines.
///
/// In most implementations of RiveScript, object macros are given as their
/// first argument a reference to the master RiveScript struct, so that the
/// subroutine can get/set user variables and things.
///
/// It is trickier in Rust to pass a mutable borrow of the RiveScript struct.
/// The Proxy instead implements a subset of useful methods from RiveScript
/// to allow access to bot/user variables, including setting new variables,
/// which get resolved after the subroutine returns.
pub struct Proxy<'a> {
    rs: &'a RiveScript,
    username: String,
    staged_user_vars: HashMap<String, String>,
    staged_bot_vars: HashMap<String, String>,
}

pub struct SubroutineResult {
    pub output: String,
    pub staged_user_vars: HashMap<String, String>,
    pub staged_bot_vars: HashMap<String, String>,
}

impl<'a> Proxy<'a> {

    /// Create a new Proxy from the current RiveScript instance and username.
    pub fn new(rs: &'a RiveScript, username: String) -> Self {
        Self {
            rs: rs,
            username: username,
            staged_user_vars: HashMap::new(),
            staged_bot_vars: HashMap::new(),
        }
    }

    /// Returns the username of the current user who invokes the object macro.
    pub fn current_username(&mut self) -> Result<String, String> {
        self.rs.current_username()
    }

    /// Set a user variable for the current user.
    pub async fn set_uservar(&mut self, name: &str, value: &str) {
        self.staged_user_vars.insert(name.to_string(), value.to_string());
    }

    /// Get a user variable for the current user.
    ///
    /// If you have recently `set_uservar()` within the same subroutine, this will
    /// return the cached value you had last set. Otherwise, it will look up the
    /// current value from the RiveScript user variable session store.
    pub async fn get_uservar(&self, name: &str) -> String {
        if let Some(value) = self.staged_user_vars.get(name) {
            return value.clone();
        }
        self.rs.sessions.get(&self.username, name).await
    }

    /// Set a bot variable.
    ///
    /// Bot variables are 'global' to the RiveScript instance and shared between
    /// all users. This is equivalent to the `<bot name=value>` tag.
    pub fn set_variable(&mut self, name: &str, value: &str) {
        self.staged_bot_vars.insert(name.to_string(), value.to_string());
    }

    /// Get a bot variable.
    ///
    /// This is equivalent to the `<bot name>` tag.
    pub fn get_variable(&self, name: &str) -> String {
        if let Some(value) = self.staged_bot_vars.get(name) {
            return value.clone();
        }
        self.rs.brain.get_bot_var(name)
    }

    /// Return a response from the object macro subroutine.
    ///
    /// The original `<call>` tag that invoked your subroutine will be replaced
    /// with the value returned here. Return an empty string if you don't want
    /// any extra output to be sent to the user.
    ///
    /// Internally, this function exports the staged bot/user variables back to
    /// the parent RiveScript struct so that any written variables can be
    /// committed back to their proper storage containers.
    pub fn finish(&mut self, output: String) -> Result<SubroutineResult, String> {
        Ok(SubroutineResult {
            output,
            staged_user_vars: std::mem::take(&mut self.staged_user_vars),
            staged_bot_vars: std::mem::take(&mut self.staged_bot_vars),
        })
    }
}