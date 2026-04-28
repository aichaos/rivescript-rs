# Examples

See below for some examples how to do various things with RiveScript in Rust.

## Example Brain

* [eg/brain](brain/) is the standard default RiveScript brain (`.rive` files) that implements an Eliza-like chatbot with added triggers to demonstrate other features of RiveScript.

    The Rust port includes `rust.rive` which demonstrates calling some object macro subroutines that are written in Rust. The default `rivescript` crate executable defines those object macros, so if you simply `cargo install rivescript` the binary is able to test those examples.

## Code Snippets

* [redis](../rivescript-redis/examples/) - Demonstrates using a Redis cache to proactively store user variables and history, instead of keeping them in the default in-memory HashMap store.