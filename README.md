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
    - [x] `! local concat = none|space|newline`
    - [x] `! global depth = 25` can change recursion depth
    - [ ] Syntax checking and strict mode
- [x] Sorting the replies
    - [x] Sorting +Triggers
    - [x] Sorting %Previous
    - [x] Sorting substitution lists
    - [x] Topic inherits/includes.
- [ ] Fetch a reply for the user
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
    - [ ] Reply Tags:
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
        - [ ] `<call>` (object macros)
        - [x] `{ok}`
        - [x] `\s`
        - [x] `\n`
        - [x] `\/`
        - [x] `\#`
- [ ] Make it pass the [RiveScript Test Suite][rsts] to verify it is _at least_ as accurate as the other 5 implementations.
- [ ] Followup niceties:
    - [ ] A JavaScript interpreter for built-in support for JS object macros.
    - [ ] Pluggable user variable session drivers (with e.g. Redis implementation).

# Testing It

Git clone this project and run: `cargo run -- eg/brain`

For help: `cargo run -- --help`

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

# License

This module will be released under MIT when it becomes functional.

Copyright © 2022-2026 Noah Petherbridge.

[rsts]: https://github.com/aichaos/rsts