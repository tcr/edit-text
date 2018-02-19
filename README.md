# edit-text

Clone and set the Rust version:

```
rustup override set `cat rust-toolchain`
rustup target add wasm32-unknown-unknown
```

To test out the text editor live, run the sync server in one terminal:

```
cd mercutio
CARGO_INCREMENTAL=1 RUST_BACKTRACE=1 cargo run --release --bin mercutio-sync
```

And in another terminal, run the client proxy:

```
cd mercutio
CARGO_INCREMENTAL=1 RUST_BACKTRACE=1 cargo run --release --bin mercutio-wasm
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
