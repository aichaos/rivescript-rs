use env_logger;
use rivescript::RiveScript;

fn main() {
    env_logger::init();

    let mut bot = RiveScript::new();
    bot.utf8 = true;
    bot.depth = 256;

    // bot.load_directory("eg/brain")
    //     .expect("Couldn't load directory!");

    // bot.load_file("eg/brain/begin.rive").expect("ok");

    // bot.stream(String::from("+ stream test\n- stream OK\n! version = 2"))
    //     .expect("Couldn't stream!");

    bot.load_file("test.rive").expect("ok");

    bot.sort_triggers();
}
