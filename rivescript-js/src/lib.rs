use std::collections::HashMap;
use async_trait::async_trait;

use rivescript_core::macros::{LanguageLoader, Proxy, SubroutineResult};

pub struct JavaScriptLoader {
    sources: HashMap<String, String>,
}

impl JavaScriptLoader {
    pub fn new() -> Self {
        Self{
            sources: HashMap::new(),
        }
    }
}

#[async_trait]
impl LanguageLoader for JavaScriptLoader {
    fn load(&mut self, name: &str, code: Vec<String>) -> Result<bool, String> {
        self.sources.insert(name.to_string(), code.join("\n"));
        Ok(true)
    }
    async fn call(&self, _proxy: &dyn Proxy, name: &str, _args: Vec<String>) -> Result<SubroutineResult, String> {
        match self.sources.get(name) {
            Some(code) => return Ok(SubroutineResult{
                    output: code.to_string(),
                    staged_bot_vars: HashMap::new(),
                    staged_user_vars: HashMap::new(),
            }),
            None => {
                return Err(format!("[ERR: Object '{}' Not Found]", name));
            }
        }
    }
}