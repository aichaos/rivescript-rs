# JavaScript Handler for RiveScript

This crate implements a JavaScript language handler for RiveScript object macros.

It uses [boa](https://github.com/boa-dev/boa) for its JavaScript engine and can allow your RiveScript program to parse and run JavaScript object macros with a similar API to the [native JavaScript port of RiveScript](https://github.com/aichaos/rivescript-js).

## Example

To register the JavaScript language handler in your RiveScript bot:

```rust
use rivescript::RiveScript;
use rivescript_js::JavaScriptLoader;
use std::io;

#[tokio::main]
async fn main() {
    let mut bot = RiveScript::new();
    bot.set_session_manager(redis_manager);

    // Register the JavaScript handler.
    bot.set_handler("javascript", JavaScriptLoader::new());

    bot.load_directory("./eg/brain")
        .expect("Error loading replies!");
    bot.sort_triggers();

    // Enter main chat loop.
    loop {
        // Input
        print!("You> ");
        io::stdout()
            .flush()
            .expect("oops");
        let mut message = String::new();
        io::stdin()
            .read_line(&mut message)
            .expect("Failed to read line");

        // Reply
        match bot.reply("localuser", &message).await {
            Ok(reply) => {
                println!("Bot> {reply}");
            },
            Err(e) => {
                eprintln!("Error: {e}");
            }
        };

    }
}
```

In your RiveScript code, you can then define object macros written in JavaScript like so:

```rivescript
// <call>hello-js</call>
> object hello-js javascript
	const username = rs.currentUser();
	return `Hello, ${username}!`;
< object

// <call>setvar $key $value</call>
// Set a user variable for the current user.
// Equivalent to <set $key=$value>.
> object setvar javascript
	var name  = args.shift();
	var value = args.join(" ");

	var uid = rs.currentUser();
	rs.setUservar(uid, name, value);
< object

// RiveScript triggers that invoke these object macros.

+ hello js
- <call>hello-js</call>

+ js set * to *
- Setting your user variable '<star1>' to '<star2>'.
^ <call>setvar <star1> <star2></call>
```

## JavaScript API

Your object macros are converted into JavaScript functions which receive a RiveScript [Proxy](#javascriptproxy) and array of arguments that were passed in from the `<call>` tag:

```javascript
function objectMacro(rs, args) {
    // Your inner object macro body goes here.
}
```

Parameters are like:

* `rs: JavaScriptProxy`: the proxy exposes a subset of useful RiveScript functions that an object macro would commonly want to use, such as to get the current username and get/set variables for them.
* `args: Vec<String>`: the `<call>` tag uses "shell-style quoting" rules so that a "quoted string" will come in as a single element on the args array.

### JavaScriptProxy

In most implementations of RiveScript (in other programming languages), object macros typically receive a pointer to the master RiveScript struct as their first argument.

However, in the Rust port it is not possible to give a mutable borrow of RiveScript to object macro subroutines. Instead, a RiveScript Proxy is given which exposes a subset of useful functions from RiveScript.

The JavaScriptProxy exposes these functions and uses an API signature compatible with [the native JavaScript implementation of RiveScript](https://github.com/aichaos/rivescript-js).

The available proxy functions include:

* rs.currentUser() -> String

    Returns the current username that was passed in originally to the RiveScript.reply() function.
* rs.setUservar(username: String, name: String, value: String)

    Set a user variable for the current user, equivalent to the `<set name=value>` tag in RiveScript.

    **Important:** only variables for the _current user_ may be set. You will need to pass in the rs.currentUser() as the username parameter; if you pass a different username, nothing will happen and the variable will not be set for the other user.
* rs.getUservar(username: String, name: String) -> String

    Retrieve a user variable for the current user.

    **Note:** like setUservar, only the rs.currentUser() is supported by this function.

    If you had called setUservar and then getUservar, it will return the staged copy recently written to rather than the original value (if any) from the user variable session manager.
* rs.setVariable(name: String, value: String)

    Set a global bot variable to a new value, equivalent to the `<bot name=value>` tag in RiveScript. Bot variables are global to the RiveScript instance and so are shared between users.

    Bot variables that are written to will be resolved and committed back to the RiveScript bot's brain after the object macro returns.
* rs.getVariable(name: String)

    Get a bot variable from RiveScript.

    If you had recently setVariable and then you getVariable on the same name, it will return the staged copy that you had set before. After your object macro returns, any changed bot variables are resolved and committed back to the main RiveScript bot instance.

## Limitations

Compared to other RiveScript implementations, the functionality of the Proxy imposes some limitations.

* You can only Get and Set user variables for the Current User.

    In the native Rust Proxy (for object macro subroutines written in Rust), the getter functions pass directly along to your User Variable Session Manager (e.g. in-memory or Redis), so it _can_ get variables belonging to other users.

    But the Set function can **only** write for the current user. If you pass any name besides rs.currentUser(), nothing will be saved.

    This is because of Rust borrow rules: a mutable RiveScript struct couldn't be passed in like is possible in other languages, so setUservar() keeps a HashMap of staged changes that get resolved and saved _after_ the macro subroutine finishes.
* Only a subset of the RiveScript API is exposed to the proxy. If you want additional methods proxied, pull requests are welcome.

## License

```
The MIT License (MIT)

Copyright (c) 2026 Noah Petherbridge

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```