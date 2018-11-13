# commandspec

Simple Rust macro for building `std::process::Command` objects. Uses macro_rules! and works on stable.

```toml
[dependencies]
commandspec = "0.10"
```

Then:

```rust
#[macro_use]
extern crate commandspec;

use commandspec::CommandSpec; // .execute() method on Command
use std::process::Command;

let result = execute!(
    r"
        cd path/location
        export RUST_LOG=full
        export RUST_BACKTRACE=1
        cargo run {release_flag} --bin {bin_name} -- {args}
    ",
    release_flag=Some("--release"),
    bin_name="binary",
    args=vec!["arg1", "arg2"],
)?;
// result = Ok(()) on success (error code 0), Err(CommandError) for all else
```

Format of the commandspec input, in order:

* (optional) `cd <path>` to set the current working directory of the command, where path can be a literal, a quoted string, or format variable.
* (optional) one or more `export <name>=<value>` lines to set environment variables, with the same formatting options.
* Last, a command you want to invoke, optionally with format arguments.

### Features:

* format-like invocation makes it easy to interpolate variables, with automatic quoting
* Equivalent syntax to shell when prototyping
* Works on stable Rust.

## License

MIT or Apache-2.0, at your option.
