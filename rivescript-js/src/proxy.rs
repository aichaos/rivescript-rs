use std::{collections::HashMap};
use boa_engine::{
    Context, JsArgs, JsData, JsError, JsNativeError, JsResult, JsValue, NativeFunction, class::{Class, ClassBuilder}
};
use boa_gc::{Finalize, GcRefCell, Trace};


/// This struct holds a "snapshot" of data from the RiveScript Proxy.
///
/// Since the RiveScript Proxy has async functions (to support user variable
/// session adapters that are async), and Rust Futures don't get along well
/// with the JavaScript context in boa, this JavaScriptProxy will first gather
/// all of the user variables for the current user and pre-populate them to
/// avoid the need for async function calls.
///
/// Additionally, this proxy will expose a JavaScript-like API with camel
/// cased function names like `getUservar()` which will be consistent with
/// the JavaScript port of RiveScript.
#[derive(Debug, Trace, Finalize, JsData)]
pub struct JavaScriptProxy {
    current_username: String,

    // Current (read-only) variables.
    user_vars: HashMap<String, String>,
    bot_vars: HashMap<String, String>,

    // Staged (written) variables.
    staged_user_vars: GcRefCell<HashMap<String, String>>,
    staged_bot_vars: GcRefCell<HashMap<String, String>>,
}

impl JavaScriptProxy {

    /// Construct the JavaScriptProxy from the RiveScript Proxy.
    ///
    /// This will pre-load user and bot variables to make them easily accessible
    /// to the JavaScript context without the need for Rust async function calls
    /// as would normally be required from the RiveScript Proxy.
    pub fn new(username: &str, user_vars: HashMap<String, String>) -> Self {
        // let username = proxy.current_username();
        // // let user_vars = proxy.get_uservars(&username).await;
        // let user_vars = HashMap::new();
        Self {
            current_username: username.to_string(),
            user_vars: user_vars.clone(),
            bot_vars: HashMap::new(),
            staged_user_vars: GcRefCell::new(HashMap::new()),
            staged_bot_vars: GcRefCell::new(HashMap::new()),
        }
    }

    // Retrieve the staged (written) user variables.
    pub fn get_staged_uservars(&self) -> HashMap<String, String> {
        let hm = self.staged_user_vars.borrow();
        return hm.clone()
    }

    // Retrieve the staged (written) bot variables.
    pub fn get_staged_botvars(&self) -> HashMap<String, String> {
        let hm = self.staged_bot_vars.borrow();
        return hm.clone()
    }

}

impl Class for JavaScriptProxy {
    const NAME: &'static str = "RiveScriptProxy";

    fn init(class: &mut ClassBuilder) -> JsResult<()> {

        // Bind public API functions.
        class
            .method(boa_engine::js_string!("currentUser"), 0, NativeFunction::from_fn_ptr(Self::current_user))
            .method(boa_engine::js_string!("getUservar"), 0, NativeFunction::from_fn_ptr(Self::get_uservar))
            .method(boa_engine::js_string!("setUservar"), 0, NativeFunction::from_fn_ptr(Self::set_uservar))
            .method(boa_engine::js_string!("getVariable"), 0, NativeFunction::from_fn_ptr(Self::get_variable))
            .method(boa_engine::js_string!("setVariable"), 0, NativeFunction::from_fn_ptr(Self::set_variable));
        Ok(())
    }

    /// This function is needed to satisfy the trait, and would be called if somebody
    /// did `new RiveScriptProxy()` in JS. In reality, we construct the object from the
    /// JavaScriptHandler to populate its inner contents. This function will return an
    /// empty version of JavaScriptProxy.
    fn data_constructor(_: &JsValue, _: &[JsValue], _: &mut boa_engine::Context) -> Result<Self, JsError> {
        Ok(Self{
            current_username: String::new(),
            user_vars: HashMap::new(),
            bot_vars: HashMap::new(),
            staged_user_vars: GcRefCell::new(HashMap::new()),
            staged_bot_vars: GcRefCell::new(HashMap::new()),
        })
    }
}

#[allow(non_snake_case)]
impl JavaScriptProxy {

    /// rs.currentUser(): String
    fn current_user(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not an object")
        })?;

        let rs = obj.downcast_ref::<Self>().ok_or_else(|| {
            JsNativeError::typ().with_message("Invalid 'this' binding: expected RiveScriptProxy")
        })?;

        Ok(JsValue::from(boa_engine::js_string!(rs.current_username.clone())))
    }

    /// rs.getUservar(username, name): String
    fn get_uservar(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not an object")
        })?;

        let rs = obj.downcast_ref::<Self>().ok_or_else(|| {
            JsNativeError::typ().with_message("Invalid 'this' binding: expected RiveScriptProxy")
        })?;

        let name = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();

        // Check the staged data first in case we recently wrote this var.
        if let Some(val) = rs.staged_user_vars.borrow().get(&name) {
            return Ok(JsValue::from(boa_engine::js_string!(val.clone())));
        }

        let val = rs.user_vars.get(&name).cloned().unwrap_or_default();
        Ok(JsValue::from(boa_engine::js_string!(val)))
    }

    /// rs.setUservar(username, name, value)
    fn set_uservar(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not an object")
        })?;

        let rs = obj.downcast_ref::<Self>().ok_or_else(|| {
            JsNativeError::typ().with_message("Invalid 'this' binding: expected RiveScriptProxy")
        })?;

        let username = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();
        let name = args.get_or_undefined(1).to_string(context)?.to_std_string_escaped();
        let val = args.get_or_undefined(2).to_string(context)?.to_std_string_escaped();

        if username == rs.current_username {
            rs.staged_user_vars.borrow_mut().insert(name, val);
        }
        Ok(JsValue::undefined())
    }

    /// rs.getVariable(name): String
    fn get_variable(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not an object")
        })?;

        let rs = obj.downcast_ref::<Self>().ok_or_else(|| {
            JsNativeError::typ().with_message("Invalid 'this' binding: expected RiveScriptProxy")
        })?;

        let name = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();

        // Check the staged data first in case we recently wrote this var.
        if let Some(val) = rs.staged_bot_vars.borrow().get(&name) {
            return Ok(JsValue::from(boa_engine::js_string!(val.clone())));
        }

        let val = rs.bot_vars.get(&name).cloned().unwrap_or_default();
        Ok(JsValue::from(boa_engine::js_string!(val)))
    }

    /// rs.setVariable(name, value)
    fn set_variable(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not an object")
        })?;

        let rs = obj.downcast_ref::<Self>().ok_or_else(|| {
            JsNativeError::typ().with_message("Invalid 'this' binding: expected RiveScriptProxy")
        })?;

        let name = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();
        let val = args.get_or_undefined(1).to_string(context)?.to_std_string_escaped();

        rs.staged_bot_vars.borrow_mut().insert(name, val);
        Ok(JsValue::undefined())
    }
}