//! # RiveScript Core
//!
//! This crate provides common types and traits that are useful for RiveScript
//! and third-party plugin modules. For example, the [LanguageLoader] for
//! foreign programming-language Object Macro Handlers lives here so that the
//! RiveScript crate and external implementations can both reference it.

pub mod ast;
pub mod macros;
pub mod parser;
pub mod regex;
pub mod sessions;

pub const DEFAULT_TOPIC: &str = "random";
pub const BEGIN_TOPIC: &str = "__begin__";
pub const ERR_NO_MATCH: &str = "ERR: No Reply Matched";
pub const ERR_NO_REPLY: &str = "[ERR: No Reply]";
pub const BEGIN_REQUEST: &str = "request";
pub const TAG_OK: &str = "{ok}";
pub const UNDEFINED: &str = "undefined";
pub const MAX_STARS: usize = 9;
pub const MAX_HISTORY: usize = 9;
pub const DEFAULT_DEPTH: usize = 50;