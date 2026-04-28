//! Foreign Programming Language Object Macros
//!
//! This module provides the LanguageLoader trait which enables you to define
//! a custom programming-language handler for RiveScript Object Macros written
//! in languages other than Rust.
//!
//! For example, a RiveScript document might define an object macro written
//! in JavaScript like so:
//!
//! ```rivescript
//! > object reverse javascript
//!     let str = args.join(" ");
//!     return str.split('').reverse().join('');
//! < object
//!
//! + reverse *
//! - "<star>" spelled backwards is "<call>reverse <star></call>."
//! ```

/// The trait for a custom programming language loader for RiveScript object macros.
///
/// The load() function will receive the name of the object macro along with its
/// source code (as lines of text). Your LanguageLoader might then parse and
/// evaluate the code using your backing runtime or VM.
///
/// The call() function is invoked when a named object macro has been called via
/// the `<call>` tag in a RiveScript reply. The `name` is the name of the object
/// macro and the `args` are the parameters (using shell-style quoting rules, so
/// a "quoted string" would come as one item of the `Vec<String>`).
pub trait LanguageLoader: Send + Sync {
    fn load(&mut self, name: &str, code: Vec<String>) -> Result<bool, String>;
    fn call(&self, name: &str, args: Vec<String>) -> Result<String, String>;
}