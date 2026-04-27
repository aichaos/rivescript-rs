use rivescript_core::macros::LanguageLoader;
use boa_engine::{Context, JsArgs, JsValue, Source, context::ContextBuilder, job::SimpleJobQueue};
use std::sync::{Arc, Mutex};

pub struct JavaScriptLoader {
    // Arc allows multiple references to the loader;
    // Mutex ensures only one thread uses the JS VM at a time.
    context: Arc<Mutex<Context>>,
}

impl JavaScriptLoader {
    pub fn new() -> Self {
        let context = ContextBuilder::new()
            .job_queue(SimpleJobQueue::new())
            .build()
            .expect("Failed to build Boa context");
        Self {
            context: Arc::new(Mutex::new(context)),
        }
    }
}

impl LanguageLoader for JavaScriptLoader {
    fn load(&mut self, name: &str, code: Vec<String>) -> Result<bool, String> {
        // Lock the VM for this operation
        let mut context = self.context.lock()
            .map_err(|_| "Failed to acquire JS context lock (poisoned)")?;

        let body = code.join("\n");
        let js_code = format!(
            "function object_{}(rs, args) {{ \n {} \n }}",
            name, body
        );

        context.eval(Source::from_bytes(js_code.as_bytes()))
            .map_err(|e| format!("JS Load Error in '{}': {}", name, e))?;

        Ok(true)
    }

    fn call(&self, name: &str, args: Vec<String>) -> Result<String, String> {
        let mut context = self.context.lock()
            .map_err(|_| "Failed to acquire JS context lock (poisoned)")?;

        let func_name = format!("object_{}", name);

        // 1. Get the function from the global object
        let global = context.global_object();
        let func = global.get(func_name.as_str(), &mut context)
            .map_err(|e| format!("JS Lookup Error: {}", e))?;

        if !func.is_callable() {
            return Err(format!("JavaScript function '{}' is not defined", func_name));
        }

        // 2. Map Rust Strings to Boa JsValues
        let js_args: Vec<JsValue> = args.into_iter()
            .map(|s| JsValue::from(s))
            .collect();

        // Convert the Vec into a JS Array object
        let args_array = boa_engine::object::builtins::JsArray::from_iter(js_args, &mut context);

        // 3. Execute the function
        // We pass 'undefined' as the 'this' context.
        let result = func.as_callable()
            .unwrap()
            .call(
                &JsValue::undefined(),
                &[JsValue::null(), args_array.into()],
                &mut context
            );

        // 4. Handle the return value
        match result {
            Ok(value) => {
                // Try to convert the result to a string for RiveScript
                let js_string = value.to_string(&mut context)
                    .map_err(|e| format!("Result conversion error: {}", e))?;

                Ok(js_string.to_std_string_escaped())
            }
            Err(e) => Err(format!("JS Runtime Error in '{}': {}", name, e)),
        }
    }
}