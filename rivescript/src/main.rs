use env_logger;
use log::{debug, warn};
use rivescript::RiveScript;
use futures::FutureExt;
use rivescript_core::macros::Proxy;
use std::{env, fs, io, io::Write, path::PathBuf, process::exit};
use structopt::StructOpt;

/// Command-line flags.
#[derive(StructOpt, Debug)]
#[structopt(
    name = "rivescript",
    about = "A stand-alone RiveScript chatbot shell for command line use.",
)]
struct Opt {
    /// Activate debug mode.
    #[structopt(short, short, long)]
    debug: bool,

    /// Enable UTF-8 mode in the RiveScript interpreter.
    #[structopt(short, short, long)]
    utf8: bool,

    /// RiveScript source documents (*.rive files) or directories of documents
    /// that make up your bot's personality. Multiple inputs will be loaded in
    /// the order specified on the command line.
    #[structopt(name = "FILES", parse(from_os_str))]
    files: Vec<PathBuf>,
}

#[tokio::main]
async fn main() {
    let opt = Opt::from_args();
    println!("{:#?}", opt);

    if opt.files.len() == 0 {
        println!("Usage: rivescript [options] path/to/brain");
        println!("See `rivescript --help` for documentation.");
        exit(1);
    }

    // Debug logging mode.
    if opt.debug {
        unsafe {
            env::set_var("RUST_LOG", "debug");
        }
    }

    // let e.g. RUST_LOG=debug to set debug output.
    env_logger::init();


    println!("      .   .
     .:...::      RiveScript Interpreter (Rust)
    .::   ::.     Library Version: v{} (build {})
 ..:;;. ' .;;:..
    .  '''  .     Type '/quit' to quit.
     :;,:,;:      Type '/help' for more options.
     :     :
Using the RiveScript bot found in: {:?}
Type a message to the bot and press Return to send it.",
        rivescript::VERSION,
        "n/a",
        opt.files,
    );

    let mut bot = RiveScript::new();
    bot.utf8 = opt.utf8;

    warn!("RiveScript-rs v{}", rivescript::VERSION);

    // Register the JavaScript handler?
    #[cfg(feature = "javascript")]
    {
        println!("Note: JavaScript object macros enabled.");
        rivescript::register_default_js_handler(&mut bot);
    }

    // An example object macro written in Rust.
    bot.set_subroutine("rust-set", |proxy, args| {
        async move {
            if args.len() >= 2 {
                let username = proxy.current_username();

                let name = args.get(0).unwrap();
                let value = args.get(1).unwrap();
                let orig_value = proxy.get_uservar(&username, &name).await;

                proxy.set_uservar(&username, name, value).await.expect("Couldn't set user variable!");
                let staged_value = proxy.get_uservar(&username, &name).await;

                return proxy.finish(format!("For username {username}: The original variable '{name}' was '{orig_value}' and I have updated it to '{value}' (staged value: '{staged_value}')"));
            }
            proxy.finish("Usage: rust-set name value".to_string())
        }.boxed()
    });
    bot.set_subroutine("rust-bot-set", |proxy, args| {
        async move {
            if args.len() >= 2 {
                let name = args.get(0).unwrap();
                let value = args.get(1).unwrap();
                let orig_value = proxy.get_variable(&name);

                proxy.set_variable(name, value);
                let staged_value = proxy.get_variable(&name);

                return proxy.finish(format!("The original bot variable '{name}' was '{orig_value}' and I have updated it to '{value}' (staged value: '{staged_value}')"));
            }
            proxy.finish("Usage: rust-set name value".to_string())
        }.boxed()
    });

    // Load all the input files/directories in order.
    for pathbuf in opt.files {
        let filename = pathbuf.to_str().unwrap();
        let attr = fs::metadata(filename).expect(format!("{}: file not found", filename).as_str());

        if attr.is_dir() {
            bot.load_directory(filename)
                .expect(format!("Error loading from directory {}", filename).as_str());
        } else if attr.is_file() {
            bot.load_file(filename).expect(format!("Error loading file {}", filename).as_str());
        }
    }

    // bot.load_file("eg/brain/begin.rive").expect("ok");

    // bot.stream(String::from("+ stream test\n- stream OK\n! version = 2"))
    //     .expect("Couldn't stream!");

    // bot.load_file("test.rive").expect("ok");

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
                println!("/dump-ast: pretty-print the loaded brain AST contents");
                println!("/help: show this help message");
                println!("/quit: exit the program");
            },
            "/dump-ast" => {
                bot.debug_print_brain();
            },
            "/dump-sorted" => {
                bot.debug_sorted_replies();
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
                        debug!("Error: {e}");
                    }
                };
            }
        }

    }
}
