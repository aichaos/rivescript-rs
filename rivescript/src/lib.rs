//! Implementation of the RiveScript chatbot scripting language.
//!
//! RiveScript is a simple scripting language designed for implementing
//! chatbots that communicate with a user through plain language. This
//! module provides an official RiveScript engine for Rust written by the
//! language author.

use crate::ast::AST;
use crate::macros::proxy::Proxy;
use log::{debug, warn};
use rivescript_core::macros::{LanguageLoader, SubroutineResult};
use rivescript_core::{sessions, parser::Parser};
use std::{collections::HashMap, error::Error, fs, sync::Arc};
use futures::future::BoxFuture;
use Result::Ok;

use rivescript_core::{DEFAULT_DEPTH, ast};
mod errors;
mod inheritance;
mod macros;
mod reply;
mod sorting;
mod tags;
mod tests;
mod utils;

/// Rust library version.
pub const VERSION: &str = "0.3.0";

/// Loader for the JavaScript object macro parser (optional builtin feature).
#[cfg(feature = "javascript")]
pub fn register_default_js_handler(rs: &mut RiveScript) {
    use rivescript_js::JavaScriptLoader;
    rs.set_handler("javascript", JavaScriptLoader::new());
}


/// RiveScript represents a single chatbot personality in memory.
pub struct RiveScript {
    pub debug: bool,
    pub utf8: bool,
    pub depth: usize,
    pub case_sensitive: bool,
    unicode_punctuation: ::regex::Regex,

    pub sessions: Arc<dyn sessions::SessionManager + Send + Sync>,
    parser: Parser,
    brain: AST,
    sorted_topics: HashMap<String, Vec<ast::Trigger>>,
    sorted_thats: HashMap<String, Vec<ast::Trigger>>,
    sorted_subs: Vec<String>,
    sorted_person: Vec<String>,
    macro_handlers: HashMap<String, Box<dyn LanguageLoader>>,
    subroutines: HashMap<String, macros::Subroutine>,
    object_langs: HashMap<String, String>,

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
            depth: DEFAULT_DEPTH,
            case_sensitive: false,
            unicode_punctuation: ::regex::Regex::new(r"[.,!?;:]").unwrap(),

            sessions: Arc::new(sessions::memory::MemorySession::new()),
            parser: Parser::new(),
            brain: AST::new(),
            sorted_topics: HashMap::new(),
            sorted_thats: HashMap::new(),
            sorted_subs: Vec::new(),
            sorted_person: Vec::new(),
            macro_handlers: HashMap::new(),
            subroutines: HashMap::new(),
            object_langs: HashMap::new(),

