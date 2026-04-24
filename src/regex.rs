use regex::Regex;
use lazy_static::lazy_static;

lazy_static! {
    /// {weight=100}
    pub static ref WEIGHT: Regex = Regex::new(r"\{weight=(\d+?)\}").unwrap();
    pub static ref INHERITS: Regex = Regex::new(r"\{inherits=(\d+)\}").unwrap();
    pub static ref TRIGGER_OPTIONALS: Regex = Regex::new(r"\[(.+?)\]").unwrap();
    pub static ref TRIGGER_ARRAY: Regex = Regex::new(r"@(.+?)\b").unwrap();
    pub static ref REPLY_ARRAY: Regex = Regex::new(r"\(@([A-Za-z0-9_]+)\)").unwrap();
    pub static ref RANDOM_TAG: Regex = Regex::new(r"\{random\}(.+?)\{/random\}").unwrap();
    pub static ref PERSON_TAG: Regex = Regex::new(r"\{person\}(.+?)\{/person\}").unwrap();
    pub static ref FORMAL_TAG: Regex = Regex::new(r"\{formal\}(.+?)\{/formal\}").unwrap();
    pub static ref SENTENCE_TAG: Regex = Regex::new(r"\{sentence\}(.+?)\{/sentence\}").unwrap();
    pub static ref UPPERCASE_TAG: Regex = Regex::new(r"\{uppercase\}(.+?)\{/uppercase\}").unwrap();
    pub static ref LOWERCASE_TAG: Regex = Regex::new(r"\{lowercase\}(.+?)\{/lowercase\}").unwrap();
    pub static ref TOPIC_TAG: Regex = Regex::new(r"\{topic=(.+?)\}").unwrap();
    pub static ref REDIRECT_TAG: Regex = Regex::new(r"\{@(.+?)\}").unwrap();
    pub static ref ANY_TAG: Regex = Regex::new(r"<([^<]+?)>").unwrap();
    pub static ref BOT_TAG: Regex = Regex::new(r"<bot (.+?)>").unwrap();
    pub static ref USER_VAR_TAG: Regex = Regex::new(r"<get (.+?)>").unwrap();
    pub static ref HISTORY_TAG: Regex = Regex::new(r"<(?:input|reply)(\d+?)>").unwrap();
    pub static ref NASTIES: Regex = Regex::new(r"[^A-Za-z0-9 ]").unwrap();
    pub static ref ZERO_WIDTH_STAR: Regex = Regex::new(r"^\*$").unwrap();
    pub static ref CONDITION: Regex = Regex::new(r"^(.+?)\s+(==|eq|!=|ne|<>|<|<=|>|>=)\s+(.*?)$").unwrap();
    pub static ref PLACEHOLDER: Regex = Regex::new(r"\x00(\d+)\x00").unwrap();
}