# RiveScript in Rust

This is a port of the RiveScript interpreter for the Rust programming language.

RiveScript is a scripting language for authoring the classic "canned responses" type of chatbots, making it easy for bot authors to program triggers and responses to build a chatbot's personality. See [rivescript.com](https://www.rivescript.com) for details.

# Usage

This crate provides both a library and a stand-alone executable, the latter of which is an interactive command line shell for testing your RiveScript bot. Run the program with the path to a folder (or file) on disk that contains your RiveScript documents. Example:

```bash
$ rivescript ./eg/brain
```

See `rivescript --help` for options it accepts, including debug mode and UTF-8 mode.

When used as a library for writing your own chatbot in Rust, the synopsis is as follows:

```rust
use rivescript::RiveScript;

#[tokio::main]
async fn main() {

    // Create a RiveScript bot instance.
    let mut bot = RiveScript::new();

    // Enable UTF-8 mode to support non-English chatbots.
    // See "UTF-8 Support" in the README for details.
    bot.utf8 = true;

    // Load a directory of RiveScript documents (.rive files)
    bot.load_directory("./eg/brain").expect("Error loading files!");

    // Load additional replies from a single .rive file.
    bot.load_file("./replies.rive").expect("Error loading file!");

    // Load RiveScript source from a string value instead of files.
    bot.stream("
        + hello bot
        - Hello, human!
    ").expect("Error parsing the streamed code!");

    // After loading your RiveScript sources, be sure to sort the triggers!
    // This populates internal sort structures to match a user's message with
    // the most optimal triggers in your bot's brain.
    bot.sort_triggers();

    // Enter a main loop to chat with the bot in your terminal.
    loop {

        // Print the prompt.
        print!("You> ");
        io::stdout().flush().expect("oops");

        // Read user input.
        let mut message = String::new();
        io::stdin()
            .read_line(&mut message)
            .expect("Failed to read line");

        // Get the reply.
        match bot.reply("local-user", &message).await {
            Ok(reply) => println!("Bot> {reply}"),
            Err(e) => println!("Error> {e}"),
        };

    }
}
```

# Examples

