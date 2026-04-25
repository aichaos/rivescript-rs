// Object macros and language handlers for RiveScript.

use async_trait::async_trait;
use futures::future::BoxFuture;

use crate::RiveScript;
use crate::macros::proxy::{Proxy, SubroutineResult};

pub mod proxy;

#[async_trait]
pub trait Rust {
    async fn call(name: &str, fields: Vec<String>) -> Result<String, String>;
}

pub type Subroutine = Box<
    dyn for<'a> Fn(&'a mut Proxy<'a>, Vec<String>) -> BoxFuture<'a, Result<SubroutineResult, String>>
    + Send
    + Sync
>;