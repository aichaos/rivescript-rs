use std::collections::HashMap;

use crate::ast::{Object, Trigger, AST};
use crate::errors::ParseError;
use log::{debug, warn};
use Result::Ok;

/// The version of the RiveScript spec we support.
const RIVESCRIPT_SPEC_VERSION: f32 = 2.0;

/// The default topic name.
const DEFAULT_TOPIC: &str = "random";

pub struct Parser {}

// enum ConcatMode {
//     None,
//     Newline,
//     Space,
// }

// impl ConcatMode {
//     fn string(self) -> &'static str {
//         match self {
//             ConcatMode::None => "",
//             ConcatMode::Newline => "\n",
//             ConcatMode::Space => " ",
//         }
//     }
// }

impl Parser {
    pub fn new() -> Self {
        Self {}
    }

    pub fn parse(&self, filename: &str, contents: String) -> Result<AST, ParseError> {
        debug!("BEGIN PARSE ON FILENAME: {}", filename);

        // Start building an AST parsed from these files.
        let mut ast = AST::new();

        // Local (file-scoped) parser options.
        let mut local_options: HashMap<String, String> = HashMap::new();
        local_options.insert("concat".to_string(), "none".to_string());

        // Some temporary state variables as we parse this file.
        let mut topic = String::from("random");
        let mut current_trigger = Trigger::new("");
        let mut lineno: usize = 0;
        let mut in_comment = false;
        let mut in_object = false;
        let mut object_name = String::from("");
        let mut object_language = String::from("");
        let mut object_buffer: Vec<String> = Vec::new();

        // Initialize the "random" topic.
        ast.init_topic(&topic);

        // Go through the lines of code.
        // let mut lines = contents.lines();
        let lines: Vec<String> = contents.lines().map(|s| s.to_string()).collect();
        loop {
            if lineno >= lines.len() {
                break;
            }

            let mut line = lines[lineno].to_string();
            lineno += 1;

            // Strip the line (skip empty lines).
            line = line.trim().to_string();
            if line.len() == 0 {
                continue;
            }

            // Are we inside of a `> object` macro?
            if in_object {
                // Have we reached the end?
                if line.contains("< object") || line.contains("<object") {
                    if object_name.len() > 0 {
                        let new_object =
                            Object::new(&object_name, &object_language, object_buffer.to_owned());
                        ast.objects.insert(object_name.to_string(), new_object);
                        in_object = false;
                    }
                } else {
                    object_buffer.push(line);
                }
                continue;
            }

            // Handle and ignore comments.
            if line.starts_with("//") {
                continue; // single-line comment.
            } else if line.starts_with("/*") {
                // Start of a multi-line comment block.
                if line.contains("*/") {
                    continue; // The end is on the same line!
                }

                // Now inside a comment block.
                in_comment = true;
                continue;
            } else if line.contains("*/") {
                // End of a multi-line comment block.
                in_comment = false;
                continue;
            } else if in_comment {
                continue;
            }

            debug!("Line #{}: {}", lineno, line);

            // Separate the command from its data.
            if line.len() < 2 {
                warn!(
                    "Weird single-character line '{}' found at {} line {}",
                    line, filename, lineno,
                );
                continue;
            }
            let cmd = &line[..1];
            let mut line = line[1..].trim().to_string();

            // Ignore inline comments at the end of the line.
            if line.contains(" // ") {
                let mut splitter = line.splitn(2, " // ");
                line = splitter.next().unwrap_or("").to_string();
            }

            // Do a look-ahead for ^Continue and %Previous commands.
            if cmd != "^" {
                let mut li = lineno;
                loop {
                    if li >= lines.len() {
                        break;
                    }

                    let lookahead = lines[li].trim();
                    li += 1;
                    if lookahead.len() < 2 {
                        continue;
                    }

                    let look_cmd = &lookahead[..1];
                    let lookahead = lookahead[1..].trim();

                    // We only care about a couple of lookahead command types.
                    if look_cmd != "^" || lookahead.len() == 0 {
                        break;
                    }

                    // If our parent command is a ! and the next command(s) are ^,
                    // we'll tack each extension on as a "fake line break" (which
                    // is useful information for !arrays especially)
                    if cmd == "!" {
                        if look_cmd == "^" {
                            line.push_str("<crlf>");
                            line.push_str(lookahead);
                        }
                    }
                }
            }

            // Handle the types of RiveScript commands.
            match cmd {
                // !Definition
                "!" => {
                    warn!("Found a !DEFINITION");

                    // The command looks like:
                    // ! version = 2.0
                    // ! global depth = 50
                    // ! var name = Chatbot
                    // ! sub who's = who is
                    let mut halves = line.splitn(2, "=");
                    let left = halves.next().unwrap_or("").trim();
                    let right = halves.next().unwrap_or("").trim();
                    let mut value = String::from("");
                    let mut kind = ""; // global, var, sub, ...
                    let mut name = "";

                    if right.len() > 0 {
                        // The right half of the = sign is always the value.
                        value.push_str(right);
                    }
                    if left.len() >= 1 {
                        // The left half has the kind and maybe the name.
                        // If `! version` there is only the kind=version,
                        // everything else has a name.
                        if left.contains(" ") {
                            let mut halves = left.splitn(2, " ");
                            kind = halves.next().unwrap_or("").trim();
                            name = halves.next().unwrap_or("").trim();
                        } else {
                            kind = left;
                        }
                    }

                    // Remove 'fake' line breaks unless this is an array.
                    if kind != "array" {
                        value = value.replace("<crlf>", "");
                    }

                    // Handle RiveScript specification version checks.
                    if kind == "version" {
                        warn!("Found a version str: {}", value);
                        let version = value.parse::<f32>().unwrap_or(0.0);
                        if version == 0.0 {
                            return Err(ParseError::new(
                                "Didn't parse version string; was it a properly formatted number?",
                            ));
                        } else if version > RIVESCRIPT_SPEC_VERSION {
                            return Err(ParseError::new("This RiveScript document declares a `! version` number higher than we support"));
                        } else {
                            ast.version = version;
                        }
                        continue;
                    }

                    // All other types of defines require a value and a name.
                    if name.len() == 0 {
                        warn!("Undefined variable name at {} line {}", filename, lineno);
                        continue;
                    } else if value.len() == 0 {
                        warn!("Undefined variable value at {} line {}", filename, lineno);
                        continue;
                    }

                    // Handle the rest of the !Define types.
                    match kind {
                        "local" => {
                            debug!("\tSet local parser option {} = {}", name, value);
                            local_options.insert(name.to_string(), value.to_string());
                        }
                        "global" => {
                            debug!("\tSet global {} = {}", name, value);
                            ast.globals.insert(name.to_string(), value.to_string());
                        }
                        "var" => {
                            debug!("\tSet bot variable {} = {}", name, value);
                            ast.vars.insert(name.to_string(), value.to_string());
                        }
                        "sub" => {
                            debug!("\tSet substitution {} => {}", name, value);
                            ast.subs.insert(name.to_string(), value.to_string());
                        }
                        "person" => {
                            debug!("\tSet person substitution {} => {}", name, value);
                            ast.person.insert(name.to_string(), value.to_string());
                        }
                        "array" => {
                            debug!("\tSet array {} = {}", name, value);

                            // Did we have multiple parts to this array? (^Continues)
                            let parts = value.split("<crlf>");

                            // Process each row of array data independently.
                            let mut fields: Vec<String> = Vec::new();
                            for val in parts {
                                if val.contains("|") {
                                    // Pipe-separated array (so the words can have spaces)
                                    let mut other: Vec<String> =
                                        val.split("|").map(str::to_string).collect();
                                    fields.append(&mut other);
                                } else {
                                    let mut other: Vec<String> =
                                        val.split_whitespace().map(str::to_string).collect();
                                    fields.append(&mut other);
                                }
                            }

                            // Convert any remaining '\s' escape sequences to spaces.
                            for field in fields.iter_mut() {
                                *field = field.replace("\\s", " ");
                            }

                            ast.arrays.insert(name.to_string(), fields);
                        }
                        &_ => {
                            warn!(
                                "Unknown definition type '{}' at {} line {}",
                                kind, filename, lineno,
                            );
                        }
                    }
                }

                // > Label
                ">" => {
                    warn!("Found a >LABEL");

                    // The command looks like:
                    // > begin
                    // > topic random
                    // > object something perl
                    let mut fields: Vec<String> =
                        line.split_whitespace().map(str::to_string).collect();
                    if fields.len() == 0 {
                        continue;
                    }

                    // First field is always the kind (begin, topic, object)
                    let mut kind = fields.remove(0);

                    // Next field may be the name (of topic or object)
                    let mut name = String::from("");
                    if fields.len() > 0 {
                        name = fields.remove(0);
                    }

                    // BEGIN is a type of topic.
                    if kind == "begin" {
                        kind = String::from("topic");
                        name = String::from("__begin__");
                    }

                    // Handle the kinds of labels.
                    match kind.as_str() {
                        "topic" => {
                            ast.init_topic(&name);

                            // Set the pointer for triggers to enter this topic.
                            topic = name.to_string();

                            // If we parsed a last trigger, commit and flush it
                            // ahead of the topic change.
                            if current_trigger.is_populated() {
                                let t = ast.topics.get_mut(&topic).expect("or else");
                                t.add_trigger(current_trigger);
                            }
                            current_trigger = Trigger::new("");

                            // Does this topic inherit or include another?
                            let mut mode = String::from("");
                            if fields.len() > 0 {
                                for field in fields {
                                    if field == "includes" || field == "inherits" {
                                        mode = field.to_string();
                                    } else if mode == "includes" {
                                        let t = ast.topics.get_mut(&topic).expect("or else");
                                        t.set_includes(field.to_string());
                                    } else if mode == "inherits" {
                                        let t = ast.topics.get_mut(&topic).expect("or else");
                                        t.set_inherits(field.to_string());
                                    }
                                }
                            }
                        }
                        "object" => {
                            // Start of an object macro definition.
                            let mut language = String::from("");
                            if fields.len() > 0 {
                                language = fields.remove(0).to_lowercase();
                            }

                            // No language defined?
                            if language.len() == 0 {
                                warn!(
                                    "No programming language defined for object '{}' at {} line {}",
                                    name, filename, lineno,
                                );
                                in_object = true;
                                object_name = name;
                                object_language = language;
                                continue;
                            }

                            // Start reading the object code.
                            object_name = name;
                            object_language = language;
                            object_buffer.truncate(0);
                            in_object = true;
                        }
                        &_ => {
                            warn!(
                                "Unsupported >LABEL kind '{}' found at {} line {}",
                                kind, filename, lineno,
                            );
                        }
                    }
                }

                // < Label
                "<" => {
                    let kind = line;

                    if kind == "begin" || kind == "topic" {
                        topic = DEFAULT_TOPIC.to_string();
                    }
                }

                // + Trigger
                "+" => {
                    // Were we working on a previous trigger? If so, give it
                    // over to the AST and start a new one. We can't give it
                    // over NOW because we will need to own/modify it to
                    // add replies/conditions/etc.
                    if current_trigger.is_populated() {
                        let t = ast.topics.get_mut(&topic).expect("or else");
                        t.add_trigger(current_trigger);
                    }

                    current_trigger = Trigger::new(line.as_str());
                }

                // % Previous
                "%" => {
                    current_trigger.previous = line.to_string();
                }

                // - Response
                "-" => {
                    current_trigger.reply.push(line.to_string());
                }

                // * Condition
                "*" => {
                    current_trigger.condition.push(line.to_string());
                }

                // @ Redirect
                "@" => {
                    current_trigger.redirect = line.to_string();
                }

                // ^ Continue was handled in lookahead above.
                "^" => continue,

                &_ => {
                    warn!(
                        "Unsupported RiveScript command '{}' found at {} line {}",
                        cmd, filename, lineno,
                    );
                }
            }
        }

        // If we had a final trigger ready to go, add it to the AST.
        if current_trigger.is_populated() {
            let t = ast.topics.get_mut(&topic).expect("or else");
            t.add_trigger(current_trigger);
        }

        Ok(ast)
    }
}
