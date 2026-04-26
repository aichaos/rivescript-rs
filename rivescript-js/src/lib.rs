use rivescript_core::macros::LanguageLoader;

pub struct JavaScriptLoader {

}

impl JavaScriptLoader {
    pub fn new() -> Self {
        Self{}
    }
}

impl LanguageLoader for JavaScriptLoader {
    fn load(&mut self, name: &str, code: Vec<String>) -> Result<bool, String> {
        Ok(true)
    }
    fn call(&self, name: &str, args: Vec<String>) -> Result<String, String> {
        Ok("".to_string())
    }
}