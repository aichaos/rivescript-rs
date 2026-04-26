use std::error::Error;
use std::fmt;

/// ParseError represents a fatal error during the parsing and sorting phase.
#[derive(Debug)]
pub struct ParseError {
    message: String,
}

impl ParseError {
    pub fn new(msg: &str) -> ParseError {
        ParseError {
            message: msg.to_string(),
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "parse error: {}", self.message)
    }
}

impl Error for ParseError {
    fn description(&self) -> &str {
        &self.message
    }
}
