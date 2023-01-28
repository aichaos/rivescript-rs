# RiveScript in Rust

This is a port of the RiveScript interpreter for the Rust programming language.

It is very much a **WORK IN PROGRESS** and is not functional yet. The checklist
below may give you an idea of its state. Watch the git log and see me learn Rust
while I figure this module out!

The rough roadmap as I see it so far:

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
    - [ ] `! local concat = none|space|newline`
    - [ ] `! global depth = 25` can change recursion depth
    - [ ] Syntax checking and strict mode
- [ ] Sorting the replies
    - [ ] Sorting +Triggers
    - [ ] Sorting %Previous
    - [ ] Sorting substitution lists
- [ ] Fetch a reply for the user
    - [ ] User variable storage
    - [ ] `> begin` blocks
    - [ ] -Reply, and (weighted) random responses.
    - [ ] @Redirect
    - [ ] %Previous
    - [ ] *Condition
    - [ ] Tags:
        - [ ] `<star>, <star1> - <starN>`
        - [ ] `<botstar>, <botstar1> - <botstarN>`
        - [ ] `<input1> - <input9>`
        - [ ] `<reply1> - <reply9>`
        - [ ] `<id>`
        - [x] `<noreply>`
        - [ ] `<bot>`
        - [ ] `<env>`
        - [ ] `<get>, <set>`
        - [ ] `<add>, <sub>, <mult>, <div>`
        - [ ] `{topic=...}`
        - [ ] `{weight=...}`
        - [ ] `{@...}, <@>`
        - [ ] `{!...}`
        - [ ] `{random}`
        - [ ] `{person}, <person>`
        - [ ] `{formal}, <formal>`
        - [ ] `{sentence}, <sentence>`
        - [ ] `{uppercase}, <uppercase>`
        - [ ] `{lowercase}, <lowercase>`
        - [ ] `{ok}`
        - [ ] `\s`
        - [ ] `\n`
        - [ ] `\/`
        - [ ] `\#`

# Testing It

Git clone this project and run: `RUST_LOG=debug cargo run`

The main.rs program is currently hardcoded to read from ./eg/brain.

# Building

Install [Rust](https://www.rust-lang.org/) and build and test this project
with commands like the following:

* `cargo build`

    Builds the rivescript(.exe) binary.

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

# License

This module will be released under MIT when it becomes functional.

Copyright © 2022 Noah Petherbridge.