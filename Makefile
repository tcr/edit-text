.PHONY:

wasm:
	cd mercutio/mercutio-wasm && CARGO_INCREMENTAL=0 cargo check --target wasm32-unknown-unknown --release --lib
	cd mercutio/mercutio-wasm && CARGO_INCREMENTAL=0 cargo build --target wasm32-unknown-unknown --release --lib
	cd mercutio/mercutio-wasm && cp ../../target/wasm32-unknown-unknown/release/mercutio.wasm ../frontend/dist

oatie-build:
	cd oatie && CARGO_INCREMENTAL=1 cargo build --release

js-build:
	cd mercutio/frontend && npx webpack ./src/app.ts ./dist/mercutio.js

js-watch:
	cd mercutio/frontend && npx webpack --watch ./src/app.ts ./dist/mercutio.js

mercutio-build:
	cd oatie && CARGO_INCREMENTAL=1 cargo build

mercutio-sync-build:
	cd mercutio && RUST_BACKTRACE=1 CARGO_INCREMENTAL=1 MERCUTIO_SYNC_LOG=1 cargo build --bin mercutio-sync

mercutio-sync-nolog:
	cd mercutio && RUST_BACKTRACE=1 CARGO_INCREMENTAL=1 cargo run --bin mercutio-sync -- --period 100

wasm-proxy:
	cd mercutio && RUST_BACKTRACE=1 CARGO_INCREMENTAL=1 MERCUTIO_WASM_LOG=1 cargo run --bin mercutio-wasm-proxy --release

mercutio-replay:
	cd mercutio && RUST_BACKTRACE=1 CARGO_INCREMENTAL=1 cargo run --release --bin mercutio-replay

# test: oatie-build
# 	cd mercutio && cargo script failrun.rs

test: oatie-build
	cd oatie && ./transform-test.sh