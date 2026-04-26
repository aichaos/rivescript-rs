

pub trait LanguageLoader: Send + Sync {
    fn load(&mut self, name: &str, code: Vec<String>) -> Result<bool, String>;
    fn call(&self, name: &str, args: Vec<String>) -> Result<String, String>;
}