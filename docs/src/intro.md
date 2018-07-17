# edit-text

edit-text is a document editor that is Markdown-compatible and supports collaborative editing. Its server and client are written in Rust, and its frontend is cross-compiled to WebAssembly and TypeScript.

## Installation

Before building, you'll need to install the following cargo tools:

```
cargo install cargo-script
cargo install wasm-bindgen-cli
cargo install diesel_cli --no-default-features --features sqlite-bundled
```

Next, install the WebAssembly target:

```
rustup target add wasm32-unknown-unknown
```

<!-- Whenever the `rust-toolchain` file is updated, re-run the `rustup target add` command to fetch the latest wasm32 target. -->

## Usage

To build all components of edit-text (server, client, and frontend) you can use the `build` command:

```
./x.rs build
```

You can rebuild individual components with `server-build`, `frontend-build`, etc. Check `./x.rs help` for more information.

### Running edit-text with WebAssembly

edit-text using WebAssembly is the "release" mode of edit-text, where you run a server process and then one or many WebAssembly + TypeScript client running in the browser can connect to it.

In your terminal session, you can run the following command (and optionally use release mode):

```
./x.rs server [--release]
```

Now open http://localhost:8000/ and you are brought to a welcome page to start editing.

After any changes to client or server code, run `./x.rs build` and restart the server process.

### Running edit-text with a client in proxy mode (for debugging)

Debugging WebAssembly code is harder (in most ways) than debugging a local Rust binary. edit-text supports running the client as an independent "proxy". An edit-text server process connects to a client proxy running in another process, and communicates with browser processes using WebSockets. This client proxy is all code that would normally be cross-compiled to WebAssembly, but runs locally in your terminal and supports the same backtrace and debugging support as a local binary.

You'll need two terminal sessions to run in this mode. First, start the server, and specify that you want to connect to a client proxy using `--client-proxy`. Without this argument, the server will expect server connections from WebAssembly instead.

```
./x.rs server --client-proxy [--release]
```

In another terminal session, you can start the proxy. (It's recommended you compile in release mode, as client code is much slower in debug mode.)

```
./x.rs client-proxy [--release]
```

Then you can open http://localhost:8000/ as before in your browser, and monitor the `client-proxy` script for status of the clients that your browser is connected to.

If you encounter a panic or fatal error, the client-proxy mechanism of debugging usually gives much more information about where the error originated. Note that aside from running as a binary, there should be no differences between client-proxy mode and cross-compiling to Webassembly.

## Crate overview

The top-level crates are these:

* oatie, an operational transform library
* edit-common, the frontend and backend to edit-text
* edit-client
* edit-server
* edit-frontend
