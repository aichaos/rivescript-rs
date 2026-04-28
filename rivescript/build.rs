/*
The build.rs handles setting up the tests/rsts git submodule for unit testing.
rsts is the RiveScript Test Suite which contains a series of YAML files for
testing various features of RiveScript.

The build.rs only runs for developers of rivescript-rs when working out of
the project repository for this crate; not when end users of the library are
building their project.
*/

fn main() {
    // For rivescript-rs developers working out of the project git root:
    // ensure that the rsts submodule is initialized for unit tests.
    if std::env::var("PROFILE").unwrap_or_default() == "debug" {
        if !std::path::Path::new("tests/rsts/.git").exists() {
            println!("cargo:warning=RSTS submodule not found. Integration tests will be skipped. Run `git submodule update --init --recursive` to initialize.");
        }
    }
    println!("cargo:rerun-if-changed=tests/rsts");
}