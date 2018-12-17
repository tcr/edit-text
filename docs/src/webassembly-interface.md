# WebAssembly Interface

The frontend invokes the client over a `wasm-bindgen` bridge, exchanging JSON messages ("commands"). The frontend exposes an editor interface using React. The client instructs the frontend on what text styling options to expose, and responds to keypresses with updated HTML to render in the editor.

