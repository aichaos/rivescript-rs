//! Example of using the Redis session manager.
//!
//! By default it will connect to a Redis server on localhost and use 'rs_demo'
//! as the key prefix. You can override these settings with command line flags.
//!
//! Example usage with a Docker Redis server:
//!
//! ```bash
//! docker run -p 6379:6379 -d redis
//! cargo run --example redis_demo -p rivescript-redis
//! ```
//!
//! When run from the rivescript git project, it will by default load the example
//! brain in the '/eg/brain' directory. In case this folder is not available to
//! you, you may provide your own path to a folder of .rive files as a command
//! line option.
//!
//! Full example with CLI flags:
//!
//! ```bash
//! cargo run --example redis_demo -p rivescript-redis -- -r 'redis://localhost/' ./eg/brain
//! ```
//!
//! To inspect the Redis cache and verify it's working, see the following examples.
//! Note: the username in the demo is always 'localuser'
//!
//! ```bash
//! # To inspect user variables or freezes
//! HGETALL rs_demo:user:localuser
//! GET rs_demo:freeze:localuser
//!
//! # To inspect the history arrays
//! LRANGE rs_demo:history:localuser:input 0 -1
//! LRANGE rs_demo:history:localuser:reply 0 -1
//! ```

use rivescript::RiveScript;
use rivescript_redis::RedisSessionManager;
use std::{io, io::Write, path::PathBuf};
use structopt::StructOpt;

/// Command-line flags.
#[derive(StructOpt, Debug)]
#[structopt(
    name = "rivescript",
    about = "A stand-alone RiveScript chatbot shell for command line use.",
)]
struct Opt {

    /// Specify a custom Redis connection URL.
    #[structopt(short, long, default_value = "redis://127.0.0.1/")]
    redis: String,

    /// Specify a custom Redis prefix key to use.
    #[structopt(short, long, default_value = "rs_demo")]
    prefix: String,

    /// Enable UTF-8 mode in the RiveScript interpreter.
    #[structopt(short, short, long)]
    utf8: bool,

    /// Path to a directory of .rive files to load. The default demo will
    /// attempt to load from /eg/brain relative to the git root, but if that
    /// path is not available to you, you may provide your own path.
    #[structopt(name = "BRAIN", parse(from_os_str))]
    brain: Option<PathBuf>,
}

#[tokio::main]
async fn main() {
    // Parse CLI arguments.
    let opt = Opt::from_args();

    // Resolve the path to the RiveScript brain.
    let brain_path = opt.brain.unwrap_or_else(|| {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../eg/brain")
    });
    if !brain_path.exists() {
        eprintln!("Error: Brain directory not found at {:?}", brain_path);
        std::process::exit(1);
    }

    // Set up the Redis session manager for user variable storage.
    let redis_manager = RedisSessionManager::new(&opt.redis, &opt.prefix).unwrap();

    // Initialize RiveScript and use the Redis session manager.
    let mut bot = RiveScript::new();
    bot.set_session_manager(redis_manager);
    bot.utf8 = opt.utf8;

    // Load your bot's brain and sort the triggers.
    bot.load_directory(brain_path.to_str().unwrap()).expect("Error loading replies!");
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

        // Process commands.
        match message.trim() {
            "/help" => {
                println!("/quit: exit the program");
            },
            "/quit" => {
                println!("Bye!");
                break;
            }
            _ => {
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

    }
}
