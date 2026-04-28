//! # RiveScript Abstract Syntax Tree
//!
//! The AST is a parsed object representing all of the contents of a RiveScript
//! bot brain. It is the output object that you get when you load_file() or
//! stream() a RiveScript document (or several). It contains all of the useful
//! inner contents of the RiveScript brain, including global bot variables,
//! substitutions and other configuration that was defined via RiveScript.

use std::{collections::HashMap, sync::RwLock};

/// Root of the "abstract syntax tree" representing a RiveScript
/// source document and its useful contents.
#[derive(Debug)]
pub struct AST {
    // Configuration fields typically found in 'begin.rive'
    pub version: f32,                         // ! version
    pub globals: RwLock<HashMap<String, String>>,     // ! global
    pub vars: RwLock<HashMap<String, String>>,        // ! var
    pub subs: HashMap<String, String>,        // ! sub stitutions
    pub person: HashMap<String, String>,      // ! person substitutions
    pub arrays: HashMap<String, Vec<String>>, // ! array sets

    // Topics and their triggers.
    pub topics: HashMap<String, Topic>,

    // Parsed object macros.
    pub objects: HashMap<String, Object>,
}

/// Topic is a group of triggers.
///
/// All triggers belong to a topic, with the default topic being
/// a special one named "random". To move the current user into a
/// different topic, use the `{topic}` tag, for example
/// `{topic=random}`. A user can ONLY match triggers that are defined
/// in their current topic, or any triggers that are 'included' or
/// 'inherited' into their current topic.
#[derive(Debug, Clone)]
pub struct Topic {
    pub name: String,
    pub triggers: Vec<Trigger>,
    pub includes: HashMap<String, bool>,
    pub inherits: HashMap<String, bool>,
}

impl AST {
    /// Initialize a new AST object ready for populating.
    pub fn new() -> Self {
        Self {
            version: 0.0,
            globals: RwLock::new(HashMap::new()),
            vars: RwLock::new(HashMap::new()),
            subs: HashMap::new(),
            person: HashMap::new(),
            arrays: HashMap::new(),
            topics: HashMap::new(),
            objects: HashMap::new(),
        }
    }

    /// Merge the current AST with the contents of the other.
    ///
    /// This is used during parsing when e.g. loading a whole directory of files.
    /// All parsed files add to the loaded AST for the root RiveScript instance.
    pub fn extend(&mut self, other: AST) {
        if other.version != 0.0 {
            self.version = other.version;
        }

        let mut global_guard = self.globals.write().expect("RwLock poisoned");
        let mut other_globals = other.globals.write().expect("RwLock poisoned");
        global_guard.extend(other_globals.drain());

        let mut vars_guard = self.vars.write().expect("RwLock poisoned");
        let mut other_vars = other.vars.write().expect("RwLock poisoned");
        vars_guard.extend(other_vars.drain());

        self.subs.extend(other.subs.into_iter());
        self.person.extend(other.person.into_iter());
        self.arrays.extend(other.arrays.into_iter());
        // self.topics.extend(other.topics.into_iter());
        self.objects.extend(other.objects.into_iter());

        // Merge topics more carefully.
        for (name, topic) in other.topics {
            match self.topics.get_mut(&name) {
                Some(mine) => {
                    mine.triggers.extend(topic.triggers);
                }
                None => {
                    self.topics.insert(name, topic);
                }
            }
        }
    }

    /// Initialize the data structure for a new topic, if it wasn't already there.
    pub fn init_topic(&mut self, name: &String) {
        if self.topics.contains_key(name) {
            return;
        }

        self.topics.insert(
            name.to_string(),
            Topic {
                name: name.to_string(),
                triggers: Vec::new(),
                includes: HashMap::new(),
                inherits: HashMap::new(),
            },
        );
    }

    /// Returns true if a >begin section exists.
    pub fn has_begin_block(&self) -> bool {
        self.topics.contains_key(crate::BEGIN_TOPIC)
    }

    /// Returns true if the topic exists.
    pub fn has_topic(&self, name: &str) -> bool {
        self.topics.contains_key(name)
    }

    /// Get a global variable. Returns "undefined" if not set.
    pub fn get_global(&self, name: &str) -> String {
        let globals_guard = self.globals.read().expect("RwLock poisoned");
        if let Some(value) = globals_guard.get(name) {
            return value.to_string();
        }
        crate::UNDEFINED.to_string()
    }

    /// Set a global variable.
    pub fn set_global(&self, name: &str, value: &str) {
        let mut globals_guard = self.globals.write().expect("RwLock poisoned");
        globals_guard.insert(name.to_string(), value.to_string());
    }

    /// Get a bot variable. Returns "undefined" if not set.
    pub fn get_bot_var(&self, name: &str) -> String {
        let vars_guard = self.vars.read().expect("RwLock poisoned");
        if let Some(value) = vars_guard.get(name) {
            return value.to_string();
        }
        crate::UNDEFINED.to_string()
    }

    /// Set a bot variable.
    pub fn set_bot_var(&self, name: &str, value: &str) {
        let mut vars_guard = self.vars.write().expect("RwLock poisoned");
        vars_guard.insert(name.to_string(), value.to_string());
    }
}

impl Topic {
    pub fn set_includes(&mut self, includes: String) {
        self.includes.insert(includes.to_string(), true);
    }

    pub fn set_inherits(&mut self, inherits: String) {
        self.inherits.insert(inherits.to_string(), true);
    }

    pub fn add_trigger(&mut self, trigger: Trigger) {
        self.triggers.push(trigger);
    }
}

/// Trigger represents a pattern that matches a user's message.
///
/// It is the base unit of intelligence for your chatbot. A trigger
/// of "hello bot" will match when the user says that phrase, and can
/// pair a set of replies (multiple OK, which will be chosen at random)
/// to be sent when that trigger is matched.
#[derive(Debug, Clone)]
pub struct Trigger {
    pub trigger: String,
    pub reply: Vec<String>,
    pub condition: Vec<Condition>,
    pub redirect: String,
    pub previous: String,
}

impl Trigger {
    pub fn new(trigger: &str) -> Self {
        Self {
            trigger: trigger.to_string(),
            reply: Vec::new(),
            condition: Vec::new(),
            redirect: String::from(""),
            previous: String::from(""),
        }
    }

    pub fn is_populated(&self) -> bool {
        self.trigger.len() > 0
    }
}

/// Condition holds the contents of a *Condition command in RiveScript.
#[derive(Debug, Clone)]
pub struct Condition {
    pub left: String,
    pub operator: String,
    pub right: String,
    pub reply: String,
}

/// Object represents a parsed object macro from a RiveScript source document.
///
/// Object macros have a name, a programming language, and an array of their
/// source code as defined in the RiveScript document. It is up to the
/// interpreter program to understand how to parse an object macro and make
/// it executable.
#[derive(Debug, Clone)]
pub struct Object {
    pub name: String,
    pub language: String,
    pub code: Vec<String>,
}

impl Object {
    pub fn new(name: &str, language: &str, code: Vec<String>) -> Self {
        Self {
            name: name.to_string(),
            language: language.to_string(),
            code,
        }
    }
}
