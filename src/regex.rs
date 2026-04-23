use regex::Regex;
use lazy_static::lazy_static;

lazy_static! {
    /// {weight=100}
    pub static ref WEIGHT: Regex = Regex::new(r"\{weight=(\d+?)\}").unwrap();
    pub static ref INHERITS: Regex = Regex::new(r"\{inherits=(\d+)\}").unwrap();
    pub static ref NASTIES: Regex = Regex::new(r"[^A-Za-z0-9 ]").unwrap();
    pub static ref ZERO_WIDTH_STAR: Regex = Regex::new(r"^\*$").unwrap();
}