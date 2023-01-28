// Data sorting logic.

use crate::ast;
use std::collections::HashMap;

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

