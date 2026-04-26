use std::collections::HashMap;

use crate::macros::proxy::Proxy;

use rand::seq::IndexedRandom;
use shell_words;

use log::debug;
use regex::Regex;

/// Process RiveScript tags in a reply segment.
pub async fn process(
    rs: &crate::RiveScript,
    username: &String,
    _message: &String, // TODO: needed?
    reply: &String,
    stars: Vec<String>,
    bot_stars: Vec<String>,
    step: usize,
) -> String {
    let mut reply = reply.clone();

    // Format the stars (putting 'undefined' for unused star values).
    let stars = pad_stars(stars);
    let bot_stars = pad_stars(bot_stars);

    // Turn (@arrays) into randomized sets.
    for (m, [name]) in crate::regex::REPLY_ARRAY.captures_iter(&reply.clone()).map(|c| c.extract()) {
        let result: String;
        if let Some(entries) = rs.brain.arrays.get(name) {
            // Substitute it in for a {random} tag.
            result = format!("{{random}}{}{{/random}}", entries.join("|"));
        } else {
            // Dummy out the literal array (it didn't exist) so we can reconstruct it later.
            result = String::from(format!("\x00@{name}\x00"));
        }

        reply = reply.replacen(m, &result, 1);
    }

    // Tag shortcuts.
    reply = reply.replace("<person>", "{person}<star>{/person}");
    reply = reply.replace("<@>", "{@<star>}");
    reply = reply.replace("<formal>", "{formal}<star>{/formal}");
    reply = reply.replace("<sentence>", "{sentence}<star>{/sentence}");
    reply = reply.replace("<uppercase>", "{uppercase}<star>{/uppercase}");
    reply = reply.replace("<lowercase>", "{lowercase}<star>{/lowercase}");

    // Weight and star tags.
    reply = crate::regex::WEIGHT.replace_all(&reply, "").to_string();
    reply = reply.replace("<star>", "<star1>");
    reply = reply.replace("<botstar>", "<botstar1>");
    for i in 1..rivescript_core::MAX_STARS {
        reply = reply.replace(
            &String::from(format!("<star{i}>")),
            stars.get(i).unwrap(),
        );
        reply = reply.replace(
            &String::from(format!("<botstar{i}>")),
            bot_stars.get(i).unwrap(),
        );
    }

    // <input> and <reply>
    reply = reply.replace("<input>", "<input1>");
    reply = reply.replace("<reply>", "<reply1>");
    let history = rs.sessions.get_history(username).await;
    for i in 1..rivescript_core::MAX_HISTORY+1 {
        reply = reply.replace(&String::from(format!("<input{i}>")), &history.input.get(i-1).unwrap_or(&rivescript_core::UNDEFINED.to_string()));
        reply = reply.replace(&String::from(format!("<reply{i}>")), &history.reply.get(i-1).unwrap_or(&rivescript_core::UNDEFINED.to_string()));
    }

    // <id> and escape codes.
    reply = reply.replace("<id>", username);
    reply = reply.replace(r"\s", " ");
    reply = reply.replace(r"\n", "\n");
    reply = reply.replace(r"\#", "#");

    // {random}
    for (m, [inner]) in crate::regex::RANDOM_TAG.captures_iter(&reply.clone()).map(|c| c.extract()) {
        let random: Vec<String> = inner.split("|").map(|s| s.to_string()).collect();
        let mut rng = rand::rng();
        if let Some(selection) = &random[..].choose(&mut rng) {
            reply = reply.replacen(m, &selection.to_string(), 1);
        }
    }

    // String formatting tags.
    reply = run_format_tag(rs, "person", &crate::regex::PERSON_TAG, &reply);
    reply = run_format_tag(rs, "formal", &crate::regex::FORMAL_TAG, &reply);
    reply = run_format_tag(rs, "sentence", &crate::regex::SENTENCE_TAG, &reply);
    reply = run_format_tag(rs, "uppercase", &crate::regex::UPPERCASE_TAG, &reply);
    reply = run_format_tag(rs, "lowercase", &crate::regex::LOWERCASE_TAG, &reply);

    // Handle all variable-related tags with an iterative regexp approach to
    // allow for nesting of tags in arbitrary ways (think <set a=<get b>>).
    // Move the <call> tags out of the way first.
    reply = reply.replace("<call>", "{__call__}");
    reply = reply.replace("</call>", "{/__call__}");
    for (m, [tag_body]) in crate::regex::ANY_TAG.captures_iter(&reply.clone()).map(|c| c.extract()) {
        let parts: Vec<String> = tag_body.splitn(2, " ").map(|s| s.to_string()).collect();
        let tag = parts.get(0).unwrap();
        let data = parts.get(1).map(|s| s.as_str()).unwrap_or("");
        let mut insert = String::new();

        // Handle the various types of tags.
        match tag.as_str() {
            "bot" | "env" => {
                // <bot> and <env> work similarly.
                if data.contains("=") {
                    // Doing an assignment.
                    let parts: Vec<String> = data.splitn(2, "=").map(|s| s.to_string()).collect();
                    let name = parts.get(0).unwrap();
                    let value = parts.get(1).map(|s| s.as_str()).unwrap_or("");

                    if tag == "bot" {
                        rs.brain.set_bot_var(name, value);
                    } else {
                        rs.brain.set_global(name, value);
                    }
                } else {
                    if tag == "bot" {
                        insert = rs.brain.get_bot_var(&data);
                    } else {
                        insert = rs.brain.get_global(&data);
                    }
                }
            },
            "set" => {
                // <set> a user variable.
                let parts: Vec<String> = data.splitn(2, "=").map(|s| s.to_string()).collect();
                let name = parts.get(0).unwrap();
                let value = parts.get(1).map(|s| s.as_str()).unwrap_or("");
                rs.sessions.set(username, HashMap::from([
                    (name.to_string(), value.to_string()),
                ])).await;
            },
            "add" | "sub" | "mult" | "div" => {
                // Math operator tags.
                let parts: Vec<String> = data.splitn(2, "=").map(|s| s.to_string()).collect();
                let name = parts.get(0).unwrap();
                let value_str = parts.get(1).map(|s| s.as_str()).unwrap_or("");

                // Initialize a numeric value?
                let mut orig_str = rs.sessions.get(username, &name).await;
                if orig_str == rivescript_core::UNDEFINED {
                    orig_str = String::from("0");
                    rs.sessions.set(username, HashMap::from([
                        (name.to_string(), orig_str.to_string()),
                    ])).await;
                }

                // Cast the original to a number.
                if let Ok(mut orig_value) = orig_str.parse::<i64>() {
                    // Cast the operand to a number.
                    if let Ok(operand) = value_str.parse::<i64>() {

                        // Do the needful.
                        let mut math_ok = true;
                        match tag.as_str() {
                            "add" => {
                                orig_value += operand;
                            },
                            "sub" => {
                                orig_value -= operand;
                            },
                            "mult" => {
                                orig_value *= operand;
                            },
                            "div" => {
                                if operand == 0 {
                                    insert = format!("[ERR: Can't Divide By Zero]");
                                    math_ok = false;
                                } else {
                                    orig_value /= operand;
                                }
                            }
                            _ => (),
                        }

                        // Successful math? Save it back to their storage.
                        if math_ok {
                            rs.sessions.set(username, HashMap::from([
                                (name.to_string(), format!("{orig_value}")),
                            ])).await;
                        }

                    } else {
                        insert = format!("[ERR: Math can't '{tag}' a non-numeric value '{value_str}' to the user variable '{name}']");
                    }
                } else {
                    insert = format!("[ERR: The stored user variable '{name}' contains a non-numeric value '{orig_str}'; can not '{tag}' to it]");
                }
            },
            "get" => {
                // <get> a user variable.
                insert = rs.sessions.get(username, &data).await;
            }
            _ => {
                // Unrecognized tag; preserve it.
                insert = format!("\x00{tag_body}\x01");
            },
        }

        reply = reply.replacen(m, &insert, 1);
    }

    // Recover mangled HTML-like tags from the above loop.
    reply = reply.replace("\x00", "<");
    reply = reply.replace("\x01", ">");

    // Topic setter.
    match crate::regex::TOPIC_TAG.captures(&reply) {
        Some(caps) => {
            let topic = caps.get(1).unwrap().as_str();
            debug!("Change user topic to: {topic}");
            rs.sessions.set(username, HashMap::from([
                ("topic".to_string(), String::from(topic)),
            ])).await;
            reply = reply.replace(caps.get_match().as_str(), "");
        },
        None => (),
    }

    // Inline redirector.
    for (m, [pattern]) in crate::regex::REDIRECT_TAG.captures_iter(&reply.clone()).map(|c| c.extract()) {
        debug!("Inline redirection to: {pattern}");
        match crate::reply::get_reply(&rs, &username, &pattern.to_string(), false, step+1).await {
            Ok(subreply) => {
                reply = reply.replace(m, &subreply);
            }
            Err(_) => (),
        };
    }

    // Finally, handle object macros.
    reply = reply.replace("{__call__}", "<call>");
    reply = reply.replace("{/__call__}", "</call>");
    {
        let captures: Vec<_> = crate::regex::CALL_TAG
            .captures_iter(&reply.clone())
            .map(|c| {
                let (full_match, [inner]) = c.extract();
                (full_match.to_string(), inner.to_string())
            }).collect();

        for (full_tag, inner_text) in captures {
            // Parse the arguments.
            let mut parts = inner_text.splitn(2, ' ');
            let name = parts.next().unwrap_or("");
            let value_str = parts.next().unwrap_or("");

            // Parse the arguments with shell-style quoting supported.
            // If there are unbalanced quotes, split by whitespace instead.
            let args = shell_words::split(value_str)
                .unwrap_or_else(|_| value_str.split_whitespace().map(str::to_string).collect());

            // Find the object macro handler/subroutine to call.
            let sub_result = {
                let mut proxy = Proxy::new(&rs, username.to_string());

                // A Rust function?
                if let Some(sub) = rs.subroutines.get(name) {
                    sub(&mut proxy, args).await
                } else {
                    Err(format!("[object {name} not found]"))
                }
            };

            let replacement = match sub_result {
                Ok(finisher) => {
                    if !finisher.staged_user_vars.is_empty() {
                        rs.sessions.set(username, finisher.staged_user_vars).await;
                    }
                    if !finisher.staged_bot_vars.is_empty() {
                        for (k, v) in finisher.staged_bot_vars {
                            rs.brain.set_bot_var(&k, &v);
                        }
                    }
                    finisher.output
                },
                Err(e) => e,
            };

            reply = reply.replace(&full_tag, &replacement);
        }
    }

    reply.clone()
}

