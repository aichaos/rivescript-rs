// Topic inheritance helper functions.

use std::collections::HashSet;

use log::{debug, warn};

use crate::ast;

/// Recursively scan topics and collect all of its triggers.
///
/// This function scans through a topic, along with any topics that it includes
/// or inherits from. Some triggers will come out having an {inherits} tag to
/// track the inheritance depth (for matching prioritization purposes).
///
/// For collectiong triggers with %Previous, provide a true value for `thats.`
pub fn get_topic_triggers(brain: &ast::AST, topic: &ast::Topic, thats: bool) -> Vec<ast::Trigger> {
    _get_topic_triggers(brain, topic, thats, 0, 0, false)
}

/// The inner logic for get_topic_triggers.
///
/// Additional private parameters include:
///
/// - depth: Recursion depth counter.
/// - inheritance: inheritance counter.
/// - inherited: inheritance status.
///
/// Important info about the depth vs. inheritance params to this function:
/// depth incrmeents by 1 each time this function recursively calls itself.
/// inheritance only increments by 1 when this topic inherits another topic.
///
/// This way, `> topic alpha includes beta inherits gamma` will have this effect:
/// - alpha and beta's triggers are combined together into one matching pool,
/// - and then those triggers all have higher priority than gamma's.
///
/// The inherited option is true if this is a recursive call, from a topic that
/// inherits other topics. This forces the {inherits} tag to be added to the
/// triggers. This only applies when the topic 'includes' another topic.
fn _get_topic_triggers(brain: &ast::AST, topic: &ast::Topic, thats: bool, depth: usize, inheritance: usize, inherited: bool) -> Vec<ast::Trigger> {

    // Break if we're in too deep.
    // TODO: rs.depth?
    if depth > crate::DEFAULT_DEPTH {
        warn!("Deep recursion while scanning topic inheritance!");
        return Vec::new();
    }

    // All triggers to return.
    let mut triggers: Vec<ast::Trigger> = Vec::new();
    let mut seen_triggers: HashSet<String> = HashSet::new();

    // Gather the ones belonging to this topic specifically.
    let mut in_this_topic: Vec<ast::Trigger> = Vec::new();
    let does_inherit = !topic.inherits.is_empty();
    for trigger in topic.triggers.clone() {
        if !thats {
            if trigger.previous.is_empty() {
                in_this_topic.push(trigger);
            }
        } else if !trigger.previous.is_empty() {
            in_this_topic.push(trigger);
        }
    }

    // 1. Process FIRST-PARTY triggers first.
    // This allows local triggers to "mask" identical ones in Included topics.
    for mut trigger in in_this_topic {
        seen_triggers.insert(trigger.trigger.clone());

        // Tag it with an {inherits} tag so it will be sorted correctly in sort_triggers().
        if inheritance > 0 || does_inherit || inherited {
            trigger.trigger = format!("{{inherits={}}}{}", inheritance, trigger.trigger);
        }
        triggers.push(trigger);
    }

    // 2. Process INCLUDES (mix-ins)
    // Included triggers are treated as same-level priority as local triggers.
    // Their triggers may be "masked" by duplicates in the local topic.
    if !topic.includes.is_empty() {

        for topic_name in topic.includes.keys() {
            debug!("Topic {} includes {:?}", topic.name, topic_name);
            let subtopic = brain.topics.get(topic_name).unwrap();

            // Gather its triggers and append to our running set.
            let append = _get_topic_triggers(brain, subtopic, thats, depth+1, inheritance, false);

            // Append the non-duplicate (masked) triggers.
            for t in append {
                if !seen_triggers.contains(&t.trigger) {
                    // Add it to seen_triggers too in case of multiple includes.
                    seen_triggers.insert(t.trigger.clone());

                    // Add to the running list.
                    triggers.push(t);
                }
            }
        }
    }

    // 3. Process INHERITS (fallbacks).
    // All of these triggers will have lower priority.
    if does_inherit {
        for topic_name in topic.inherits.keys() {
            debug!("Topic {} inherits {:?}", topic.name, topic_name);
            let subtopic = brain.topics.get(topic_name).unwrap();

            // Gather its triggers and append to our running set.
            let append = _get_topic_triggers(brain, subtopic, thats, depth+1, inheritance+1, true);
            triggers.extend(append);
        }
    }

    // Combine the trigger sets together.
    // If this topic inherited any others, it means that this topic's triggers
    // have a higher priority than those in inherited topics. Enforce this with
    // an {inherits} tag added to each inherited trigger.
    // if does_inherit || inherited {
    //     for trigger in in_this_topic {
    //         debug!("Prefixing trigger with {{inherits={}}} {}", inheritance, trigger.trigger);
    //         let mut trigger = trigger.clone();
    //         trigger.trigger = format!("{{inherits={}}}{}", inheritance, trigger.trigger);
    //         triggers.push(trigger);
    //     }
    // } else {
    //     triggers.extend(in_this_topic);
    // }

    triggers
}

/// Get a list of every topic name related to a topic (all of its includes/inherits).
pub fn get_topic_tree(brain: &ast::AST, topic: &ast::Topic, depth: usize) -> Vec<String> {
    let mut topics: Vec<String> = Vec::new();

    if depth > crate::DEFAULT_DEPTH {
        warn!("Deep recursion while scanning topic tree!");
        return topics;
    }

    topics.push(topic.name.clone());

    for includes in topic.includes.keys() {
        let subtopic = brain.topics.get(includes).unwrap();
        topics.extend(get_topic_tree(brain, subtopic, depth+1));
    }

    for inherits in topic.inherits.keys() {
        let subtopic = brain.topics.get(inherits).unwrap();
        topics.extend(get_topic_tree(brain, subtopic, depth+1));
    }

    topics
}