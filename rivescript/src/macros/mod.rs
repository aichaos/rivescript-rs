//! Object Macros
//!
//! This module defines types and traits useful for writing a RiveScript object
//! macro subroutine in Rust.

use futures::future::BoxFuture;

use crate::macros::proxy::{Proxy, SubroutineResult};

pub mod proxy;

/// Subroutine is a function pattern for defining a RiveScript object macro in Rust.
///
/// Example:
///
/// bot.set_subroutine("rust-set", |proxy, args| {
///     async move {
///         proxy.finish("Hello rust!".to_string())
///     }.boxed()
/// });
pub type Subroutine = Box<
    dyn for<'a> Fn(&'a mut Proxy<'a>, Vec<String>) -> BoxFuture<'a, Result<SubroutineResult, String>>
    + Send
    + Sync
>;