            in_reply_context: false,
            current_username: String::new(),
        }
    }

    /// Replace the Unicode punctuation regexp when running with UTF-8 mode enabled.
    ///
    /// In UTF-8 mode, the user's message is (for the most part) left untouched, with only
    /// backslashes and HTML angle brackets stripped. This can cause matching errors though
    /// if common punctuation symbols were left intact, for example, a trigger that looks for
    /// `+ hello bot` might not match the string "Hello bot." because of the period at the end.
    ///
    /// The default regexp is `[.,!?;:]` which matches common English punctuation symbols to be
    /// removed. In case you need to customize this, you can provide your own regexp here.
    pub fn set_unicode_punctuation(&mut self, re: ::regex::Regex) {
        self.unicode_punctuation = re;
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
    ///     bot.load_directory("../eg/brain").expect("Couldn't load directory!");
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
    ///     bot.load_file("../eg/brain/eliza.rive").expect("Couldn't load file from disk!");
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
        let objects = ast.objects.clone();
        self.brain.extend(ast);

        // In case the parse changed the depth variable, update it.
        if let Ok(depth) = self.brain.get_global("depth").parse() {
            self.depth = depth;
        }

        // Load all the parsed object macros.
        for (name, object) in objects {
            if !self.macro_handlers.contains_key(&object.language) {
                debug!("Note: object macro '{}' is written in an unhandled language '{}'; skipping", name, object.language);
                continue;
            }

            debug!("Loading object macro {} ({})", name, object.language);
            let handler: &mut Box<dyn LanguageLoader> = self.macro_handlers.get_mut(&object.language).unwrap();
            match handler.load(&name, object.code) {
                Ok(_) => {
                    // Store the language handler for this macro's name.
                    self.object_langs.insert(name, object.language);
                },
                Err(e) => warn!("Error parsing object macro '{}': {}", name, e),
            };
        }

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
        // debug!("sorted_topics: {:#?}", self.sorted_topics);
        // debug!("sorted_thats: {:#?}", self.sorted_thats);
        // debug!("sorted_subs: {:#?}", self.sorted_subs);
        // debug!("sorted_person: {:#?}", self.sorted_person);
    }

    /// Get a reply from the chatbot.
    pub async fn reply(&mut self, username: &str, message: &str) -> Result<String, String> {
        // let msg = reply::Message{
        //     username: String::from("username"),
        // }
        reply::reply(self, username, message).await
    }

    /// Define an object macro handler from a Rust function.
    ///
    /// This is a named function that you can call from RiveScript using the `<call>` tag. The parameters
    /// to your function will be the RiveScript interpreter and the array of arguments (shell quote style)
    /// passed in to the call.
    ///
    /// Example: `<call>example "hello world"</call>`
    pub fn set_subroutine<F>(&mut self, name: &str, f: F)
    where
        F: for<'a> Fn(&'a mut Proxy<'a>, Vec<String>) -> BoxFuture<'a, Result<SubroutineResult, String>> + Send + Sync + 'static
    {
        self.subroutines.insert(name.to_string(), Box::new(f));
    }

    /// Set a handler for custom object macros written in other programming languages.
    pub fn set_handler(&mut self, language: &str, loader: impl LanguageLoader + 'static) {
        self.macro_handlers.insert(language.to_string(), Box::new(loader));
    }

    /// Get the current user's username.
    ///
    /// This is only valid from within a reply context, e.g. from a Rust object macro subroutine.
    pub fn current_username(&self) -> Result<String, String> {
        if !self.in_reply_context {
            Err("current_username is only valid during a reply context".to_string())
        } else {
            Ok(self.current_username.to_string())
        }
    }

    /// Set a user variable for a user.
    ///
    /// Equivalent to `<set name=value>` in RiveScript for the username.
    pub async fn set_uservar(&self, username: &str, name: &str, value: &str) {
        self.sessions.set(username, HashMap::from([
            (name.to_string(), value.to_string()),
        ])).await
    }

    /// Get a user variable from a user.
    ///
    /// Equivalent to `<get name>` in RiveScript.
    ///
    /// Returns the string "undefined" if not set.
    pub async fn get_uservar(&self, username: &str, name: &str) -> String {
        self.sessions.get(username, name).await
    }

    /// Set many user variables for a given user.
    ///
    /// With this function, you could restore a full set of user variables (e.g. which
    /// were previously retrieved from [get_uservars]) by providing a full HashMap of
    /// key/value pairs.
    pub async fn set_uservars(&self, username: &str, vars: HashMap<String, String>) {
        self.sessions.set(username, vars).await
    }

    /// Get all stored user variables for a given user.
    pub async fn get_uservars(&self, username: &str) -> HashMap<String, String> {
        self.sessions.get_any(username).await
    }

    /// Get all stored user variables about all users.
    ///
    /// This function may be most useful when using the default in-memory user variable storage.
    /// It returns a HashMap of usernames paired to the HashMap of all of their data.
    ///
    /// If you are using a third-party storage driver (such as to use Redis or SQL), you
    /// will probably not want to call this function in case it scrapes your entire table
    /// end-to-end and returns ALL data about ALL users.
    pub async fn get_all_uservars(&self) -> HashMap<String, HashMap<String, String>> {
        self.sessions.get_all().await
    }

    /// Debugging: print the loaded bot's brain (AST) to console.
    pub fn debug_print_brain(&self) {
        println!("{:#?}", self.brain);
    }

    /// Debugging: print the sorted trigger lists.
    pub fn debug_sorted_replies(&self) {
        println!("{:#?}", self.sorted_topics);
    }
}
