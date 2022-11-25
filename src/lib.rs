//! Implementation of the RiveScript chatbot scripting language.
//!
//! RiveScript is a simple scripting language designed for implementing
//! chatbots that communicate with a user through plain language. This
//! module provides an official RiveScript engine for Rust written by the
//! language author.

use log::debug;
use parser::Parser;
use std::fs;
use Result::Ok;

mod ast;
mod parser;

/// RiveScript represents a single chatbot personality in memory.
pub struct RiveScript {
    pub debug: bool,
    pub utf8: bool,
    pub depth: i32,

    parser: Parser,
}

impl RiveScript {
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
    /// # fn main() {
    ///     let mut bot = RiveScript::new();
    ///     bot.load_directory("./eg/brain").expect("Couldn't load directory!");
    /// # }
    /// ```
    pub fn load_directory(&self, path: &str) -> Result<bool, std::io::Error> {
        debug!("load_directory called on: {}", path);

        let paths = match fs::read_dir(path) {
            Ok(res) => res,
            Err(err) => return Err(err),
        };

        for filename in paths {
            let filepath = match filename {
                Ok(res) => res.path(),
                Err(err) => return Err(err),
            };

            match filepath.extension() {
                Some(ext) => {
                    if ext.eq_ignore_ascii_case("rive") || ext.eq_ignore_ascii_case(".rs") {
                        match self.load_file(filepath.as_path().display().to_string().as_str()) {
                            Ok(_) => continue,
                            Err(err) => return Err(err),
                        }
                    }
                }
                None => continue,
            }
        }

        return Ok(true);
    }

    /// Load a RiveScript document by filename on disk.
    pub fn load_file(&self, path: &str) -> Result<bool, std::io::Error> {
        debug!("load_file called on: {}", path);

        let contents = match fs::read_to_string(path) {
            Ok(res) => res,
            Err(err) => return Err(err),
        };

        self.parser.parse(path, contents);
        Ok(true)
    }

    /// Stream a string containing RiveScript syntax into the bot, rather than read from the filesystem.
    pub fn stream(&self, source: String) -> Result<bool, std::io::Error> {
        self.parser.parse("stream()", source);
        Ok(true)
    }

    /// After loading RiveScript source documents, this function will
    /// pre-populate sort buffers in memory.
    pub fn sort_triggers(&self) {}

    pub fn test(&self) {
        println!(
            "RiveScript debug={} utf8={} depth={}",
            self.debug, self.utf8, self.depth
        );
    }
}
