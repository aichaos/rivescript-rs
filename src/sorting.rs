// Data sorting logic.

use itertools::Itertools;
use log::{debug, warn};

use crate::{ast, regex, errors::ParseError};
use std::{collections::HashMap, error::Error};

struct SortTracker {
    atomic: HashMap<i32, Trigger>, // Sort atomic triggers by number of whole words
    option: HashMap<i32, Trigger>, // Sort optionals by number of words
    alpha: HashMap<i32, Trigger>,  // Sort alpha wildcards by no. of words
    number: HashMap<i32, Trigger>, // Sort numeric wildcards by no. of words
    wild: HashMap<i32, Trigger>,   // Sort wildcard triggers by no. of words
    pound: Vec<Trigger>,           // Triggers of just '#'
    under: Vec<Trigger>,           // Triggers of just '_'
    star: Vec<Trigger>,            // Triggers of just '*'
}

struct Trigger {
    text: String,
    pointer: ast::Trigger,
}

/// Process the parsed AST of your bot and create optimal sort buffers
/// for trigger texts (with longer, more specific triggers on top and
/// ones with wildcards below).
pub fn sort_triggers(mut brain: &ast::AST) -> Result<bool, ParseError> {
    let mut sorted_topics: HashMap<String, Vec<Trigger>> = HashMap::new();

    // If there are no topics, give an error.
    if brain.topics.len() == 0 {
        return Err(ParseError::new(
            "sort_triggers: no topics were found. Did you load any RiveScript code?",
        ));
    }

    println!("Sorting triggers...");

    // let keys = triggers.topics.into_iter().collect();

    // Loop through all the topics.
    for name in brain.topics.keys() {
        let topic = brain.topics.get(name).unwrap();
        debug!("Analyzing topic {}", name);

        // TODO: inherits/includes

        // Sort these triggers.
        sorted_topics.insert(name.to_string(), sort_trigger_set(topic.triggers.to_vec()));
    }

    Ok(true)
}

/// Sort a group of triggers in an optimal sorting order.
fn sort_trigger_set(triggers: Vec<ast::Trigger>) -> Vec<Trigger> {
    // The running sort buffer of triggers as we add them.
    let sorted: Vec<Trigger> = Vec::new();

    // Create a priority map, of priority numbers -> their triggers (for {weight} tags)
    let mut prior: HashMap<isize, Vec<ast::Trigger>> = HashMap::new();

    // Map the incoming triggers into their priority buckets.
    for trigger in triggers {
        let mut weight: isize = 0;
        match regex::WEIGHT.captures(trigger.trigger.as_str()) {
            Some(cap) => {
                weight = cap.get(1).unwrap().as_str().parse::<isize>().unwrap_or(0);
            },
            None => (),
        }

        if !prior.contains_key(&weight) {
            prior.insert(weight, Vec::new());
        }
        let mut vt: Vec<ast::Trigger> = prior.get(&weight).unwrap().to_vec();
        vt.insert(vt.len(), trigger);
        prior.insert(weight, vt);
    }

    warn!("Prior map: {:#?}", prior);

    // Sort the priority values by largest first.
    for key in prior.keys().sorted().rev() {
        warn!("Key: {}", key);
    }

    sorted
}