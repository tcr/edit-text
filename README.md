# edit-text

![Preview Image](https://user-images.githubusercontent.com/80639/37248514-50f31bcc-24a2-11e8-9be0-9f7d6132289b.png)

edit-text is a collaborative rich text editor, written in Rust with a frontend in WebAssembly.

## Running

You'll need `cargo-script` to run the build tool and `diesel-cli` for the Sqlite file:

```
cargo install cargo-script
cargo install diesel_cli --no-default-features --features sqlite
```

Clone and set the Rust version:

```
rustup override set `cat rust-toolchain`
rustup target add wasm32-unknown-unknown
```

To test out the text editor live, compile our wasm bundle then run the server:

```
diesel setup
./x.rs wasm-build
./x.rs server
```

Then go to <localhost:8000> and start editing.

## Local client proxy in Rust (no WASM)

Set the sync server in one terminal with this switch:

```
./x.rs server --client-proxy
```

In another terminal, run the client proxy:

```
./x.rs client-proxy
```

Then go to <localhost:8000> and start editing.

## License

Apache-2.0

Favicon bear by Alexander Krasnov from the Noun Project
