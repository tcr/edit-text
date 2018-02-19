# edit-text

Clone and set the Rust version:

```
rustup override set `cat rust-toolchain`
rustup target add wasm32-unknown-unknown
```

You'll also need `cargo-script` to run the build tool:

```
cargo install cargo-script
```

To test out the text editor live, run the sync server in one terminal:

```
./x.rs mercutio-sync
```

Then go to <localhost:8000> and start editing.

## Proxy mode

Set the sync server with this switch (TODO: switch doesnt work atm):

```
./x.rs mercutio-sync --client-proxy
```

In another terminal, run the client proxy:

```
./x.rs wasm-proxy
```

Then go to <localhost:8000> and start editing.

## Transform test

Run the transformer tester:

```
cd oatie
cat in/1 | cargo run --bin oatie-transform
```

## license

mit
