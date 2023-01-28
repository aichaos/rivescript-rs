use env_logger;
use log::warn;
use rivescript::RiveScript;
use std::{path::PathBuf, process::exit, fs, env};
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

fn main() {
    let opt = Opt::from_args();
    println!("{:#?}", opt);

    if opt.files.len() == 0 {
        println!("Usage: rivescript [options] path/to/brain");
        println!("See `rivescript --help` for documentation.");
        exit(1);
    }

    // Debug logging mode.
    if opt.debug {
        env::set_var("RUST_LOG", "debug");
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
    bot.depth = 256;

    warn!("RiveScript-rs v{}", rivescript::VERSION);

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
}
