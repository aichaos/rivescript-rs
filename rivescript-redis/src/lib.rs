use std::collections::HashMap;

use async_trait::async_trait;
use rivescript_core::sessions::{SessionManager, History, ThawAction};
use redis::{self, AsyncCommands};
use redis::aio::MultiplexedConnection;

pub struct RedisSessionManager {
    client: redis::Client,
    prefix: String,
    max_history: isize,
    allow_dangerous: bool,
}

impl RedisSessionManager {

    /// Initialize the Redis session manager.
    ///
    /// ## Arguments
    ///
    /// * `connection_str` - the Redis connection string, like `redis://127.0.0.1/`
    /// * `prefix` - a prefix string for all Redis keys, default is `rivescript`
    pub fn new(connection_str: &str, prefix: &str) -> redis::RedisResult<Self> {
        let client = redis::Client::open(connection_str)?;
        let max_history: isize = (rivescript_core::MAX_HISTORY-1).try_into().unwrap();

        let mut prefix = prefix;
        if prefix.is_empty() {
            prefix = "rivescript";
        }

        Ok(Self{
            client,
            prefix: prefix.to_string(),
            max_history,
            allow_dangerous: false,
        })
    }

    /// Get an async MultiplexedConnection handle from the Redis client.
    pub async fn conn(&self) -> MultiplexedConnection {
        self.client.get_multiplexed_async_connection().await.unwrap()
    }

    /// Allow access to dangerous functions like `get_all` and `clear_all`.
    ///
    /// These functions can tank your Redis server performance because they incur O(N) operations
    /// which can be very slow on large datasets. It is not recommended to use these functions.
    ///
    /// By default, they will panic with `unimplemented!` when called.
    ///
    /// If you really want to use these functions, pass a true value to this function
    /// to allow the dangerous calls to run.
    pub fn allow_dangerous(&mut self, value: bool) {
        self.allow_dangerous = value;
    }

    fn user_key(&self, username: &str) -> String {
        format!("{}:user:{}", self.prefix, username)
    }

    fn history_key(&self, username: &str, kind: &str) -> String {
        format!("{}:history:{}:{}", self.prefix, username, kind)
    }

    fn freeze_key(&self, username: &str) -> String {
        format!("{}:freeze:{}", self.prefix, username)
    }
}

#[async_trait]
impl SessionManager for RedisSessionManager {
    async fn set(&self, username: &str, vars: HashMap<String, String>) {
        let mut conn = self.conn().await;
        let key = self.user_key(username);

        // Use HSET to store variables as Redis hash fields.
        if !vars.is_empty() {
            let _: () = conn.hset_multiple(key, &vars.into_iter().collect::<Vec<_>>()).await.unwrap();
        }
    }

    async fn add_history(&self, username: &str, input: &str, reply: &str) {
        let mut conn = self.conn().await;
        let in_key = self.history_key(username, "input");
        let re_key = self.history_key(username, "reply");

        // Push to list and trim to maintain max history.
        let _: () = redis::pipe()
            .lpush(&in_key, input)
            .ltrim(&in_key, 0, self.max_history)
            .lpush(&re_key, reply)
            .ltrim(&re_key, 0, self.max_history)
            .query_async(&mut conn)
            .await
            .unwrap();
    }

    async fn get_history(&self, username: &str) -> History {
        let mut conn = self.conn().await;
        let mut inputs: Vec<String> = conn.lrange(self.history_key(username, "input"), 0, self.max_history).await.unwrap_or_default();
        let mut replies: Vec<String> = conn.lrange(self.history_key(username, "reply"), 0, self.max_history).await.unwrap_or_default();

        // Pad out the history arrays with default "undefined" values to meet the expected length.
        inputs.resize(rivescript_core::MAX_HISTORY, rivescript_core::UNDEFINED.to_string());
        replies.resize(rivescript_core::MAX_HISTORY, rivescript_core::UNDEFINED.to_string());

        History {
            input: inputs,
            reply: replies,
        }
    }

    async fn get(&self, username: &str, name: &str) -> String {
        let mut conn = self.conn().await;
        let val: Option<String> = conn.hget(self.user_key(username), name).await.unwrap();
        val.unwrap_or_else(|| rivescript_core::UNDEFINED.to_string())
    }

    async fn get_any(&self, username: &str) -> HashMap<String, String> {
        let mut conn = self.conn().await;
        conn.hgetall(self.user_key(username)).await.unwrap_or_default()
    }

