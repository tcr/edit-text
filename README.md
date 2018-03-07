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
./x.rs server
```

Then go to <localhost:8000> and start editing.

## Local wasm (client) proxy

Set the sync server with this switch:

```
./x.rs server --client-proxy
```

In another terminal, run the client proxy:

```
./x.rs client-proxy
```

Then go to <localhost:8000> and start editing.

## Transform test

Run the transformer tester:

```
cd oatie
cat in/1 | cargo run --bin oatie-transform
```

## License

Apache-2.0

Favicon bear by Alexander Krasnov from the Noun Project
