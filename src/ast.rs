use std::collections::HashMap;

// Root of the "abstract syntax tree" representing a RiveScript
// source document and its useful contents.
pub struct AST {
    globals: HashMap<String, String>,
    vars: HashMap<String, String>,
    subs: HashMap<String, String>, // ! sub stitutions
    person: HashMap<String, String>, // ! person substitutions
                                   // arrays: *mut String, // ! array sets
}

impl AST {
    pub fn new() -> Self {
        Self {
            globals: HashMap::new(),
            vars: HashMap::new(),
            subs: HashMap::new(),
            person: HashMap::new(),
        }
    }
}
