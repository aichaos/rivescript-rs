use crate::ast::AST;
use log::debug;

pub struct Parser {}

impl Parser {
    pub fn new() -> Self {
        debug!("rivescript::parser initialized!");
        Self {}
    }

    pub fn parse(&self, filename: &str, contents: String) {
        debug!("parse() called on: {}", filename);

        // Start building an AST parsed from these files.
        let mut ast = AST::new();

        let mut lines = contents.lines();
        loop {
            let line = match lines.next() {
                Some(line) => line,
                None => break,
            };
            debug!("\tline: {}", line);
        }
    }
}
