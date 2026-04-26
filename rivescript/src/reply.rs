use async_recursion::async_recursion;
use rand::seq::IndexedRandom;

use log::{debug, warn};
use ::regex::Regex;

use crate::{RiveScript, ast, inheritance};

/// Get a reply to the username's message.
pub async fn reply(rs: &mut RiveScript, username: &str, message: &str) -> Result<String, String> {

    // Store the current user's ID.
    rs.in_reply_context = true;
    rs.current_username = String::from(username);

    let mut msg = String::from(message);
    let mut answer = String::new();

    // Format their message (run substitutions, etc.)
    msg = format_message(rs, &msg);

    debug!("Find a reply to: {msg}");

    // If the BEGIN block exists, consult it first.
    if rs.brain.has_begin_block() {
        debug!("Has a BEGIN block");
        match get_reply(rs, &rs.current_username, &String::from(rivescript_core::BEGIN_REQUEST), true, 0).await {
            Ok(begin) => {
                debug!("Answer to BEGIN request: {begin}");

                // Is it OK to continue?
                if begin.contains(rivescript_core::TAG_OK) {
                    // Get the real reply and substitute it in.
                    match get_reply(rs, &rs.current_username, &msg, false, 0).await {
                        Ok(reply) => {
                            debug!("Answer to reply request: {reply}");
                            answer = begin.replace(rivescript_core::TAG_OK, &reply);
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
        match get_reply(rs, &rs.current_username, &msg, false, 0).await {
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
        answer = String::from(rivescript_core::ERR_NO_REPLY);
    }

    // Save their message history.
    rs.sessions.add_history(username, message, &answer).await;

    // Unset the current user's ID.
    rs.current_username = String::new();
    rs.in_reply_context = false;

    return Ok(answer);
}

// Inner reply-fetching logic, called for the >BEGIN block too.
#[async_recursion]
pub async fn get_reply(
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
        topic = String::from(rivescript_core::BEGIN_TOPIC);
    } else {
        topic = rs.sessions.get(username, "topic").await;
    }

    // Collect matched regex stars.
    let mut stars: Vec<String> = Vec::new();
    let mut bot_stars: Vec<String> = Vec::new();
    let mut reply = String::new();

    // Avoid letting them fall into a missing topic.
    if !rs.brain.has_topic(&topic) {
        warn!("User {username} was in an empty topic named '{topic}' - rescuing them back to '{}'", rivescript_core::DEFAULT_TOPIC);
        topic = String::from(rivescript_core::DEFAULT_TOPIC)
    }

    // Keep a pointer to the matched Trigger once we find it.
    let mut matched: &ast::Trigger = &ast::Trigger::new("");
    let mut found_match = false;

    // See if there were any %Previous's in this topic, or any topic related to
    // it. This should only be done the first time -- not during a recursive
    // redirection. This is because in a redirection, "lastReply" is still gonna
    // be the same as it was the first time, resulting in an infinite loop!
    if step == 0 {

        // Gather all of the topics (inherits/includes).
        let this_topic = rs.brain.topics.get(&topic).unwrap();
        let all_topics = inheritance::get_topic_tree(&rs.brain, &this_topic, 0);

        // Scan all the topics.
        'previous: for topic in all_topics {
            debug!("Checking topic {topic} for any %Previous's.");

            let triggers = rs.sorted_thats.get(&topic).unwrap();
            if triggers.len() > 0 {
                debug!("There's a %Previous in this topic!");
            }

            for trig in triggers {
                // Get the bot's last reply to the user.
                let history = rs.sessions.get_history(username).await;
                let last_reply = history.reply.get(0).unwrap();

                // Format the bot's last reply the same way as the human's.
                let last_reply = format_message(rs, last_reply);
                debug!("Bot's last reply: {last_reply}");

                // See if the bot's last reply matches.
                let pattern = &trig.previous;
                let regexp = trigger_regexp(rs, username, pattern).await;
                match regexp.captures(&last_reply) {
                    Some(caps) => {
                        // Huzzah! See if OUR message is right too...
                        debug!("Bot side matched!");

                        // Collect the bot stars while we're here.
                        bot_stars = caps.iter()
                            .filter_map(
                                |m|
                                m.map(|m| m.as_str().to_string())
                            )
                            .collect();

                        // Compare the trigger to the user's message.
                        let pattern = &trig.trigger;
                        let regexp = trigger_regexp(rs, username, pattern).await;
                        match regexp.captures(message) {
                            Some(caps) => {
                                // The user side matched too!
                                // Collect the stars.
                                stars = caps.iter()
                                    .filter_map(
                                        |m|
                                        m.map(|m| m.as_str().to_string())
                                    )
                                    .collect();

                                // Mark that we found a match.
                                matched = trig;
                                found_match = true;
                                break 'previous;
                            },
                            None => continue,
                        }
                    },
                    None => continue,
                }
            }
        }

    }

    // Search their topic for a match to their trigger.
    if !found_match {
        debug!("Searching their topic ({topic}) for a match...");
        let triggers = rs.sorted_topics.get(&topic).unwrap();
        for trig in triggers {
            let pattern = &trig.trigger;
            let regexp = trigger_regexp(rs, username, pattern).await;
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
                match get_reply(&rs, &username, &redirect, false, step+1).await {
                    Ok(r) => {
                        reply = r;
                        break;
                    }
                    Err(e) => return Err(e),
                }
            }

            // Check the conditionals.
            for row in matched.condition.clone() {
                // Process tags on the left and right sides.
                let mut left = crate::tags::process(rs, username, message, &row.left, stars.clone(), bot_stars.clone(), step).await;
                let mut right = crate::tags::process(rs, username, message, &row.right, stars.clone(), bot_stars.clone(), step).await;

                // Defaults?
                if left.len() == 0 {
                    left = rivescript_core::UNDEFINED.to_string();
                }
                if right.len() == 0 {
                    right = rivescript_core::UNDEFINED.to_string();
                }

                debug!("Check if [{left}] {} [{right}]", row.operator);
                let mut passed = false;
                match row.operator.as_str() {
                    "eq" | "==" => {
                        passed = left == right;
                    },
                    "ne" | "!=" | "<>" => {
                        passed = left != right;
                    },
                    _ => {
                        // The other operators deal with numbers.
                        if let Ok(left_value) = left.parse::<i64>() {
                            if let Ok(right_value) = right.parse::<i64>() {

                                // Do the needful.
                                match row.operator.as_str() {
                                    "<" => passed = left_value < right_value,
                                    "<=" => passed = left_value <= right_value,
                                    ">" => passed = left_value > right_value,
                                    ">=" => passed = left_value >= right_value,
                                    _ => (),
                                }
                            } else {
                                warn!("Right side of condition was non-numeric: {:?}", row);
                            }
                        } else {
                            warn!("Left side of condition was non-numeric: {:?}", row);
                        }
                    },
                }

                // Did the condition pass?
                if passed {
                    reply = row.reply;
                }
            }

            // Have our reply yet?
            if !reply.is_empty() {
                break;
            }

            // We are down to the final -Reply tags.
            // Process {weight} in the replies.
            let mut bucket: Vec<String> = Vec::new();
            for rep in matched.reply.clone() {
                match crate::regex::WEIGHT.captures(&rep) {
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
        return Ok(String::from(rivescript_core::ERR_NO_MATCH));
    } else if reply.is_empty() {
        return Ok(String::from(rivescript_core::ERR_NO_REPLY));
    }

    // Process tags for the BEGIN block.
    if is_begin {
        // TODO: set topic and user vars.
    } else {
        reply = crate::tags::process(&rs, &username, &message, &reply, stars, bot_stars, step).await;
    }
    // TODO

    Ok(String::from(reply))
}

// Format the input message for safe processing.
pub fn format_message(rs: &RiveScript, msg: &str) -> String {
    let mut msg = String::from(msg);

    // Lowercase it.
    if !rs.case_sensitive {
        msg = msg.to_lowercase();
    }

    // Run substitutions and sanitize what's left.
    msg = crate::tags::substitute(rs.brain.subs.clone(), rs.sorted_subs.clone(), &msg);

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
pub async fn trigger_regexp(rs: &RiveScript, username: &String, pattern: &String) -> Regex {
    let mut pattern = pattern.clone();

    // If the trigger is simply '*' then the * needs to become (.*?)
    // instead of the usual (.+?), to match the blank string too.
    pattern = crate::regex::ZERO_WIDTH_STAR.replace_all(&pattern, "<zerowidthstar>").to_string();

    // Simple replacements.
    pattern = pattern.replace("*", r"(.+?)");  // *
    pattern = pattern.replace("#", r"(\d+?)"); // #
    pattern = pattern.replace("_", r"(\w+?)"); // _

    // Remove {weight} and {inherits}
    pattern = crate::regex::WEIGHT.replace_all(&pattern, "").to_string();
    pattern = crate::regex::INHERITS.replace_all(&pattern, "").to_string();

    // Recover the zero-width star.
    pattern = pattern.replace("<zerowidthstar>", r"(.*?)");

    // UTF-8 mode special characters.
    if rs.utf8 {
        // Literal @ symbols (like in an e-mail address) conflict with arrays. If the RiveScript
        // trigger had an escaped '\@' sequence, move it out of the way first.
        pattern = pattern.replace(r"\@", r"\u0040");
    }

    // Optionals.
    for (_, [inner]) in crate::regex::TRIGGER_OPTIONALS.captures_iter(&pattern.clone()).map(|c| c.extract()) {
        let parts: Vec<String> = inner.split("|").map(|s| s.to_string()).collect();
        let mut options: Vec<String> = Vec::new();

        for p in parts {
            options.push(format!(r"(?:\s|\b)+{}(?:\s|\b)+", p));
        }

        // If this optional had a star or anything in it, make it non-capturing.
        let mut pipes = options.join("|");
        pipes = pipes.replace(r"(.+?)", r"(?:.+?)");
        pipes = pipes.replace(r"(\d+?)", r"(?:\d+?)");
        pipes = pipes.replace(r"(\w+?)", r"(?:\w+?)");

        // Substitute the original [optional] greedily back into the pattern.
        let qm = regex::escape(&inner);
        let replacement = format!(r"(?:{}|(?:\s+|\b))", pipes);

        pattern = Regex::new(&String::from(format!(r"\s*\[{qm}\]\s*")))
            .unwrap()
            .replace_all(&pattern, &replacement)
            .to_string();
    }

    // Don't let _ wildcards match numbers!
    // A quick note on why it's this way: the initial replacement above that
    // swaps (_ => (\w+?)) needed to be \w because the square brackets
    // in [\s\d] will confuse the optionals logic just above. So then we
    // switch it back down here. Also, we don't just use \w+ because that
    // will match digits, and similarly [A-Za-z] would not match Unicode.
    pattern = pattern.replace(r"\w", r"[^\s\d]");

    // Filter in arrays.
    for (m, [name]) in crate::regex::TRIGGER_ARRAY.captures_iter(&pattern.clone()).map(|c| c.extract()) {
        let mut replacement = String::new();
        if let Some(items) = rs.brain.arrays.get(name) {
            replacement = format!(r"(?:{})", items.join("|"));
        }
        pattern = pattern.replace(m, &replacement);
    }

    // Filter in bot variables.
    for (m, [name]) in crate::regex::BOT_TAG.captures_iter(&pattern.clone()).map(|c| c.extract()) {
        let mut replacement = rs.brain.get_bot_var(name);
        replacement = strip_nasties(replacement).to_lowercase();
        pattern = pattern.replace(m, &replacement);
    }

    // Filter in <get> user variables.
    for (m, [name]) in crate::regex::USER_VAR_TAG.captures_iter(&pattern.clone()).map(|c| c.extract()) {
        let mut replacement = rs.sessions.get(username, name).await;
        replacement = strip_nasties(replacement).to_lowercase();
        pattern = pattern.replace(m, &replacement);
    }

    // Filter in <input>/<reply> tags.
    if pattern.contains("<input") || pattern.contains("<reply") {
        pattern = pattern.replace("<input>", "<input1>");
        pattern = pattern.replace("<reply>", "<reply1>");
        let history = rs.sessions.get_history(username).await;

        for (_, [number]) in crate::regex::HISTORY_TAG.captures_iter(&pattern.clone()).map(|c| c.extract()) {
            let mut idx: usize = 1;
            if !number.is_empty() {
                idx = number.parse().unwrap();
            }

            let input = history.input.get(idx-1).unwrap();
            let reply = history.reply.get(idx-1).unwrap();

            // Format the previous inputs for the regexp engine.
            let input = &format_message(rs, input);
            let reply = &format_message(rs, reply);

            pattern = pattern.replace(
                &String::from(format!("<input{idx}>")),
                input,
            );
            pattern = pattern.replace(
                &String::from(format!("<reply{idx}>")),
                reply,
            );
        }
    }

    // Recover escaped Unicode symbols (@ signs).
    if rs.utf8 && pattern.contains(r"\u") {
        // TODO: make it more general.
        pattern = pattern.replace(r"\u0040", "@");
    }

    pattern = String::from(format!(r"^{}$", pattern));

    Regex::new(&pattern).unwrap_or(Regex::new("").unwrap())
}

pub fn strip_nasties(msg: String) -> String {
    let mut msg = msg.clone();
    msg = crate::regex::NASTIES.replace_all(&msg, "").to_string();
    msg
}