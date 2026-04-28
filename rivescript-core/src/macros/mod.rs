//! Traits and types for RiveScript Object Macros.

use std::collections::HashMap;

use async_trait::async_trait;

/// Proxy is given as the first argument to RiveScript object macro functions.
///
/// It stands in as a 'proxy' for the master RiveScript struct. In most RiveScript
/// implementations (in other programming languages), object macros receive a
/// pointer to RiveScript as their first parameter so that they can get/set user
/// variables or manipulate the inner state of the bot to varying degrees.
///
/// In Rust, it is not possible to give a mutable borrow of RiveScript to the
/// object macros. Instead, the Proxy stands in for RiveScript and exposes a
/// subset of useful function calls.
///
/// Functions like get_uservar should proxy through and get the live value from
/// the RiveScript struct. Functions like set_uservar should cache or 'stage'
/// the updates (e.g. using an internal HashMap), and return those values if
/// a subsequent get function asks for them.
///
/// Object macros return their final response via the Proxy.finish() method,
/// which (in the SubroutineResult) carries the staged writes to user variables
/// back out so the master RiveScript struct can commit them all after the
/// subroutine has returned.
///
/// For object macro subroutines written in Rust, the concrete implementation
/// of this trait can be found in [rivescript::macros::proxy].
#[async_trait]
pub trait Proxy: Send + Sync {
    fn current_username(&mut self) -> Result<String, String>;
    async fn set_uservar(&mut self, name: &str, value: &str);
    async fn get_uservar(&self, name: &str) -> String;
    fn set_variable(&mut self, name: &str, value: &str);
    fn get_variable(&self, name: &str) -> String;
    fn finish(&mut self, output: String) -> Result<SubroutineResult, String>;
}

/// SubroutineResult is the return value from object macro subroutines,
/// especially those written natively in Rust.
///
/// Its job is to carry the text output of the object macro, along with any
/// 'staged' user or bot variables that the macro wanted to update.
pub struct SubroutineResult {
    pub output: String,
    pub staged_user_vars: HashMap<String, String>,
    pub staged_bot_vars: HashMap<String, String>,
}

/// # Foreign Programming Language Object Macros
///
/// The LanguageLoader trait enables you to define a custom programming-language
/// handler for RiveScript Object Macros written in languages other than Rust.
///
/// For example, a RiveScript document might define an object macro written
/// in JavaScript like so:
///
/// ```rivescript
/// > object reverse javascript
///     let str = args.join(" ");
///     return str.split('').reverse().join('');
/// < object
///
/// + reverse *
/// - "<star>" spelled backwards is "<call>reverse <star></call>."
/// ```
///
/// The load() function will receive the name of the object macro along with its
/// source code (as lines of text). Your LanguageLoader might then parse and
/// evaluate the code using your backing runtime or VM.
///
/// The call() function is invoked when a named object macro has been called via
/// the `<call>` tag in a RiveScript reply. The `name` is the name of the object
/// macro and the `args` are the parameters (using shell-style quoting rules, so
/// a "quoted string" would come as one item of the `Vec<String>`).
#[async_trait]
pub trait LanguageLoader: Send + Sync {
    fn load(&mut self, name: &str, code: Vec<String>) -> Result<bool, String>;
    async fn call(&self, proxy: &dyn Proxy, name: &str, args: Vec<String>) -> Result<SubroutineResult, String>;
}