// Star tags can hold an index from 1-9 corresponding to regex capture groups. Pad the unused values with 'undefined'.
fn pad_stars(stars: Vec<String>) -> Vec<String> {
    if stars.len() == 10 {
        return stars;
    }

    let mut stars = stars.clone();
    let cap = stars.len();

    // Note: stars[0] is the full regex capture and stars[1..9] are <star1> thru <star9>.
    // In the end the stars.len() should be 10 to include values for all 9 stars.

    for i in 0..rivescript_core::MAX_STARS+1 {
        if cap < i+1 {
            stars.push(rivescript_core::UNDEFINED.to_string());
        }
    }

    debug_assert_eq!(stars.len(), rivescript_core::MAX_STARS+1);

    stars
}

/// Run substitutions or person substitutions on a string.
pub fn substitute(map: HashMap<String, String>, sorted: Vec<String>, message: &String) -> String {
    let mut message = message.clone();

    // Safety check.
    if map.is_empty() {
        return message;
    }

    // Make placeholders each time we substitute something, so we don't process
    // the same part of the string too many times.
    let mut ph: Vec<String> = Vec::new();
    let mut pi = 0;

    for pattern in sorted {

        if !message.contains(&pattern) {
            continue;
        }

        let result = map.get(&pattern).unwrap();
        let qm = regex::escape(&pattern);

        // Make a placeholder.
        ph.push(result.to_string());
        let placeholder = format!("\x00{pi}\x00");
        pi = pi + 1;

        // Run substitutions.
        message = Regex::new(&String::from(format!(r"^{qm}$")))
            .unwrap()
            .replace_all(&message, &placeholder)
            .to_string();
        message = Regex::new(&String::from(format!(r"^{qm}(\W+)")))
            .unwrap()
            .replace_all(&message, String::from(format!("{}$1", &placeholder)))
            .to_string();
        message = Regex::new(&String::from(format!(r"(\W+){qm}(\W+)")))
            .unwrap()
            .replace_all(&message, String::from(format!("$1{}$2", &placeholder)))
            .to_string();
        message = Regex::new(&String::from(format!(r"(\W+){qm}$")))
            .unwrap()
            .replace_all(&message, String::from(format!("$1{}", &placeholder)))
            .to_string();
    }

    // Convert the placeholders back in.
    for (m, [id]) in crate::regex::PLACEHOLDER.captures_iter(&message.clone()).map(|c| c.extract()) {
        let id: usize = id.parse().unwrap();
        if let Some(result) = ph.get(id) {
            message = message.replace(
                m,
                result,
            );
        }
    }

    message
}

