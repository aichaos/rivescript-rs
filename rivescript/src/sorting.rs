// Data sorting logic.

use itertools::Itertools;
use log::{debug, warn};

use crate::{ast, inheritance, errors::ParseError};
use std::{cmp::Reverse, collections::HashMap};

// struct Trigger {
//     text: String,
//     pointer: ast::Trigger,
// }

pub struct SortedResult {
    pub topics: HashMap<String, Vec<ast::Trigger>>,
    pub thats: HashMap<String, Vec<ast::Trigger>>,
    pub subs: Vec<String>,
    pub person: Vec<String>,
}

/// Process the parsed AST of your bot and create optimal sort buffers
/// for trigger texts (with longer, more specific triggers on top and
/// ones with wildcards below).
pub fn sort_triggers(brain: &ast::AST) -> Result<SortedResult, ParseError> {
    let mut result = SortedResult{
        topics: HashMap::new(),
        thats: HashMap::new(),
        subs: Vec::new(),
        person: Vec::new(),
    };

    // If there are no topics, give an error.
    if brain.topics.len() == 0 {
        return Err(ParseError::new(
            "sort_triggers: no topics were found. Did you load any RiveScript code?",
        ));
    }

    warn!("Sorting triggers...");

    // let keys = triggers.topics.into_iter().collect();

    // Loop through all the topics.
    for name in brain.topics.keys() {
        let topic = brain.topics.get(name).unwrap();
        debug!("Analyzing topic {}", name);

        // Collect all of the triggers we're going to worry about, including triggers
        // belonging to an included or inherited topic.
        let all_triggers = inheritance::get_topic_triggers(brain, topic, false);

        // Sort these triggers.
        result.topics.insert(name.to_string(), sort_trigger_set(all_triggers.to_vec()));

        // Get all of the %Previous triggers.
        let that_triggers = inheritance::get_topic_triggers(brain, topic, true);

        // And sort them, too.
        result.thats.insert(name.to_string(), sort_trigger_set(that_triggers));
    }

    // Sort the substitution lists.
    result.subs = sort_list(brain.subs.clone());
    result.person = sort_list(brain.person.clone());

    Ok(result)
}

