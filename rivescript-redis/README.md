# Redis Sessions for RiveScript

This crate provides support for using a [Redis cache](https://redis.io/) to proactively store user variables for RiveScript, instead of the default in-memory HashMap store.

## Quick Start

```rust
use rivescript::RiveScript;
use rivescript_redis::RedisSessionManager;
use std::io;

#[tokio::main]
async fn main() {
    // Set up the Redis session manager for user variable storage.
    let redis_manager = RedisSessionManager::new("redis://127.0.0.1/", "rs_demo").unwrap();

    // Initialize RiveScript and use the Redis session manager.
    let mut bot = RiveScript::new();
    bot.set_session_manager(redis_manager);

    // Load your bot's brain and sort the triggers.
    bot.load_directory("./eg/brain").expect("Error loading replies!");
    bot.sort_triggers();

    // Enter main prompt loop.
    loop {
        print!("You> ");
        io::stdout()
            .flush()
            .expect("oops");
        let mut message = String::new();
        io::stdin()
            .read_line(&mut message)
            .expect("Failed to read line");

        // Get a reply from the bot.
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

## Redis CLI Examples

To inspect the data stored in Redis to verify this program is working, see the following example Redis commands.

Note: the default username used in the demo is "localuser" as seen in these examples.

```bash
# To inspect user variables or freezes
HGETALL rs_demo:user:localuser
GET rs_demo:freeze:localuser

# To inspect the history arrays
LRANGE rs_demo:history:localuser:input 0 -1
LRANGE rs_demo:history:localuser:reply 0 -1
```

The history arrays may have a length between 1 and 9 as RiveScript only stores the most recent 9 inputs and replies (e.g. corresponding to the RiveScript tags `<input1>` thru `<input9>`)

## License

Released under the same terms as RiveScript itself (MIT license).