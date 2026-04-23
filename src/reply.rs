use std::error::Error;
use rand::seq::IndexedRandom;

use log::{debug, warn};
use ::regex::Regex;

use crate::{RiveScript, ast};
use crate::errors::ParseError;
use crate::regex;

struct Message {
    username: String,
    message: String,
}

/// Get a reply to the username's message.
pub async fn reply(rs: &mut RiveScript, username: &str, message: &str) -> Result<String, String> {

    // TODO: Initialize a user profile for the user.
    // rs.sessions.init(username)

    // Store the current user's ID.
    rs.in_reply_context = true;
    rs.current_username = String::from(username);

    let mut msg = String::from(message);
    let mut answer = String::new();

    // Format their message (run substitutions, etc.)
    msg = format_message(rs, msg);

    debug!("Find a reply to: {msg}");

    // If the BEGIN block exists, consult it first.
    if rs.brain.has_begin_block() {
        debug!("Has a BEGIN block");
        match get_reply(rs, &rs.current_username, &String::from(crate::BEGIN_REQUEST), true, 0) {
            Ok(begin) => {
                debug!("Answer to BEGIN request: {begin}");

                // Is it OK to continue?
                if begin.contains(crate::TAG_OK) {
                    // Get the real reply and substitute it in.
                    match get_reply(rs, &rs.current_username, &msg, false, 0) {
                        Ok(reply) => {
                            debug!("Answer to reply request: {reply}");
                            answer = begin.replace(crate::TAG_OK, &reply);
                        },
                        Err(e) => {
                            return Err(e);
                        },
                    }
                }
            },
            Err(e) => {
                return Err(e);
            },
        }
    } else {
        debug!("No BEGIN block");
        match get_reply(rs, &rs.current_username, &msg, false, 0) {
            Ok(reply) => {
                debug!("Answer to reply request: {reply}");
                answer = reply
            },
            Err(e) => {
                return Err(e);
            },
        }
    }

    if answer.is_empty() {
        answer = String::from(crate::ERR_NO_REPLY);
    }
    return Ok(answer);
}

// Inner reply-fetching logic, called for the >BEGIN block too.
fn get_reply(
    rs: &RiveScript,
    username: &String,
    message: &String,
    is_begin: bool,
    step: usize,
) -> Result<String, String> {

    // They forgot to sort replies?
    if rs.sorted_topics.is_empty() {
        return Err("You forgot to call sort_replies()".to_string());
    }

    // Avoid deep recursion.
    if step > rs.depth {
        return Err("Deep recursion detected".to_string())
    }

    // What topic are we in?
    let mut topic: String;
    if is_begin {
        topic = String::from(crate::BEGIN_TOPIC);
    } else {
        // TODO: user vars
        topic = String::from(crate::DEFAULT_TOPIC);
    }

    // Collect matched regex stars.
    let mut stars: Vec<String> = Vec::new();
    let mut that_stars: Vec<String> = Vec::new();
    let mut reply = String::new();

    // Avoid letting them fall into a missing topic.
    if !rs.brain.has_topic(&topic) {
        warn!("User {username} was in an empty topic named '{topic}'");
        topic = String::from(crate::DEFAULT_TOPIC)
    }

    // Keep a pointer to the matched Trigger once we find it.
    let mut matched: &ast::Trigger = &ast::Trigger::new("");
    let mut found_match = false;

    // See if there were any %Previous's in this topic, or any topic related to
    // it. This should only be done the first time -- not during a recursive
    // redirection. This is because in a redirection, "lastReply" is still gonna
    // be the same as it was the first time, resulting in an infinite loop!
    if step == 0 {
        // TODO
    }

    // Search their topic for a match to their trigger.
    if !found_match {
        debug!("Searching their topic for a match...");
        let triggers = rs.sorted_topics.get(&topic).unwrap();
        for trig in triggers {
            let pattern = &trig.trigger;
            let regexp = trigger_regexp(username, pattern);
            debug!("Compare:{regexp}");

            match regexp.captures(message) {
                Some(caps) => {
                    stars = caps.iter()
                        .filter_map(
                            |m|
                            m.map(|m| m.as_str().to_string())
                        )
                        .collect();

                    // We found a match!
                    matched = trig;
                    found_match = true;

                    debug!("Matched: stars={:?}", stars);
                    break;
                },
                None => continue,
            }
        }
    }

    // Store what trigger they last matched on.
    // TODO

    // Did we find a match after all?
    if found_match {

        // A single loop so we can break early.
        loop {

            // See if there are any hard redirects.
            if !matched.redirect.is_empty() {
                debug!("Redirecting us to: {}", matched.redirect);
                let mut redirect = matched.redirect.clone();
                // redirect = process_tags(username, ...)
                redirect = redirect.to_lowercase();

                debug!("Pretend user said: {redirect}");
                match get_reply(&rs, &username, &redirect, false, step+1) {
                    Ok(r) => {
                        reply = r;
                        break;
                    }
                    Err(e) => return Err(e),
                }
            }

            // Check the conditionals.
            // TODO

            // Have our reply yet?
            if !reply.is_empty() {
                break;
            }

            // We are down to the final -Reply tags.
            // Process {weight} in the replies.
            let mut bucket: Vec<String> = Vec::new();
            for rep in matched.reply.clone() {
                match regex::WEIGHT.captures(&rep) {
                    Some(caps) => {
                        let weight: usize = caps.get(1).unwrap().as_str().parse().unwrap();
                        for _ in 0..weight {
                            bucket.push(rep.clone());
                        }
                    }
                    None => {
                        bucket.push(rep);
                    }
                }
            }

            // Get a random reply.
            if !bucket.is_empty() {
                let mut rng = rand::rng();
                if let Some(selection) = &bucket[..].choose(&mut rng) {
                    reply = selection.to_string();
                } else {
                    return Err("No random replies!".to_string());
                }
            }

            break;
        }
    }

    // Still no reply?? Give up with the fallback error replies.
    if !found_match {
        return Ok(String::from(crate::ERR_NO_MATCH));
    } else if reply.is_empty() {
        return Ok(String::from(crate::ERR_NO_REPLY));
    }

    // Process tags for the BEGIN block.
    // TODO

    Ok(String::from(reply))
}

