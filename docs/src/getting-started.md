# Getting Started

## Requirements

**rustup:** `edit-text` is written in Rust, and so you will need a Rust compiler in order to serve the application. Rust may be installed using your system package manager, but the preferred way to download and install Rust is through the [`rustup` toolchain](http://rustup.rs/) available at rustup.rs. To check if you have `rustup` installed, you can run the following command:

```
$ rustup show active-toolchain
nightly-2018-09-25-x86_64-apple-darwin  # for example
```

This command should print a rust version that is equivalent to the contents of the `./rust-toolchain` file. This is the version of the nightly Rust compiler the project currently depends on. This file is updated periodically; `rustup` will automatically manage downloading and using the correct compiler version for you.

**Node.js:** To build the frontend you will need to install [Node.js](http://nodejs.org/) and [Yarn](http://yarnpkg.com). To install Node.js, see [installation instructions for your OS](https://nodejs.org/en/download/package-manager/). To check if you have a recent version of Node.js installed, see if the output of this command is `>= v6.0.0`:

```
$ node -v
v10.12.0  # for example
```

The frontend is written partly in TypeScript, and the build tool uses Yarn to install and manage its JavaScript package dependencies. To install yarn, follow the [installation instructions for your OS](https://yarnpkg.com/en/docs/install#mac-stable) or just run `npm i -g yarn`. To see if Yarn is installed and available:

```
$ yarn -v
v1.10.1  # for example
```

## Usage

Clone the repository from Github:

```
git clone https://github.com/tcr/edit-text
```

Build commands are executed using the `./tools` script. You can rebuild individual edit-text components with `./tools server-build`, `./tools frontend-build`, etc. Run `./tools help` for more information.

### Run the server

The production configuration of edit-text is a long-running server process, and
one or many WebAssembly + TypeScript clients running in the browser that connect
to it.

You can build the WebAssembly client as well as the frontend webpack module
using the following command:

```sh
./tools frontend-build
```

This cross-compiles all frontend code and pulls in the compiled WebAssembly binary,
using [wasm-bindgen](https://github.com/rustwasm/wasm-bindgen) to talk to the frontend.

In your terminal session, you can then run this command to start the server:

```
./tools server
```

Now open <http://localhost:8000/> and you are brought to a welcome page to start
editing text!

### Development Workflow

A simple development pattern is to launch `./tools server` in one window, and 
to watch and recompile frontend code whenever it's modified with this command in
another window:

```sh
./tools frontend-watch
```

This command watches the edit-frontend directory and continuously builds its
after each change. The frontend will periodically display a notification if a
newer version of the client code has been compiled.

After you make changes to `edit-server/`, `edit-common/`, or `oatie/`, you
should kill and re-run the `./tools server` command to rebuild and launch it.
The `frontend-watch` command will automatically rebuild code that it depends
on for you.
