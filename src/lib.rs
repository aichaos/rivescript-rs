//! Implementation of the RiveScript chatbot scripting language.
//!
//! RiveScript is a simple scripting language designed for implementing
//! chatbots that communicate with a user through plain language. This
//! module provides an official RiveScript engine for Rust written by the
//! language author.

use log::debug;
use parser::Parser;
use std::{error::Error, fs};
use Result::Ok;

mod ast;
mod errors;
mod parser;
mod tests;

/// RiveScript represents a single chatbot personality in memory.
pub struct RiveScript {
    pub debug: bool,
    pub utf8: bool,
    pub depth: i32,

    parser: Parser,
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
            parser: Parser::new(),
        }
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
    pub fn load_directory(&self, path: &str) -> Result<bool, Box<dyn Error>> {
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
    pub fn load_file(&self, path: &str) -> Result<bool, Box<dyn Error>> {
        debug!("load_file called on: {}", path);

        let contents = fs::read_to_string(path)?;

        let _ast = self.parser.parse(path, contents)?;
        Ok(true)
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
    pub fn stream(&self, source: String) -> Result<bool, Box<dyn Error>> {
        let _ast = self.parser.parse("stream()", source)?;
        Ok(true)
    }

    /// After loading RiveScript source documents, this function will
    /// pre-populate sort buffers in memory.
    pub fn sort_triggers(&self) {}
}
