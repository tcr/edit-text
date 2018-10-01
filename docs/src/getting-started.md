# Getting Started

# Requirements

**rustup:** `edit-text` is written in Rust, and so you will need a Rust compiler in order to serve the application. Rust may be installed using your system package manager, but the preferred way to download and install Rust is through the [`rustup` toolchain](http://rustup.rs/) available at rustup.rs. To check if you have `rustup` installed, you can run the following command:

```
$ rustup show active-toolchain
nightly-2018-09-25-x86_64-apple-darwin
```

This result should be equivalent to the value contained in the `./rust-toolchain` file. This value indicates which version of the nightly Rust compiler the project depends on. `rustup` automatically manages downloading and using this compiler version for us.

**Global build tools:** The build environment requires some Rust binary dependencies to be installed using `cargo install`. You can run the following commands to install these requirements one at a time:

```
# This generates the bindings between Rust and JavaScript when compiling to WebAssembly.
cargo install wasm-bindgen-cli
# The diesel command creates database files and manages migrations.
cargo install diesel_cli --no-default-features --features sqlite-bundled
# Watch an entire directory for changes with cargo-watch
cargo install cargo-watch
# (Optional) mdbook is the build system for this documentation you're reading.
cargo install mdbook
```

**Node.js:** You will also need to make sure you have [Node.js](http://nodejs.org/) installed. The build tool uses `npm` to manage frontend dependencies by installing and managing JavaScript packages. To check if you have a recent version of Node, see if the output of this command is `>= v6.0.0`:

```
$ node -v
v9.5.0
```

## Usage

Clone the repository from Github:

```
git clone https://github.com/tcr/edit-text
```

To build all components of edit-text (server, client, and frontend) at once, run this command from the root of the repository:

```
./tools build
```

Build commands are executed using the `./x.rs` script. ([Read more.](http://timryan.org/2018/07/02/moving-from-the-shell-to-rust-with-commandspec.html))  You can rebuild individual edit-text components with `./x.rs server-build`, `./x.rs frontend-build`, etc. Run `./x.rs help` for more information.

### Running edit-text (standard)

The production configuration of edit-text is a long-running server process, and one or many WebAssembly + TypeScript clients running in the browser that connect to it.

In your terminal session, you can run this command to start the server (and optionally compile with release optimizations):

```
./tools server [--release]
```

Now open <http://localhost:8000/> and you are brought to a welcome page to start editing text!

Note that the server also serves WebAssembly code to the browser that contains the edit-text client. After you make changes are made to client or server code, you should re-run `./x.rs build` to recompile both and then restart the server process. (If only server changes were made, you can skip this step and just run `./x.rs server` directly.)

### Running edit-text with a client in proxy mode (for debugging)

Debugging WebAssembly code is harder (in most ways) than debugging a local Rust binary. edit-text supports running the client as an independent "proxy". An edit-text server running in one terminal connects to a client proxy running in another terminal, and communicates with frontend code running in the browser (TypeScript) over WebSockets. This client proxy is all code that would normally be cross-compiled to WebAssembly, but runs locally in your terminal and supports the same backtrace and debugging support as a local binary.

You'll need two terminal sessions to run in this mode. First, start the server, and specify that you want to connect to a client proxy using `--client-proxy`. Without this argument, the server will expect server connections from WebAssembly instead.

```
./tools server --client-proxy [--release]
```

In another terminal session, you can start the proxy. (It's recommended you compile in release mode, as client code is much slower in debug mode.)

```
./tools client-proxy [--release]
```

Then you can open http://localhost:8000/ as before in your browser, and monitor the `client-proxy` terminal for status of the clients that your browser is connected to.

You will see any failures appear in the client-proxy code that would appear in the browser console when in WASM mode. If you encounter a panic or fatal error, this "proxy" mechanism of debugging usually gives much more information about where the error originated. Note that aside from running as a binary, there should be no differences in behavior between the client-proxy and the client in Webassembly.

## Compiling the frontend

If you're made changes to WebAssembly code in "edit-client/", you can cross-compile the wasm binary including any [wasm-bindgen](https://github.com/rustwasm/wasm-bindgen) with this command:

```sh
./tools wasm-build
```

The bundled frontend code (written in TypeScript) is tracked in git and can be run immediately after cloning the repository. You can also compile it yourself. Make sure you have Node installed first, then build the frontend:

```sh
./tools frontend-build
```

This cross-compiles all frontend code and pulls in the compiled WebAssembly binary. To watch and recompile frontend code whenever it's modified, run this command:

```sh
./tools frontend-watch
```

This command watches the edit-frontend directory and continuously builds its after each change. Note that you may need to run `./x.rs wasm-build` as well. 
