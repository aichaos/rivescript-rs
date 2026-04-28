use std::{collections::HashMap, error::Error};
use rivescript::RiveScript;
use serde;

// TestCase wraps each RiveScript test.
struct TestCase {
    filename: String,
    name: String,
    username: String,
    rs: RiveScript,
    steps: Vec<TestStep>,
}

impl TestCase {

    /// Initialize a new TestCase from the YAML schema.
    pub fn new(filename: &str, name: &str, schema: TestSchema) -> Self {
        // Get optional values and defaults.
        let username = schema.username.unwrap_or("localuser".to_string());
        let utf8 = schema.utf8.unwrap_or(false);

        let mut bot = RiveScript::new();
        bot.utf8 = utf8;

        Self {
            filename: filename.to_string(),
            name: name.to_string(),
            username: username.to_string(),
            rs: bot,
            steps: schema.tests.unwrap(),
        }
    }

    /// Run all of the steps of the test case.
    pub async fn run(&mut self) {

        // Check if this test has any errors and collect their details.
        let mut has_errors = false;
        let mut errors: Vec<String> = Vec::new();

        let mut i: usize = 0;
        for step in self.steps.clone() {
            i += 1;
            let mut error = String::new();

            // What sort of command is this step doing?
            if let Some(code) = step.source {
                // Adding new RiveScript sources?
                match self.source(code) {
                    Ok(_) => {
                        self.rs.sort_triggers();
                    },
                    Err(e) => {
                        has_errors = true;
                        error = e.to_string();
                    }
                };
            } else if let Some(input) = step.input {
                // Testing an input/reply?
                let reply = step.reply.unwrap_or(MultiReply::Single("".to_string()));
                match self.input(input, reply).await {
                    Ok(_) => (),
                    Err(e) => {
                        has_errors = true;
                        error = e;
                    },
                };
            } else if let Some(set) = step.set {
                self.set(set).await;
            } else if let Some(assert) = step.assert {
                match self.assert(assert).await {
                    Ok(_) => (),
                    Err(e) => {
                        has_errors = true;
                        error = e;
                    },
                };
            }

            if !error.is_empty() {
                errors.push(format!("Step {i}: {error}"));
            }

        }

        // Report errors at the end.
        let mut symbol = String::from("✓");
        if has_errors {
            symbol = String::from("×");
        }
        println!("{} {}#{}", symbol, self.filename, self.name);
        if has_errors {
            for error in errors {
                eprintln!("{error}");
            }
        }
    }

    // Handle `source` actions.
    // This streams RiveScript source code into the current bot.
    fn source(&mut self, code: String) -> Result<bool, Box<dyn Error>> {
        self.rs.stream(code)
    }

    // Handle `input` (and `reply`) actions.
    async fn input(&mut self, message: String, expected: MultiReply) -> Result<bool, String> {
        match self.rs.reply(&self.username, &message).await {
            Ok(reply) => {

                // Expecting a single reply or checking random replies?
                match expected {
                    MultiReply::Single(single) => {
                        let single = single.trim();
                        if reply != single {
                            return Err(
                                format!("With input '{message}', expected: '{single}', got: '{reply}'").to_string(),
                            );
                        }
                    },
                    MultiReply::Multiple(options) => {
                        let mut matched = false;
                        for option in options {
                            let option = option.trim();
                            if reply == option {
                                matched = true;
                                break;
                            }
                        }

                        if !matched {
                            return Err(
                                format!("Didn't get any of the expected random replies (input: {message}), got: {reply}").to_string(),
                            );
                        }
                    },
                }

            },
            Err(e) => {
                eprintln!("Error: {e}");
            }
        }
        Ok(true)
    }

    // Handle `set` actions to set user variables.
    async fn set(&mut self, values: HashMap<String, String>) {
        self.rs.set_uservars(&self.username, values).await
    }

    // Handle `assert` actions to get user variables from the backend and match their values.
    async fn assert(&mut self, values: HashMap<String, String>) -> Result<bool, String> {
        for (key, expect) in values {
            let actual = self.rs.get_uservar(&self.username, &key).await;
            if actual != expect {
                return Err(
                    format!("User variable '{key}', expected: '{expect}', got: '{actual}'"),
                );
            }
        }
        Ok(true)
    }
}

// TestSchema loads the contents from YAML.
// The top-level YAML structure is a Map<String, TestSchema>.
#[derive(serde::Deserialize, Debug)]
struct TestSchema {
    username: Option<String>,
    utf8: Option<bool>,
    tests: Option<Vec<TestStep>>,
}

#[derive(serde::Deserialize, Debug, Clone)]
struct TestStep {
    source: Option<String>,
    input: Option<String>,
    reply: Option<MultiReply>,
    assert: Option<HashMap<String, String>>,
    set: Option<HashMap<String, String>>,
}

// In the RSTS YAML schema, the TestStep.reply field can be either a
// String (for single input/reply tests) or a Vec<String> (for random
// reply tests).
#[derive(serde::Deserialize, Debug, Clone)]
#[serde(untagged)]
enum MultiReply {
    Single(String),
    Multiple(Vec<String>),
}

#[tokio::test]
async fn test_rivescript_suite() {
    let test_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/rsts/tests");

    for entry in walkdir::WalkDir::new(test_dir) {
        let entry = entry.unwrap();
        if entry.path().extension().and_then(|s| s.to_str()) == Some("yml") {
            let filename = entry.file_name().to_str().unwrap();
            // let filename = entry.path().to_str().unwrap();
            println!("Loading test: {}", filename);
            let content = std::fs::read_to_string(entry.path()).unwrap();
            let suite: HashMap<String, TestSchema> = serde_yaml::from_str(&content).expect("Failed to parse yaml");

            for (test_name, schema) in suite {
                // Initialize unfilled defaults.
                let mut case = TestCase::new(filename, &test_name, schema);
                case.run().await;
            }

            println!("");

            // break;
        }
    }
}