// Format the input message for safe processing.
pub fn format_message(rs: &RiveScript, msg: String) -> String {
    let mut msg = msg.clone();

    // Lowercase it.
    if !rs.case_sensitive {
        msg = msg.to_lowercase();
    }

    // Run substitutions and sanitize what's left.
    msg = substitute(rs, msg);

    // In UTF-8 mode, only strip metacharacters and HTML brackets.
    if rs.utf8 {
        // TODO
    } else {
        // For everything else, strip all non-alphanumerics.
        msg = strip_nasties(msg);
    }

    msg
}

// Prepare a trigger pattern for the regular expression engine.
pub fn trigger_regexp(username: &String, pattern: &String) -> Regex {
    let mut pattern = pattern.clone();

    // If the trigger is simply '*' then the * needs to become (.*?)
    // instead of the usual (.+?), to match the blank string too.
    pattern = regex::ZERO_WIDTH_STAR.replace_all(&pattern, "<zerowidthstar>").to_string();

    // Simple replacements.
    pattern = pattern.replace("*", r"(.+?)");  // *
    pattern = pattern.replace("#", r"(\d+?)"); // #
    pattern = pattern.replace("_", r"(\w+?)"); // _

    // Remove {weight} and {inherits}
    pattern = regex::WEIGHT.replace_all(&pattern, "").to_string();
    pattern = regex::INHERITS.replace_all(&pattern, "").to_string();

    // Recover the zero-width star.
    pattern = pattern.replace("<zerowidthstar>", r"(.*?)");

    // UTF-8 mode special characters.
    // TODO

    // TODO: Optionals, Arrays, Bot/User Vars, Input/Reply Tags

    pattern = String::from(format!(r"^{}$", pattern));

    Regex::new(&pattern).unwrap_or(Regex::new("").unwrap())
}

pub fn substitute(rs: &RiveScript, msg: String) -> String {
    let mut msg = msg.clone();

    msg
}

pub fn strip_nasties(msg: String) -> String {
    let mut msg = msg.clone();
    msg = regex::NASTIES.replace_all(&msg, "").to_string();
    msg
}