# RiveScript Core

This crate provides common types and traits that are useful for RiveScript and third-party plugin modules. For example, the LanguageLoader for foreign programming-language Object Macro Handlers lives here so that the RiveScript crate and external implementations can both reference it.

## AST and Parser

The RiveScript Parser lives here and it converts RiveScript source documents into their Abstract Syntax Tree (AST). The AST carries all of the useful data for a RiveScript personality, including its configuration (bot variables, globals, substitutions, arrays), topics and replies, and inline object macro source codes.

## Traits

Useful traits in this crate include:

* rivescript_core::sessions::SessionManager for implementing a storage driver for user variables.

    By default, RiveScript stores user variables in memory using HashMaps and it provides API functions like get_uservars() and set_uservars() to export and import them in bulk. Instead, a custom SessionManager can persist user variables proactively into something like a database or cache.

    Example: [rivescript-redis](https://github.com/aichaos/rivescript-rs/tree/main/rivescript-redis) can use a Redis cache to store your user variables.

* rivescript_core::macros::LanguageLoader for implementing custom Object Macro handlers written in foreign programming languages.

    Note: this trait is a work in progress until a JavaScript example is implemented, to confirm it works correctly.
