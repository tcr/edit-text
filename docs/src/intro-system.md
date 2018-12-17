# System Overview

edit-text is built from the ground up as a collaborative text editor. It uses operational transform to merge updates from multiple clients, so it requires a synchronizing server. The server is also in charge of storing page content, so that every page can be shared via its URL.

A client runs in a browser: text editing and synchronization code is written in Rust and cross-compiled to WebAssembly, and frontend code is written in TypeScript.

Here's a diagram demonstrating the different components:

```svgbob
+--------+        +---------------+--------------+        +--------------+
| Server | <--+-->|   Client A    |  Controller  |<------>|   Frontend   |
| (Rust) |    |   |---------------'--------------|        | (TypeScript) |
|        |    |   |         (Rust + Wasm)        |        |              |
+--------+    |   +------------------------------+        +--------------+
              |
              |-----> Client B
              |
              |-----> Client C
              |
              +-----> ...
```

Notice that Client and Controller are part of the same component. This is useful
from an API perspective: commands that are addressed to the client will always
originate from the server, and commands addressed to the Controller will always
originate from the frontend. On the implementation level, however, Client and
Controller are the same process.

## Server APIs

The server performs document synchronization. It is the "server" component that orchestrates simultaneous document modifications which happen on several Users.

| Port | Path Mapping | Description 
|------|------|-------------
| 8000 | /    | HTML Server
| 8002 | /$/ws    | WebSocket
| 8003 | /$/graphql    | GraphQL

HTML is served from `/`. Static versions of each page are available before scripting is fully downloaded.

When the client-side script connects the WebSocket, the server recognizes it as a new synchronization client and reloads the content of the page. Editing is then enabled. Each edit made by the client is sent to the server as an operation, and the server computes and pushes push new deltas to the client.

There is an additional API exposed as GraphQL for non-synchronization tasks. This exposes mutations like updating a page with Markdown, downloading and renaming pages, and other page-editing features.

## Frontend

The edit-text client is written in Rust and can be run both in the browser (to power the editor) or from the command line (for tools like the client proxy, and client replay).

The frontend invokes the client over a `wasm-bindgen` bridge, exchanging JSON messages ("commands"). The frontend exposes an editor interface using React. The client instructs the frontend on what text styling options to expose, and responds to keypresses with updated HTML to render in the editor.

## Crate/Module overview

The top-level crates/modules are these:

* oatie, the operational transform crate
* edit-common, the shared code crate
* edit-client, the client crate
* edit-server, the server crate
* edit-frontend, the TypeScript module using webpack
