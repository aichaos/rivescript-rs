//! Implementation of the RiveScript chatbot scripting language.
//!
//! RiveScript is a simple scripting language designed for implementing
//! chatbots that communicate with a user through plain language. This
//! module provides an official RiveScript engine for Rust written by the
//! language author.

use crate::ast::AST;
use crate::parser::Parser;
use log::{debug, warn};
use std::{collections::HashMap, error::Error, fs, string, sync::Arc};
use Result::Ok;

mod ast;
mod errors;
mod parser;
mod regex;
mod reply;
mod sessions;
mod sorting;
mod tags;
mod tests;
mod utils;

/// Rust library version.
pub const VERSION: &str = "0.1.0";

// Various internal constants.
const DEFAULT_TOPIC: &str = "random";
const BEGIN_TOPIC: &str = "__begin__";
const ERR_NO_MATCH: &str = "[ERR: No Trigger Matched]";
const ERR_NO_REPLY: &str = "[ERR: No Reply]";
const BEGIN_REQUEST: &str = "request";
const TAG_OK: &str = "{ok}";
const UNDEFINED: &str = "undefined";
const MAX_STARS: usize = 9;
const MAX_HISTORY: usize = 9;

/// RiveScript represents a single chatbot personality in memory.
pub struct RiveScript {
    pub debug: bool,
    pub utf8: bool,
    pub depth: usize,
    pub case_sensitive: bool,

    pub sessions: Arc<dyn sessions::SessionManager + Send + Sync>,
    parser: Parser,
    brain: AST,
    sorted_topics: HashMap<String, Vec<ast::Trigger>>,
    sorted_thats: HashMap<String, Vec<ast::Trigger>>,
    sorted_subs: Vec<String>,
    sorted_person: Vec<String>,

    // Runtime (in-reply) variables.
    in_reply_context: bool,
    current_username: String,
}

impl RiveScript {
    /// Initialize a new RiveScript chatbot personality.
    ///
    /// A single instance of RiveScript is able to have its own set of responses ("brain") independently
    /// of other instances of RiveScript. Also, by default, RiveScript keeps track of temporary user
    /// variables (such as recent reply history and any variables the bot has learned about them) at
    /// in local memory of this instance, with each instance keeping its own separate data store.
    pub fn new() -> Self {
        Self {
            debug: false,
            utf8: false,
            depth: 50,
            case_sensitive: false,

            sessions: Arc::new(sessions::memory::MemorySession::new()),
            parser: Parser::new(),
            brain: AST::new(),
            sorted_topics: HashMap::new(),
            sorted_thats: HashMap::new(),
            sorted_subs: Vec::new(),
            sorted_person: Vec::new(),

            in_reply_context: false,
            current_username: String::new(),
        }
    }

    /// Replace the default in-memory User Variable Session manager with an alternative.
    pub fn set_session_manager(&mut self, manager: impl sessions::SessionManager + Send + Sync + 'static) {
        self.sessions = Arc::new(manager);
    }

    /// Load a directory of RiveScript documents (.rive or .rs extension) from a folder on disk.
    /// Example
    /// ```rust
    /// # use rivescript::RiveScript;
    /// # fn main() {
    ///     let mut bot = RiveScript::new();
    ///     bot.load_directory("./eg/brain").expect("Couldn't load directory!");
    /// # }
    /// ```
    pub fn load_directory(&mut self, path: &str) -> Result<bool, Box<dyn Error>> {
        debug!("load_directory called on: {}", path);

        let paths = fs::read_dir(path)?;

        for filename in paths {
            let filepath = match filename {
                Ok(res) => res.path(),
                Err(err) => return Err(Box::new(err)),
            };

            match filepath.extension() {
                Some(ext) => {
                    if ext.eq_ignore_ascii_case("rive") || ext.eq_ignore_ascii_case(".rs") {
                        self.load_file(filepath.as_path().display().to_string().as_str())?;
                        // match self.load_file(filepath.as_path().display().to_string().as_str()) {
                        //     Ok(_) => continue,
                        //     Err(err) => return Err(err),
                        // }
                    }
                }
                None => continue,
            }
        }

        return Ok(true);
    }

    /// Load a RiveScript document by filename on disk.
    /// Example
    /// ```rust
    /// # use rivescript::RiveScript;
    /// # fn main() {
    ///     let mut bot = RiveScript::new();
    ///     bot.load_file("./eg/brain/eliza.rive").expect("Couldn't load file from disk!");
    /// # }
    /// ```
    pub fn load_file(&mut self, path: &str) -> Result<bool, Box<dyn Error>> {
        debug!("load_file called on: {}", path);
        let contents = fs::read_to_string(path)?;
        self._stream(path, contents)
    }

    /// Stream a string containing RiveScript syntax into the bot, rather than read from the filesystem.
    /// Example
    /// ```rust
    /// # use rivescript::RiveScript;
    /// # fn main() {
    ///     let mut bot = RiveScript::new();
    ///     let code = String::from(
    ///         "
    ///         + hello bot
    ///         - Hello, human!
    ///         ",
    ///     );
    ///     bot.stream(code).expect("Couldn't parse code!");
    /// # }
    /// ```
    pub fn stream(&mut self, source: String) -> Result<bool, Box<dyn Error>> {
        self._stream("stream()", source)
    }

    // Internal, centralized funnel to load a RiveScript document.
    fn _stream(&mut self, filename: &str, source: String) -> Result<bool, Box<dyn Error>> {
        let ast = self.parser.parse(filename, source)?;
        self.brain.extend(ast);
        Ok(true)
    }

    /// Sort the internal data structures for optimal matching.
    pub fn sort_triggers(&mut self) {
        warn!("sort_triggers called, final AST is: {:#?}", self.brain);
        match sorting::sort_triggers(&self.brain) {
            Ok(result) => {
                self.sorted_topics = result.topics;
                self.sorted_thats = result.thats;
                self.sorted_subs = result.subs;
                self.sorted_person = result.person;
            },
            Err(_) => (),
        }

        // DEBUG
        debug!("sorted_topics: {:#?}", self.sorted_topics);
        debug!("sorted_thats: {:#?}", self.sorted_thats);
        debug!("sorted_subs: {:#?}", self.sorted_subs);
        debug!("sorted_person: {:#?}", self.sorted_person);
    }
    /// Get a reply from the chatbot.
    pub async fn reply(&mut self, username: &str, message: &str) -> Result<String, String> {
        // let msg = reply::Message{
        //     username: String::from("username"),
        // }
        reply::reply(self, username, message).await
    }
}