/// Process string format tags (uppercase, lowercase, formal, sentence).
fn run_format_tag(rs: &crate::RiveScript, tag: &str, re: &regex::Regex, reply: &String) -> String {
    let mut message = reply.clone();

    for (m, [inner]) in re.captures_iter(&reply).map(|c| c.extract()) {
        debug!("m: {m} inner: {inner}");

        // Person substitutions?
        if tag == "person" {
            let result = substitute(rs.brain.person.clone(), rs.sorted_person.clone(), &String::from(inner));
            message = message.replace(m, &result);
            continue;
        }

        message = message.replace(m, &format_string(tag, &String::from(inner)));
    }

    message
}

/// Format a string (uppercase, lowercase, sentence, formal).
fn format_string(tag: &str, value: &String) -> String {
    let mut value = value.clone();

    match tag {
        "uppercase" => {
            value = value.to_uppercase();
        },
        "lowercase" => {
            value = value.to_lowercase();
        },
        "sentence" => {
            value = value.to_lowercase();
            let mut c = value.chars();
            return match c.next() {
                None => value,
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        },
        "formal" => {
            let words = value.split_whitespace().collect::<Vec<&str>>();
            let mut formalized: Vec<String> = Vec::new();
            for word in words {
                formalized.push(format_string("sentence", &String::from(word)));
            }
            return formalized.join(" ");
        },
        _ => return value,
    }

    value
}