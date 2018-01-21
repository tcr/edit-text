.PHONY:

wasm:
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

mercutio-server:
	cd mercutio && CARGO_INCREMENTAL=1 cargo run --release --bin mercutio-sync

test: oatie-build
	cd mercutio && cargo script failrun.rs