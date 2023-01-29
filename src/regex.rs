use regex::Regex;
use lazy_static::lazy_static;

lazy_static! {
    /// {weight=100}
    pub static ref WEIGHT: Regex = Regex::new(r"\{weight=(\d+?)\}").unwrap();
}