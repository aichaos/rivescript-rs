use std::collections::HashMap;
use async_trait::async_trait;
use boa_engine::{Context, JsObject, JsResult, JsString, JsValue, Source, context::intrinsics::Intrinsics, native_function::NativeFunction};

use rivescript_core::macros::{LanguageLoader, Proxy, SubroutineResult};

use crate::proxy::JavaScriptProxy;

mod proxy;

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

        // Create a JavaScriptProxy from the RiveScript Proxy.
        // This will glob all user variables and expose a JS-friendly API.
        let js_proxy = {
            let username = proxy.current_username();
            let user_vars = proxy.get_uservars(&username).await;
            JavaScriptProxy::new(&username, user_vars)
        };

        // Initialize Boa context.
        let mut context = Context::default();
        let global = context.global_object();

        // Register the JavaScriptProxy class globally in the JS context.
        context.register_global_class::<JavaScriptProxy>().map_err(|e| e.to_string())?;

        // Instantiate the JavaScriptProxy for the `rs` in `(rs, args)` function parameter.
        let constructor = global.get(boa_engine::js_string!("RiveScriptProxy"), &mut context)
            .map_err(|_| "Constructor not found")?
            .as_object()
            .ok_or_else(|| "Constructor is not an object")?;
        let prototype = constructor.get(boa_engine::js_string!("prototype"), &mut context)
            .map_err(|_| "Prototype property not found")?
            .as_object()
            .ok_or_else(|| "Prototype is not an object")?;
        let rs_instance = boa_engine::object::JsObject::from_proto_and_data(
            prototype,
            js_proxy,
        );

        // Evaluate the object macro source code as a JS function.
        let js_src = format!("function objectMacro (rs, args) {{ {} }}", code);
        context.eval(Source::from_bytes(js_src.as_bytes()))
            .map_err(|e| format!("JS Eval Error: {}", e))?;

        // Prepare arguments for the JS function call.
        let js_args = boa_engine::object::builtins::JsArray::from_iter(
            args.iter().map(|s| JsValue::from(JsString::from(s.as_str()))),
            &mut context,
        );
        let dispatch_args = [
            rs_instance.clone().into(),
            js_args.into(),
        ];

        // Get the Function object for the object macro.
        let func = global.get(JsString::from("objectMacro"), &mut context)
            .map_err(|_| format!("JS Function '{}' not defined in object", name))?;

        // Call it!
        if let Some(callable) = func.as_callable() {
            let mut result = callable.call(&JsValue::undefined(), &dispatch_args, &mut context)
                .map_err(|e| format!("JS Execution Error: {}", e))?;

            // If the function returned undefined (no return value), treat it as empty string.
            if result.is_undefined() {
                result = JsValue::from(boa_engine::js_string!(""));
            }

            // Harvest the (possibly changed) user variables back from the proxy.
            let final_user_vars = {
                let rs_borrow = rs_instance.downcast_ref::<JavaScriptProxy>()
                    .ok_or_else(|| "Failed to downcast JavaScriptProxy for harvesting".to_string())?;
                rs_borrow.get_staged_uservars()
            };
            let final_bot_vars = {
                let rs_borrow = rs_instance.downcast_ref::<JavaScriptProxy>()
                    .ok_or_else(|| "Failed to downcast JavaScriptProxy for harvesting".to_string())?;
                rs_borrow.get_staged_botvars()
            };

            // Return the output.
            Ok(SubroutineResult{
                output: result.to_string(&mut context)
                    .map(|js_str| js_str.to_std_string_escaped())
                    .unwrap_or_default(),
                staged_user_vars: final_user_vars,
                staged_bot_vars: final_bot_vars,
            })
        } else {
            Err(format!("Object '{}' is not a callable function in JS", name))
        }
    }
}