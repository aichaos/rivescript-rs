use std::collections::HashMap;

/// Root of the "abstract syntax tree" representing a RiveScript
/// source document and its useful contents.
#[derive(Debug)]
pub struct AST {
    // Configuration fields typically found in 'begin.rive'
    pub version: f32,                         // ! version
    pub globals: HashMap<String, String>,     // ! global
    pub vars: HashMap<String, String>,        // ! var
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
#[derive(Debug)]
pub struct Topic {
    pub triggers: Vec<Trigger>,
    pub includes: HashMap<String, bool>,
    pub inherits: HashMap<String, bool>,
}

impl AST {
    pub fn new() -> Self {
        Self {
            version: 0.0,
            globals: HashMap::new(),
            vars: HashMap::new(),
            subs: HashMap::new(),
            person: HashMap::new(),
            arrays: HashMap::new(),
            topics: HashMap::new(),
            objects: HashMap::new(),
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
                triggers: Vec::new(),
                includes: HashMap::new(),
                inherits: HashMap::new(),
            },
        );
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
#[derive(Debug)]
pub struct Trigger {
    pub trigger: String,
    pub reply: Vec<String>,
    pub condition: Vec<String>, // TODO: richer formatted
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

/// Object represents a parsed object macro from a RiveScript source document.
///
/// Object macros have a name, a programming language, and an array of their
/// source code as defined in the RiveScript document. It is up to the
/// interpreter program to understand how to parse an object macro and make
/// it executable.
#[derive(Debug)]
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
