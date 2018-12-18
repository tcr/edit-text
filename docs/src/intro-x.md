# Build with ./tools

edit-text has a custom build script written in Rust that you invoke by running
`./tools` in the project root directory. This is a basic wrapper over `cargo`
and `npm` commands and other essential project functionality, and provides an
easy way to launch the edit-text server and compile the frontend JavaScript
bundle.

To see a list of build commands, open the project directory in your terminal
and run the following command:

```
./tools help
```

NOTE: If you are on Windows running in `cmd.exe`, you will need to
invoke the build tool with `.\tools` instead. Please substitute `./tools` with
`.\tools` throughout this guide.

## Building the Server

To build the edit-text server:

```sh
./tools server-build
```

To build and launch the server on HTTP port `8000`:

```sh
./tools server
```

## Building the Frontend

The frontend is all the JavaScript code that runs in the browser, and optionally including the WASM build system. To build the frontend, run this from the root directory:

```sh
./tools frontend-build
```

If you want to launch a long-lived script to build the frontend and rebuild each time a frontend file changes:

```sh
./tools frontend-watch
```

### Just compiling the WebAssembly client

Building *just* the frontend WebAssembly component generated from `edit-client` can be done using this command:

```sh
./tools wasm-build
```

This will compile the wasm bundle and save it to `edit-frontend/src/bindgen`, which will be linked with the frontend code bundle. WASM is automatically compiled during the `frontend-build` or `frontend-watch` steps.

## Testing

This command will run all unit tests as well as integration tests (end-to-end testing using a machine-controlled browser).

```sh
./tools test
```

If you're in a continuous integration (CI) environment, you can perform all relevant test steps for your branch by running:

```sh
./tool ci
```

## Client Proxy

If you are testing changes to the `edit-client` library, you have the option of choosing between running client code in the browser (via WebAssembly) or running it in a local Rust process, having all commands proxied through websockets.

```sh
./tools client-proxy
```

## Building the book

You can build the book with the book-build command:

```sh
./tools book-build
```

Or watch for all changes as they are being made with book-watch.

```sh
./tools book-watch
```

By navigating to <http://localhost:3000/>, you'll see the page refresh automatically as you edit markdown files under `docs-src/`.


## Running edit-text with a client in proxy mode (for debugging)

**NOTE:** You can skip this section if you are just getting started.

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
