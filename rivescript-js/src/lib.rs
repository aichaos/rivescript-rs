use std::collections::HashMap;
use async_trait::async_trait;
use boa_engine::{Context, JsObject, JsResult, JsString, JsValue, Source, context::intrinsics::Intrinsics, native_function::NativeFunction};

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

    async fn call(&self, proxy: &dyn Proxy, name: &str, args: Vec<String>) -> Result<SubroutineResult, String> {
        let code = self.sources.get(name)
            .ok_or_else(|| format!("[ERR: Object '{}' Not Found]", name))?;

        // Initialize Boa context.
        let mut context = Context::default();
        let global = context.global_object();

        // Set up the RiveScript "bridge" object.
        let proxy_mutex = std::sync::Arc::new(tokio::sync::Mutex::new(proxy));
        let rs_obj = JsObject::with_null_proto();

        // Define the rs.currentUser() -> proxy.current_username() bridge.
        // let p_clone = proxy_mutex.clone();
        // let current_user_fn = NativeFunction::from_copy_closure(move |_this, _args, _context| {
        //     let mut p = p_clone.try_lock()
        //         .map_err(|_| boa_engine::JsNativeError::error().with_message("Proxy lock contention"))?;
        //     let name = p.current_username().unwrap_or_else(|_| rivescript_core::UNDEFINED.to_string());
        //     Ok(JsValue::from(JsString::from(name)))
        // });

        // rs_obj.set(
        //         JsString::from("currentUser"),
        //         context.create_native_function(
        //             JsString::from("currentUser"),
        //             current_user_fn,
        //         ),
        //         false,
        //         &mut context
        //     )
        //     .map_err(|e| format!("JS Setup Error: {}", e))?;

        // Turn it into a JavaScript function and evaluate it.
        let js_src = format!("function objectMacro (rs, args) {{ {} }}", code);
        println!("EVAL: {}", js_src);
        context.eval(Source::from_bytes(js_src.as_bytes()))
            .map_err(|e| format!("JS Eval Error: {}", e))?;

        // Prepare arguments for the JS function call.
        let js_args = boa_engine::object::builtins::JsArray::from_iter(
            args.iter().map(|s| JsValue::from(JsString::from(s.as_str()))),
            &mut context,
        );
        let dispatch_args = [
            rs_obj.into(),
            js_args.into(),
        ];
        println!("JS args: {:#?}", dispatch_args);

        // Call the function.
        let func = global.get(JsString::from("objectMacro"), &mut context)
            .map_err(|_| format!("JS Function '{}' not defined in object", name))?;

        if let Some(callable) = func.as_callable() {
            let result = callable.call(&JsValue::undefined(), &dispatch_args, &mut context)
                .map_err(|e| format!("JS Execution Error: {}", e))?;

            // Return the output.
            Ok(SubroutineResult{
                output: result.to_string(&mut context)
                    .map(|js_str| js_str.to_std_string_escaped())
                    .unwrap_or_default(),
                staged_bot_vars: HashMap::new(),
                staged_user_vars: HashMap::new(),
            })
        } else {
            Err(format!("Object '{}' is not a callable function in JS", name))
        }
    }
}