# Build Tool ./x.rs

`./x.rs` is the build tool. In order to run it, you'll need to install [`cargo-script`](https://github.com/DanielKeep/cargo-script):

```
cargo install cargo-script
```

Then while you're in the root of the `edit-text/` respository, from your command line you can run:

```sh
./x.rs help
```

## Server building

To launch the edit-text server:

```sh
./x.rs server-build
```

To run it on port `8000`:

```sh
./x.rs server
```

## Frontend build

The frontend is all the JavaScript code that runs in the browser, and optionally including the WASM build system. To build this, you should run `npm install` in the `mercutio-frontend/` directory:

```sh
cd mercutio-frontend
npm install
```

This will locally install Webpack, Typescript, and all JavaScript dependencies required by the frontend. To then build the frontend, run this from the root directory:

```sh
./x.rs frontend-build
```

If you want to launch a long-lived script to build the frontend and rebuild each time a relevant file changes:

```sh
./x.rs frontend-watch
```

Building the frontend component may also require that you use build the WASM bundle from `mercutio-client`, which can be generated with this command:

```sh
./x.rs wasm-build
```

This will compile the wasm bundle and save it to `mercutio-frontend/src/bindgen`, which will be linked with the frontend code bundle (generated using `frontend-build` or `frontend-watch`).

## Testing

This command will run comprehensive end-to-end tests. It's used by CI to test all new pull requests:

```sh
./x.rs test
```

## Client Proxy

If you are testing changes to the `mercutio-client` library, you have the option of choosing between running client code in the browser (via WebAssembly) or running it in a local Rust process, having all commands proxied through websockets.

```sh
./x.rs client-proxy
```

## Building the book

You can build the book with the book-build command:

```sh
./x.rs book-build
```

Or watch for all changes as they are being made with book-watch.

```sh
./x.rs book-watch
```

By navigating to <http://localhost:3000/>, you'll see the page refresh automatically as you edit markdown files under `docs-src/`.

## Deploy

You can deploy edit-text to a Dokku server using `./x.rs deploy`.

* This first cross-compiles the mercutio-server binary using a local Docker image.
* It then uploads the binary using the `dokku tar:in` command on a remote server (not the Git endpoint).
* You can configure the dokku URL using the `EDIT_DEPLOY_URL` environment variable.
* You can configure the dokku application name using the `EDIT_DOKKU_NAME` environment variable.

## Help

```
edit-text build scripts 0.1.0
Build scripts for mercutio and oatie

USAGE:
    x <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    book-build        Builds the book.
    book-watch        Watches and rebuilds the book.
    client-proxy      Run client code in your terminal.
    deploy            Deploy to sandbox.edit.io.
    frontend-build    Bundle the frontend JavaScript code.
    frontend-watch    Watch the frontend JavaScript code, building continuously.
    help              Prints this message or the help of the given subcommand(s)
    oatie-build       Build the operational transform library.
    replay            Replay an edit-text log.
    server            Run the edit-text server.
    server-build      Build the edit-text server.
    test              
    wasm-build        Compile the WebAssembly bundle.
```
