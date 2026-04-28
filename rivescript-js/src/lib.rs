use rivescript_core::macros::LanguageLoader;

pub struct JavaScriptLoader {

}

impl JavaScriptLoader {
    pub fn new() -> Self {
        Self{}
    }
}

impl LanguageLoader for JavaScriptLoader {
    fn load(&mut self, _name: &str, _code: Vec<String>) -> Result<bool, String> {
        Ok(true)
    }
    fn call(&self, _name: &str, _args: Vec<String>) -> Result<String, String> {
        Ok("".to_string())
    }
}