# RiveScript in Rust

This is a port of the RiveScript interpreter for the Rust programming language.

It is very much a **WORK IN PROGRESS** and is not functional yet. The checklist
below may give you an idea of its state. Watch the git log and see me learn Rust
while I figure this module out!

The rough roadmap as I see it so far:

- [ ] Read and parse RiveScript source documents into memory.
    - [x] load_directory(), load_file() and stream() can access RiveScript sources.
    - [ ] Parse document into complete 'abstract syntax tree' mapping out topics,
          triggers and replies.
    - [ ] Support all RiveScript **commands**:
        - [ ] `! DEFINITION`
        - [ ] `> LABEL`
        - [ ] `+ TRIGGER`
        - [ ] `- RESPONSE`
        - [ ] `% PREVIOUS`
        - [ ] `^ CONTINUE`
        - [ ] `@ REDIRECT`
        - [ ] `* CONDITION`
        - [ ] `// COMMENT` and `/* multiline comments */`
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

# License

This module will be released under MIT when it becomes functional.

Copyright © 2022 Noah Petherbridge.