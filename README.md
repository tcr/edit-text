# oatie

Run the transformer tester:

```
cd oatie
cat in/1 | cargo +nightly run --bin oatie-transform
```

# Mercutio

To test out the text editor live live:

```
CARGO_INCREMENTAL=1 RUST_BACKTRACE=1 cargo run --release --bin mercutio-sync
```

In another window:

```
CARGO_INCREMENTAL=1 RUST_BACKTRACE=1 cargo run --release --bin mercutio-wasm
```

## license

mit
