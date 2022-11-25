use env_logger;
use rivescript::RiveScript;

fn main() {
    env_logger::init();

    let mut bot = RiveScript::new();
    bot.utf8 = true;
    bot.depth = 256;

    bot.load_directory("eg/brain")
        .expect("Couldn't load directory!");

    bot.stream(String::from("+ stream test\n- stream OK"))
        .expect("Couldn't stream!");

    bot.sort_triggers();
}
