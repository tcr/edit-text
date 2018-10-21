# Build Tool ./tools

`./tools` is the build tool. Invocation will be different if you are on Windows or a Linux or macOS system:

```
./tools help  # Linux, macOS, and Powershell
tools help    # Windows (cmd.exe)
```

Please substitute either `./tools` or `tools` in the examples below accordingly. Running the `help` should display a list of build steps that available for edit-text.

## Server building

To launch the edit-text server:

```sh
./tools server-build
```

To run it on port `8000`:

```sh
./tools server
```

## Frontend build

The frontend is all the JavaScript code that runs in the browser, and optionally including the WASM build system. To build the frontend, run this from the root directory:

```sh
./tools frontend-build
```

If you want to launch a long-lived script to build the frontend and rebuild each time a frontend file changes:

```sh
./tools frontend-watch
```

### Building WASM

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
./tool sh
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
