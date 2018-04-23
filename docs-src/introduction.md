# edit-text

edit-text is a collaborative text editor, written in Rust.

* oatie, an operational transform library
* mercutio, the frontend and backend to edit-text

The frontend is broken out into several crates:

* mercutio-common
* mercutio-client
* mercutio-server
* mercutio-frontend

## Usage

Invoking the server:

```
./x.rs server [--release] [--client-proxy]
```

Invoking the client proxy:

```
./x.rs client-proxy [--release]
```

Building the frontend (TypeScript):

```
./x.rs frontend-build
./x.rs frontend-watch # watches builds and rebuilds as you edit
```

Building the WebAssembly bundle:

```
./x.rs wasm-build
```