/// Sort a group of triggers in an optimal sorting order.
fn sort_trigger_set(triggers: Vec<ast::Trigger>) -> Vec<ast::Trigger> {
    // The running sort buffer of triggers as we add them.
    let mut running: Vec<ast::Trigger> = Vec::new();

    // Create a priority map, of priority numbers -> their triggers (for {weight} tags)
    let mut prior: HashMap<isize, Vec<ast::Trigger>> = HashMap::new();

    // Map the incoming triggers into their priority buckets.
    for trigger in triggers {
        let mut weight: isize = 0;
        match rivescript_core::regex::WEIGHT.captures(trigger.trigger.as_str()) {
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

    // Sort the priority values by largest first.
    // let sorted_priorities: Vec<isize> = Vec::new();
    for p in prior.keys().sorted().rev() {
        warn!("Sorting triggers with weight: {}", p);

        // Some of these triggers may include an {inherits} tag, if they came
        // from a topic which inherits another topic. Lower inherits values mean
        // higher priority on the stack. Triggers that have NO inherits value at
        // all (which will default to -1) will be moved to the END of the stack
        // at the end (having the highest number/lowest priority).
        let mut inherits = -1;
        let mut highest_inherits = -1;

        // Loop through and categorize these triggers.
        let mut track: HashMap<isize, SortTracker> = HashMap::new();
        track.insert(inherits, SortTracker::new());

        // Loop over triggers in this priority group.
        for trig in prior.get(p).unwrap().to_vec() {
            let mut pattern = trig.trigger.clone();
            debug!("Looking at trigger: {pattern}");

            // See if the trigger has an {inherits} tag.
            match rivescript_core::regex::INHERITS.captures(pattern.as_str()) {
                Some(cap) => {
                    inherits = cap.get(1).unwrap().as_str().parse::<isize>().unwrap_or(-1);
                    if inherits > highest_inherits {
                        highest_inherits = inherits;
                    }
                    debug!("Trigger belongs to a topic that inherits other topics. Level={inherits}");

                    // Remove the {inherits} tag from the raw pattern.
                    pattern = rivescript_core::regex::INHERITS.replace_all(&pattern, "").to_string();
                },
                None => (),
            }

            // Upsert and get the sort track for this trigger's inherits value.
            let this_track = track.entry(inherits).or_insert_with(SortTracker::new);

            // Start inspecting the trigger's contents.
            if pattern.contains("_") {
                // Has an alphabetic wildcard '_'
                let wc = word_count(&pattern, false);
                debug!("Has a _ wildcard and {wc} words");
                if wc > 0 {
                    let entries = this_track.alpha.entry(wc).or_insert_with(Vec::new);
                    entries.push(SortedTriggerEntry {
                        text: pattern,
                        pointer: trig,
                    });
                } else {
                    this_track.under.push(SortedTriggerEntry {
                        text: pattern,
                        pointer: trig,
                    });
                }
            } else if pattern.contains("#") {
                // Has a numeric wildcard '#'
                let wc = word_count(&pattern, false);
                debug!("Has a # wildcard and {wc} words");
                if wc > 0 {
                    let entries = this_track.number.entry(wc).or_insert_with(Vec::new);
                    entries.push(SortedTriggerEntry {
                        text: pattern,
                        pointer: trig,
                    });
                } else {
                    this_track.pound.push(SortedTriggerEntry {
                        text: pattern,
                        pointer: trig,
                    });
                }
            } else if pattern.contains("*") {
                // Has a wildcard '*'
                let wc = word_count(&pattern, false);
                debug!("Has a * wildcard and {wc} words");
                if wc > 0 {
                    let entries = this_track.wild.entry(wc).or_insert_with(Vec::new);
                    entries.push(SortedTriggerEntry {
                        text: pattern,
                        pointer: trig,
                    });
                } else {
                    this_track.star.push(SortedTriggerEntry {
                        text: pattern,
                        pointer: trig,
                    });
                }
            } else if pattern.contains("[") {
                // Has [optionals] included.
                let wc = word_count(&pattern, false);
                debug!("Has optionals with {wc} words");

                let entries = this_track.option.entry(wc).or_insert_with(Vec::new);
                entries.push(SortedTriggerEntry {
                    text: pattern,
                    pointer: trig,
                });
            } else {
                // Totally atomic.
                let wc = word_count(&pattern, false);
                debug!("Totally atomic trigger with {wc} words");

                let entries = this_track.atomic.entry(wc).or_insert_with(Vec::new);
                entries.push(SortedTriggerEntry{
                    text: pattern,
                    pointer: trig,
                });
            }

        }

        // Sort the track (inherits levels) from the lowest to the highest.
        let mut track_sorted: Vec<isize> = Vec::new();
        for k in track.keys() {
            track_sorted.push(k.clone());
        }
        track_sorted.sort();

        // Go through each priority level from greatest to smallest.
        for ip in track_sorted {
            // debug!("ip={ip} track={:?}", track);
            let ip_track = track.entry(ip).or_insert_with(SortTracker::new);

            // Sort each of the main kinds of triggers by their word counts.
            sort_by_words(&mut running, &ip_track.atomic);
            sort_by_words(&mut running, &ip_track.option);
            sort_by_words(&mut running, &ip_track.alpha);
            sort_by_words(&mut running, &ip_track.number);
            sort_by_words(&mut running, &ip_track.wild);

            // Add the single wildcard triggers, sorted by length.
            sort_by_length(&mut running, &ip_track.under);
            sort_by_length(&mut running, &ip_track.pound);
            sort_by_length(&mut running, &ip_track.star);
        }
    }

    running
}

#[derive(Debug)]
struct SortTracker {
    atomic: HashMap<isize, Vec<SortedTriggerEntry>>, // Sort atomic triggers by number of whole words
    option: HashMap<isize, Vec<SortedTriggerEntry>>, // Sort optionals by number of words
    alpha: HashMap<isize, Vec<SortedTriggerEntry>>,  // Sort alpha wildcards by no. of words
    number: HashMap<isize, Vec<SortedTriggerEntry>>, // Sort numeric wildcards by no. of words
    wild: HashMap<isize, Vec<SortedTriggerEntry>>,   // Sort wildcard triggers by no. of words
    pound: Vec<SortedTriggerEntry>,           // Triggers of just '#'
    under: Vec<SortedTriggerEntry>,           // Triggers of just '_'
    star: Vec<SortedTriggerEntry>,            // Triggers of just '*'
}

impl SortTracker {
    pub fn new() -> Self {
        Self {
            atomic: HashMap::new(),
            option: HashMap::new(),
            alpha: HashMap::new(),
            number: HashMap::new(),
            wild: HashMap::new(),
            pound: Vec::new(),
            under: Vec::new(),
            star: Vec::new(),
        }
    }
}

#[derive(Clone)]
#[derive(Debug)]
struct SortedTriggerEntry {
    text: String,
    pointer: ast::Trigger,
}

/// Count the number of real words in a string.
fn word_count(pattern: &str, all: bool) -> isize {
    let words: Vec<&str>;
    if all {
        words = pattern.split(' ').collect();
    } else {
        words = pattern.split(&[' ', '*', '#', '_', '|']).collect();
    }

    let mut wc = 0;
    for word in words {
        if word.len() > 0 {
            wc += 1;
        }
    }

    wc
}

/// Sort a list of strings (like substitutions) from a string:string map.
fn sort_list(dict: HashMap<String, String>) -> Vec<String> {

    // Group the list by number of words.
    let mut track: HashMap<isize, Vec<&String>> = HashMap::new();
    for phrase in dict.keys() {
        let wc = word_count(phrase, true);
        let entries = track.entry(wc).or_insert_with(Vec::new);
        entries.push(phrase);
    }

    // Sort them by word count, descending.
    let distinct_counts = track.keys().unique().sorted().rev();
    let mut sorted_patterns: Vec<String> = Vec::new();
    debug!("distinct_counts: {:?}", distinct_counts);

    for wc in distinct_counts {
        let entries = track.get(wc).unwrap();
        for entry in entries {
            sorted_patterns.push(entry.to_string());
        }
    }

    sorted_patterns.sort_by(|a, b| b.len().cmp(&a.len()));
    sorted_patterns
}

/// Sort a set of triggers by their word count and overall length.
///
/// This is a helper function for sorting the `atomic`, `option`, `alpha`, `number` and
/// `wild` attributes of the SortTrack and adding them to the running sort buffer in that
/// specific order.
///
/// The `triggers` parameter is a map of word counts to the triggers.
fn sort_by_words(running: &mut Vec<ast::Trigger>, triggers: &HashMap<isize, Vec<SortedTriggerEntry>>) {

    // debug!("sort_by_words: got {:?}", triggers);

    // Sort their word counts from greatest to least.
    let mut sorted_wc: Vec<isize> = Vec::new();
    for wc in triggers.keys() {
        sorted_wc.push(wc.clone());
    }
    sorted_wc.sort_by_key(|k| Reverse(*k));

    for wc in sorted_wc {

        // Triggers with equal word counts should be sorted by overall text length.
        let mut sorted_patterns: Vec<String> = Vec::new();
        let mut pattern_map: HashMap<String, Vec<&SortedTriggerEntry>> = HashMap::new();

        let entries = triggers.get(&wc).unwrap();
        for trig in entries {
            sorted_patterns.push(trig.text.clone());
            let entries = pattern_map.entry(trig.text.clone()).or_insert_with(Vec::new);
            entries.push(trig);
        }

        sorted_patterns.sort_by(|a, b| b.len().cmp(&a.len()));

        // Add the triggers to the running bucket.
        let mut distinct_pattern: HashMap<String, bool> = HashMap::new();
        for pattern in sorted_patterns {
            // Ensure unique patterns.
            if distinct_pattern.contains_key(&pattern) {
                continue;
            }
            distinct_pattern.insert(pattern.clone(), true);

            let entries = pattern_map.get(&pattern).unwrap();
            for entry in entries {
                debug!("sort_by_words: wc={wc} pattern={pattern}");
                running.push(entry.pointer.clone());
            }
        }
    }

}

/// Sort a set of triggers purely by character length.
///
/// This is like sort_by_words, but it's intended for triggers that consist solely of
/// wildcard-like symbols with no real words. For example a trigger of `* * * ` qualifies
/// for this, and it has no words, so we sort by length so it gets a priority higher
/// than the simple `*` trigger.
fn sort_by_length(running: &mut Vec<ast::Trigger>, triggers: &Vec<SortedTriggerEntry>) {
    let mut sorted_patterns: Vec<String> = Vec::new();
    let mut pattern_map: HashMap<String, Vec<SortedTriggerEntry>> = HashMap::new();

    for trig in triggers {
        sorted_patterns.push(trig.text.clone());
        let entries = pattern_map.entry(trig.text.clone()).or_insert_with(Vec::new);
        entries.push(trig.clone());
    }

    sorted_patterns.sort_by(|a, b| b.len().cmp(&a.len()));

    for pattern in sorted_patterns {
        let entries = pattern_map.get(&pattern).unwrap();
        for entry in entries {
            running.push(entry.pointer.clone());
        }
    }
}