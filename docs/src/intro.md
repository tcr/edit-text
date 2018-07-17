# edit-text

edit-text is a Markdown-compatible document editor that supports collaborative editing. Its server and client are written in Rust, and its frontend uses TypeScript and WebAssembly.

![Preview Image](https://user-images.githubusercontent.com/80639/42796912-9f2ae852-895a-11e8-9aae-9dede91296bf.png)

### Getting Started

Before building, you'll need to install the following cargo build tools:

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

### Usage

To build all components of edit-text (server, client, and frontend) you can use the `build` command:

```
./x.rs build
```

You can rebuild individual components with `server-build`, `frontend-build`, etc. Check `./x.rs help` for more information.

### Running edit-text (standard)

edit-text using WebAssembly is the production configuration of edit-text, where you run a server process and then one or many WebAssembly + TypeScript clients running in the browser can connect to it.

In your terminal session, you can run the following command to start the server (and optionally compile with release optimizations):

```
./x.rs server [--release]
```

Now open http://localhost:8000/ and you are brought to a welcome page to start editing!

After any changes are made to client or server code, run `./x.rs build` and restart the server process.

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

You will see any failures appear in the client-proxy code that would appear in the browser console when in WASM mode. If you encounter a panic or fatal error, this "proxy" mechanism of debugging usually gives much more information about where the error originated. Note that aside from running as a binary, there should be no differences in behavior between the client-proxy and the client in Webassembly.

### Compiling the frontend

The bundled frontend code (written in TypeScript) is tracked in git, but you can also compile it yourself. Make sure you have Node installed first, then:

```
npm i --prefix ./edit-frontend
./x.rs frontend-watch
```

This command watches the edit-frontend directory and continuously builds all frontend code, including the `wasm-bindgen` bindings.


