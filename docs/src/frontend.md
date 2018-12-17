# Frontend

The Frontend is written in TypeScript and communicates with a WebAssembly
component containing the Client and Controller.

The edit-text client is written in Rust and can be run both in the browser (to
power the editor) or from the command line (for tools like the client proxy, and client replay).

The frontend invokes the client over a `wasm-bindgen` bridge, exchanging JSON messages ("commands"). The frontend exposes an editor interface using React. The client instructs the frontend on what text styling options to expose, and responds to keypresses with updated HTML to render in the editor.

The frontend is written in React.