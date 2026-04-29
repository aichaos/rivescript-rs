use std::collections::HashMap;
use async_trait::async_trait;
use rivescript_core::macros;

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

}

#[async_trait]
impl<'a> macros::Proxy for Proxy<'a> {
    /// Returns the username of the current user who invokes the object macro.
    fn current_username(&self) -> String {
        self.username.clone()
    }

    /// Set a user variable for the current user.
    ///
    /// Note: only the current username can have variables set for them. If you pass a
    /// different username, it will return an error. This is due to the way that the Proxy
    /// holds onto 'staged' set_uservars (to commit them after your subroutine returns).
    ///
    /// You may `get_uservar()` for other users normally (as the get will come directly from
    /// RiveScript's user variable session store), but sets are staged only for the current user.
    async fn set_uservar(&mut self, username: &str, name: &str, value: &str) -> Result<bool, String> {
        if username != self.username {
            return Err("New user variables can only be set for the current_username()".to_string());
        }
        self.staged_user_vars.insert(name.to_string(), value.to_string());
        Ok(true)
    }

    /// Get a user variable for the current user.
    ///
    /// If you have recently `set_uservar()` within the same subroutine, this will
    /// return the cached value you had last set. Otherwise, it will look up the
    /// current value from the RiveScript user variable session store.
    async fn get_uservar(&self, username: &str, name: &str) -> String {
        if let Some(value) = self.staged_user_vars.get(name) {
            return value.clone();
        }
        self.rs.sessions.get(&username, name).await
    }

    /// Get all stored variables about the user.
    async fn get_uservars(&self, _username: &str) -> HashMap<String, String> {
        self.rs.sessions.get_any(&self.rs.current_username().unwrap()).await
    }

    /// Set a bot variable.
    ///
    /// Bot variables are 'global' to the RiveScript instance and shared between
    /// all users. This is equivalent to the `<bot name=value>` tag.
    fn set_variable(&mut self, name: &str, value: &str) {
        self.staged_bot_vars.insert(name.to_string(), value.to_string());
    }

    /// Get a bot variable.
    ///
    /// This is equivalent to the `<bot name>` tag.
    fn get_variable(&self, name: &str) -> String {
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
    fn finish(&mut self, output: String) -> Result<macros::SubroutineResult, String> {
        Ok(macros::SubroutineResult {
            output,
            staged_user_vars: std::mem::take(&mut self.staged_user_vars),
            staged_bot_vars: std::mem::take(&mut self.staged_bot_vars),
        })
    }
}