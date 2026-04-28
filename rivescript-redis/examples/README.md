# Redis Example

This example program demonstrates the use of the rivescript-redis crate to proactively store user variables in Redis for your RiveScript bot.

## Quick Start

If you don't have a Redis server, you can run one locally with Docker:

```bash
docker run -p 6379:6379 -d redis
```

Run the example:

```bash
cargo run --example redis_demo -p rivescript-redis
```

## Command Line Arguments

The demo makes some default assumptions for its options, but you can customize all of these using command line flags.

To see the available flags and documentation:

```bash
cargo run --example redis_demo \
    -p rivescript-redis -- --help
```

The main options and defaults are as follows:

* `--redis "redis://127.0.0.1/"`

    Provide the connection string to your Redis server. The default will use a localhost Redis and the default DB number (0).

* `--prefix "rs_demo"`

    Customize the Redis key prefix. All keys will use this prefix so that we don't clobber other keys you may be using in an existing Redis cache.

    An example Redis key used would be `rs_demo:history:username:reply` where `username` is the distinct RiveScript username for a particular user.

* `brain = ${CARGO_MANIFEST_DIR}/../eg/brain`

    This positional argument defines a folder full of .rive files that make up your bot's brain.

    The default is to try and use the /eg/brain example folder that lives in the rivescript-rs git project. In case that folder is not available to you (e.g., you are running the demo externally), you may provide your own path to a RiveScript brain to be used instead.

An example with customized parameters including a custom brain path:

```bash
cargo run --example redis_demo -p rivescript-redis -- \
    --redis 'redis://127.0.0.1/2' \
    --prefix 'rivescript' \
    ./path/to/rivescript/brain
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