See the [eg/ folder](https://github.com/aichaos/rivescript-rs/tree/main/eg) on GitHub for some examples how to do various things with RiveScript.

A default example RiveScript brain (`.rive` files) can be found there, as well as examples how to use a Redis cache to proactively store user variables.

# Crates and Features

The rivescript-rs project publishes several useful crates:

* **[rivescript](rivescript/)** is the primary crate. It implements both the library and the stand-alone command-like program.
* **[rivescript-js](rivescript-js/)** enables RiveScript [Object Macros](#rust-object-macros) to be written in JavaScript.
* **[rivescript-redis](rivescript-redis/)** can store your [User Variables](#user-variable-session-adapters) in a Redis cache rather than in-memory HashMaps.
* **[rivescript-core](rivescript-core/)** contains common base types, traits, constants and so on for RiveScript. The AST (Abstract Syntax Tree) and Parser modules live there as well.

## JavaScript Feature

The primary **rivescript** crate has a `javascript` feature that applies to the command-line program.

To install `rivescript` with JavaScript support enabled:

```bash
cargo install rivescript --features javascript
```

To run it from the git project:

```bash
cargo run --features javascript -- ./eg/brain
```

The rivescript program should print a notice that JavaScript object macros are enabled just before the prompt:

```
      .   .
     .:...::      RiveScript Interpreter (Rust)
    .::   ::.     Library Version: v0.3.0 (build n/a)
 ..:;;. ' .;;:..
    .  '''  .     Type '/quit' to quit.
     :;,:,;:      Type '/help' for more options.
     :     :
Using the RiveScript bot found in: ["eg/brain"]
Type a message to the bot and press Return to send it.
Note: JavaScript object macros enabled.
You>
```

# Stability

**Current Status: Beta**

This port of RiveScript is "feature complete" and functional, implementing all of the commands and tags of RiveScript, and it passes the [RiveScript Test Suite (RSTS)][rsts].

The "stable 1.0.0" version of rivescript-rs will be released when:

* [x] 1. The [RiveScript Test Suite (RSTS)][rsts] has been implemented to verify that the Rust port is _at least_ as accurate as the other 5 official RiveScript ports are.
* [x] 2. A [JavaScript engine](rivescript-js/) for RiveScript Object Macros has been implemented, to verify that the interface for foreign language macro handlers is correctly done.
* [x] 3. A Redis driver for [User Variable Session Management](#user-variable-session-adapters) is implemented, to verify that the trait for that works as intended.
* [ ] 4. I am happy with the documentation, [its API](#rivescript-features-supported), etc.

If breaking changes need to happen before then, the second version number will increment (e.g. 0.3.x -> 0.4.0).

# Configuration

After calling `RiveScript::new()` you may configure the object to customize its behavior by setting the following attributes:

* `debug: bool` to enable debug mode. This will use log::debug and log::warn to print details about RiveScript's inner execution to your console. Note: the debug output is _very_ verbose!
* `utf8: bool` can enable [UTF-8 mode](#utf-8-support).
* `depth: usize` will set the recursion depth limit (default 50). This limit protects your bot from infinite recursion errors, in case two triggers redirect to each other.
* `case_sensitive: bool` can make user messages case sensitive. The default is false, and user messages are made lowercase before matching against your triggers. If you set a true value, their message will not be made lowercase.

The `rivescript` command-line program can set some of these options with flags like `--debug` and `--utf8`. See `rivescript --help` for full details.

The recursion depth limit can also be overridden in your RiveScript brain using the `! global` command like so:

```rivescript
! global depth = 256
```

# Async API

The main `rivescript.reply()` function is an async function, so you will need to use an async runtime such as `tokio` to use this library. The example above uses an `async fn main()` using tokio.

Historically, most of the other implementations of RiveScript (written in Perl, Python, Java, and Go) were written in a synchronous (procedural) manner, where the reply() function was not async. This was OK for those languages because those languages were not generally async aware overall: common libraries for things like SQL databases and HTTP requests all had blocking (synchronous) API calls; so for example, an [Object Macro](#rust-object-macros) was able to interact with these APIs and get its answer synchronously and the main reply() function could be synchronous to match, and similarly, [User Variable Session Adapters](#user-variable-session-adapters) were able to get/set variables in a Redis cache or SQL database using the synchronous APIs common to those languages.

This model led to some friction with its JavaScript port, because JavaScript is a heavily async language and all of the useful libraries (for web requests, SQL, etc.) were asynchronous, and RiveScript wasn't able to stop and await for these during the reply() phase. Eventually, when Async/Await support dropped in JavaScript, RiveScript.js was able to await these calls while still keeping its overall logic in line with the other ports.

For the Rust port, async/await was built in from the beginning in case you want to call async crates from within a RiveScript reply.

# UTF-8 Support

RiveScript, historically, was not designed with UTF-8 in mind from the beginning. All ports of RiveScript provide a "UTF-8 mode," however, which is labeled as an 'experimental' feature of RiveScript (because its use may affect trigger matching behavior in subtle ways).

By default (without UTF-8 mode enabled), RiveScript triggers are only allowed to contain basic ASCII characters (no foreign characters), and the user's input message will be stripped of all characters except for letters, numbers and spaces. Note: this stripping happens after substitutions are run, so you can `! sub what's = what is` to normalize and process their message first (and substitutions for those kind of contractions is recommended practice).

When UTF-8 mode is enabled, these restrictions are lifted:

* Triggers in RiveScript sources will only be limited to not contain certain metacharacters such as backslashes.
* The user's message is only stripped of backslashes and HTML angled brackets (to protect from obvious XSS attacks if you use RiveScript in a web application).

    Additionally, common punctuation characters will be stripped from the user's message, with the default set being `/[.,!?;:]/` which can be overridden by providing a new regexp of your own (RiveScript.set_unicode_punctuation()).

The `<star>` tags in RiveScript would therefore be able to match the user's "raw" input strings (with non-ASCII characters preserved).

# Rust Object Macros

RiveScript has a feature called "object macros" that enable you to write custom program code to provide a dynamic response in your chatbot. For example, your bot can have a trigger for "what is the weather like in Los Angeles?" which could run custom code to fetch the answer from a weather API or similar.

All RiveScript interpreters support object macros written in their native programming language, and the Rust port is no exception!

Here is an example how to define a custom object macro subroutine in Rust:

```rust
#[tokio::main]
async fn main() {
    let mut bot = RiveScript::new();

    // Define an object macro named "hello-rust"
    bot.set_subroutine("hello-rust", |proxy, args| {
        async move {
            if args.len() >= 1 {
                let value = args.join(" ");
                return proxy.finish(format!("Hello, {value}!"));
            }
            proxy.finish("Hello, rust!".to_string())
        }.boxed()
    });

    // Example RiveScript document to call this macro.
    bot.stream("
        + hello rust
        - <call>hello-rust</call>

        + hello *
        - <call>hello-rust <star></call>
    ").expect("Failed to parse");

    bot.sort_triggers();

    assert_eq!(bot.reply("username", "hello rust").await, "Hello, rust!");
}
```

## RiveScript Proxy for Object Macro Subroutines

If you are familiar with the other RiveScript ports, the Rust version has some unique nuances due to the borrow checker: usually, object macro subroutines would receive a pointer to the master RiveScript struct and a string array of parameters, but in Rust it wouldn't be possible to send a mutual borrow of RiveScript with the subroutine.

Instead, a rivescript::macros::Proxy is passed in. The Proxy exposes a subset of useful RiveScript functions (such as get_uservar and set_uservar) which are most commonly useful for subroutines. This allows object macros to get and set user and bot variables. When getting variables, the master RiveScript struct can provide their values. When setting variables, the Proxy holds a local HashMap of 'staged' data which is committed after your subroutine returns. If you set and then get a variable within your subroutine, you will get back the 'staged' copy from the Proxy.

Here is an example subroutine that gets and sets a user variable:

```rust
bot.set_subroutine("rust-set", |proxy, args| {
    async move {
        if args.len() >= 2 {
            let username = proxy.current_username().unwrap_or(String::new());

            let name = args.get(0).unwrap();
            let value = args.get(1).unwrap();
            let orig_value = proxy.get_uservar(&name).await;

            proxy.set_uservar(name, value).await;
            let staged_value = proxy.get_uservar(&name).await;

            return proxy.finish(format!("For username {username}: The original variable '{name}' was '{orig_value}' and I have updated it to '{value}' (staged value: '{staged_value}')"));
        }
        proxy.finish("Usage: rust-set name value".to_string())
    }.boxed()
});
```

And its usage from RiveScript:

```rivescript
+ rust set * *
- <call>rust-set <star1> "<star2>"</call>
```

# User Variable Session Adapters

By default, RiveScript stores user variables in memory using a HashMap keyed by the username passed in to the reply() function. You can import and export user variables with functions like get_uservars() and set_uservars().

Like most of the other RiveScript implementations, this crate also provides support for pluggable User Variable Session Adapters so you may persist user variables proactively into something like a Redis cache or SQL database.

See **[rivescript-redis](rivescript-redis/)** for an example implementation that uses a Redis cache.

# Development

Git clone this project recursively so that you pull in the [RiveScript Test Suite][rsts] submodule:

```bash
git clone --recursive git@github.com:aichaos/rivescript-rs
```

To run the `rivescript` command line program for testing:

```bash
cd rivescript/
cargo run -- ./eg/brain
```

# Building

Install [Rust](https://www.rust-lang.org/) and build and test this project
with commands like the following:

* `cargo build`

    Builds the rivescript(.exe) binary.

# Testing

Among the unit tests, the [RiveScript Test Suite (RSTS)][rsts] is used to do the bulk of the RiveScript language testing. RSTS is included as a git submodule, so be sure to either clone the rivescript-rs repo recursively or run `git submodule init && git submodule update` to download it after the fact.

The `rivescript/build.rs` file will check whether the RSTS is available at build time, and log a note if it is not. Note: this only happens when you are building from the local git repo, it shouldn't log when you are simply using rivescript-rs from your own separate project.

To debug the RSTS test results, run the tests like so:

```bash
cargo test rivescript -- --nocapture
```

The happy path should show all RSTS tests passing on your terminal:

```
Loading test: triggers.yml
✓ triggers.yml#trigger_arrays
✓ triggers.yml#weighted_triggers
✓ triggers.yml#atomic
✓ triggers.yml#alternatives_and_optionals
✓ triggers.yml#wildcards

Loading test: unicode.yml
✓ unicode.yml#wildcards
✓ unicode.yml#unicode

Loading test: options.yml
✓ options.yml#test_concat_space_with_conditionals
✓ options.yml#test_concat_none_with_conditionals
✓ options.yml#concat
✓ options.yml#test_concat_newline_with_conditionals

Loading test: math.yml
✓ math.yml#addition

Loading test: test-spec.yml
✓ test-spec.yml#test_name

Loading test: begin.yml
✓ begin.yml#simple_begin_block
✓ begin.yml#blocked_begin_block
✓ begin.yml#no_begin_block
✓ begin.yml#conditional_begin_block

Loading test: substitutions.yml
✓ substitutions.yml#person_substitutions
✓ substitutions.yml#message_substitutions

Loading test: bot-variables.yml
✓ bot-variables.yml#global_variables
✓ bot-variables.yml#bot_variables

Loading test: replies.yml
✓ replies.yml#continuations
✓ replies.yml#previous
✓ replies.yml#redirects
✓ replies.yml#embedded_tags
✓ replies.yml#random
✓ replies.yml#set_uservars
✓ replies.yml#conditions
✓ replies.yml#questionmark
✓ replies.yml#redirect_with_undefined_input
✓ replies.yml#reply_arrays
✓ replies.yml#redirect_with_undefined_vars

test test_rivescript_suite ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s
```

# RiveScript Features Supported

This port of RiveScript is "feature complete" and implements all of the commands and tags of RiveScript. The checklist below was used during the development of this module which lays out all of the tasks that a RiveScript interpreter must fulfill.

- [ ] Read and parse RiveScript source documents into memory.
    - [x] load_directory(), load_file() and stream() can access RiveScript sources.
    - [x] Parse document into complete 'abstract syntax tree' mapping out topics,
          triggers and replies.
    - [x] Support all RiveScript **commands**:
        - [x] `! DEFINITION`
        - [x] `> LABEL`
        - [x] `+ TRIGGER`
        - [x] `- RESPONSE`
        - [x] `% PREVIOUS`
        - [x] `^ CONTINUE`
        - [x] `@ REDIRECT`
        - [x] `* CONDITION`
        - [x] `// COMMENT` and `/* multiline comments */`
        - [x] Object macros (collecting names, languages, source code)
    - [x] `! local concat = none|space|newline`
    - [x] `! global depth = 25` can change recursion depth
    - [ ] Syntax checking and strict mode
- [x] Sorting the replies
    - [x] Sorting +Triggers
    - [x] Sorting %Previous
    - [x] Sorting substitution lists
    - [x] Topic inherits/includes.
- [x] Fetch a reply for the user
    - [x] User variable storage
    - [x] Substitutions (`! sub`)
    - [x] `> begin` blocks
    - [x] -Reply, and (weighted) random responses.
    - [x] @Redirect
    - [x] %Previous
    - [x] *Condition
    - [x] Trigger Tags:
        - [x] `[optionals]`
        - [x] `@arrays`
        - [x] `<bot>` and `<get>` user vars
        - [x] `<input>` and `<reply>` tags
    - [x] Reply Tags:
        - [x] `<star>, <star1> - <starN>`
        - [x] `<botstar>, <botstar1> - <botstarN>` (%Previous)
        - [x] `<input1> - <input9>` (user vars)
        - [x] `<reply1> - <reply9>` (user vars)
        - [x] `<id>`
        - [x] `<noreply>`
        - [x] `<bot>`, `<bot name=value>`
        - [x] `<env>`, `<env name=value>`
        - [x] `<get>, <set>` (user vars)
        - [x] `<add>, <sub>, <mult>, <div>` (user vars)
        - [x] `{topic=...}` (partially; needs user var storage)
        - [x] `{weight=...}`
        - [x] `{@...}, <@>`
        - [ ] `{!...}` (~~DEPRECATED~~)
        - [x] `{random}` and `@(arrays)`
        - [x] `{person}, <person>`
        - [x] `{formal}, <formal>`
        - [x] `{sentence}, <sentence>`
        - [x] `{uppercase}, <uppercase>`
        - [x] `{lowercase}, <lowercase>`
        - [x] `<call>` (object macros)
        - [x] `{ok}`
        - [x] `\s`
        - [x] `\n`
        - [x] `\/`
        - [x] `\#`
- [x] Make it pass the [RiveScript Test Suite][rsts] to verify it is _at least_ as accurate as the other 5 implementations.
- [x] Followup niceties:
    - [x] A JavaScript interpreter for built-in support for JS object macros.
    - [x] Pluggable user variable session drivers (with e.g. Redis implementation).

# Developer Notes

This may be put somewhere else when the module is closer to "done."

Just some notes about integrating this module as compared to the
other programming languages RiveScript was written in:

* For Rust borrowing/ownership, when the parser finds a +Trigger it
  can not "give" it to the AST immediately like it does in most other
  implementations; because -Reply or *Condition need to write into the
  Trigger reference which it can't do if the AST has it. So the buffer
  for the current Trigger is given to the AST when:
    * Another +Trigger command is found which starts a new trigger;
      the current trigger is given to AST before starting the new one.
    * When a `> begin` or `> topic` is started; any trigger-in-progress
      for the old topic is committed to AST.
    * At the end of the parse phase: if one final trigger was being
      populated it is given to AST before returning.
* In the parser: most implementations do a look-ahead scan both to
  collect `^Continues` (append them to the current line) and to peek
  for `%Previous` underneath triggers. In rivescript-rs we only look
  ahead for `^Continue` and process `%Previous` in the normal command
  switch similar to `@Redirect` or `*Condition`
* A long-standing bug with topic inheritance/includes was uncovered!

    In the eg/brain/rpg.rive `rpg demo` that demonstrates the feature, the
    game would get stuck in topic `puzzle1` because of a conflict with the
    included topic `puzzle` having a duplicate trigger for "west" which
    caused the user to always be taken back to the beginning of the puzzle.

    A very long time ago, RiveScript implementations kept the sorted list
    of triggers in-memory as being a simple list of strings (`Vec<String>`),
    and when the user matched a trigger, the reply details for it were looked
    up from a HashMap. However, that HashMap approach made it impossible to
    have duplicate triggers (as you might want to have when using %Previous,
    e.g. the bot could ask multiple yes/no questions and you could program a
    trigger for `yes` having a %Previous pointing to the bot's question, but
    multiple `yes` triggers would trample over that).

    Somewhere between 2012-2014, in the "v1.0" era of the JavaScript and
    Python ports to RiveScript, the sorted trigger set was changed to hold the
    full response data too, but this introduced a bug in the way that the
    topic inherits/includes feature worked.

    With "included" topics, the sets of triggers for all topics are treated
    as equals and sorted amongst themselves, with only "inherited" topics
    having their own priority. Anyway, since `puzzle1` had a "duplicate"
    trigger "west" shared by included topic `puzzle`, and with the ordering,
    both triggers were added to the sort list but not in the correct order
    (letting puzzle2's version match first).

    This is fixed in the Rust port, by having `inherits::get_topic_triggers`
    prioritize adding the local topic's triggers _first_, while de-duplicating
    copies of those triggers from included topics (allowing the local topic's
    trigger to "shadow" the included one's), before finally mixing in the
    inherited topics. The result is that when sort_replies() is finally doing
    the final sort (by {weight} and inheritance level, etc.), only one copy of
    the `west` trigger exists (from `puzzle1` which over-shadowed `puzzle`)
    resulting in the Rust port of RiveScript being the only one in the last
    14 years that can play the RPG demo correctly.

    Apparently also, the [RiveScript Test Suite][rsts] doesn't exercise this
    feature of RiveScript at all so the bug went unnoticed for many years!

    The JavaScript, Python and Go ports of RiveScript all shared the bug, with
    only the original Perl version (with its legacy implementation) working
    correctly.
* Object macros (Rust subroutines) posed an interesting dilemma!

    In most RiveScript implementations, Subroutines can be defined in the
    native programming language and they tended to accept a reference to the
    master RiveScript struct as their first parameter (so they could get/set
    user variables or manipulate the bot's inner state).

    (Subroutines with a function signature like `(*RiveScript, []args)` can)
    be invoked from a RiveScript reply with the `<call>name args...</call>`
    syntax and have the subroutine's result substituted in its place).

    For the Rust borrow checker, it wasn't possible to share a mutable
    RiveScript with the Subroutine, so instead a Proxy object is sent in.
    The Proxy has a subset of RiveScript functions that the subroutine might
    want (to get/set variables, etc.), and when Reading a variable it will
    come directly from RiveScript or its user variable session store. The
    Proxy also stages writes to variables using its own HashMap, so if you
    set_uservar and then get_uservar you will get the staged copy while within
    your Subroutine, and then RiveScript will commit the staged changes after
    your Subroutine returns.

    See src/main.rs and eg/brain/rust.rive for examples and details.

# License

```
The MIT License (MIT)

Copyright (c) 2022-2026 Noah Petherbridge

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

[rsts]: https://github.com/aichaos/rsts