    /// Dangerously clear all stored Redis keys for RiveScript.
    ///
    /// WARNING: This is an O(N) operation. On large datasets, this may be very slow
    /// and it can lock your production Redis server.
    ///
    /// By default, this function will not allow you to call it and will panic with an
    /// `unimplemented!` error. If you're sure you know what you're doing, you may unlock
    /// this function by first calling `allow_dangerous(true)` on the RedisSessionManager.
    async fn get_all(&self) -> HashMap<String, HashMap<String, String>> {
        let mut conn = self.conn().await;
        let pattern = format!("{}:user:*", self.prefix);
        let mut all_data = HashMap::new();

        // Use SCAN to find all user keys without blocking the server.
        let mut iter: redis::AsyncIter<String> = conn.scan_match(&pattern).await.unwrap();

        let mut conn = self.conn().await;
        while let Some(key) = iter.next_item().await {
            let key = key.unwrap();

            // Extract the username from the key (e.g. "prefix:user:username" -> "username")
            if let Some(username) = key.strip_prefix(&format!("{}:user", self.prefix)) {
                let vars: HashMap<String, String> = conn.hgetall(&key).await.unwrap_or_default();
                all_data.insert(username.to_string(), vars);
            }
        }

        all_data
    }

    async fn clear(&self, username: &str) {
        let mut conn = self.conn().await;
        let _: () = conn.del(&[
            self.user_key(username),
            self.history_key(username, "input"),
            self.history_key(username, "reply"),
            self.freeze_key(username),
        ]).await.unwrap();
    }

    /// Dangerously clear all stored Redis keys for RiveScript.
    ///
    /// WARNING: This is an O(N) operation. On large datasets, this may be very slow
    /// and it can lock your production Redis server.
    ///
    /// This will delete all user data, history, and frozen states.
    ///
    /// By default, this function will not allow you to call it and will panic with an
    /// `unimplemented!` error. If you're sure you know what you're doing, you may unlock
    /// this function by first calling `allow_dangerous(true)` on the RedisSessionManager.
    async fn clear_all(&self) {
        // Danger zone!
        if !self.allow_dangerous {
            unimplemented!("This is an expensive operation and is not recommended. Call allow_dangerous(true) to enable this function.");
        }

        let mut conn = self.conn().await;
        let pattern = format!("{}:*", self.prefix);

        let mut iter: redis::AsyncIter<String> = conn.scan_match(&pattern).await.unwrap();
        let mut keys_to_delete = Vec::new();

        let mut conn = self.conn().await;
        while let Some(key) = iter.next_item().await {
            keys_to_delete.push(key.unwrap());

            // Delete in batches of 100 to keep memory usage low.
            if keys_to_delete.len() >= 100 {
                let _: () = conn.del(&keys_to_delete).await.unwrap();
                keys_to_delete.clear();
            }
        }

        // Clean up remaining keys.
        if !keys_to_delete.is_empty() {
            let _: () = conn.del(&keys_to_delete).await.unwrap();
            keys_to_delete.clear();
        }
    }

    async fn freeze(&self, username: &str) -> Result<bool, String> {
        let mut conn = self.conn().await;
        let current_vars = self.get_any(username).await;

        if current_vars.is_empty() {
            return Ok(false);
        }

        // Serialize the map to a JSON string for simple storage in a single key.
        let serialized = serde_json::to_string(&current_vars).map_err(|e| e.to_string())?;
        let _: () = conn.set(self.freeze_key(username), serialized)
            .await
            .map_err(|e| e.to_string())?;

        Ok(true)
    }

    async fn thaw(&self, username: &str, action: ThawAction) -> Result<bool, String> {
        let mut conn = self.conn().await;
        let f_key = self.freeze_key(username);

        let frozen: Option<String> = conn.get(&f_key)
            .await
            .map_err(|e| e.to_string())?;
        let data = match frozen {
            Some(d) => d,
            None => return Ok(false),
        };

        match action {
            ThawAction::Thaw | ThawAction::Keep => {
                let vars: HashMap<String, String> = serde_json::from_str(&data).map_err(|e| e.to_string())?;
                self.set(username, vars).await;
            },
            ThawAction::Discard => (),
        }

        if !matches!(action, ThawAction::Keep) {
            let _: () = conn.del(&f_key).await.map_err(|e| e.to_string())?;
        }

        Ok(true)
    }
}