# System Overview

edit-text is built from the ground up as a collaborative text editor. It uses operational transform to merge updates from multiple clients, so it requires a synchronizing server. The server is also in charge of storing page content, so that every page can be shared via its URL.

A client runs in a browser: text editing and synchronization code is written in Rust and cross-compiled to WebAssembly, and frontend code is written in TypeScript.

Here's a rough diagram:

```
 Server <-+--> Client <--------> Frontend
 (Rust)   | (Rust + Wasm)     (TypeScript)
          |
          |--> Client <--------> Frontend
          |--> Client <--------> Frontend
          \--> ...
```

## Server APIs

The server performs document synchronization. It is the "server" component that orchestrates simultaneous document modifications which happen on several Users.

```
dev: 0.0.0.0:8000/  prod: /            HTML Server
dev: 0.0.0.0:8002/  prod: /$/ws        WebSocket
dev: 0.0.0.0:8003/  prod: /$/graphql   GraphQL
```

HTML is served from `/`. Static versions of each page are available before scripting is fully downloaded.

When the client-side script connects the WebSocket, the server recognizes it as a new synchronization client and reloads the content of the page. Editing is then enabled. Each edit made by the client is sent to the server as an operation, and the server computes and pushes push new deltas to the client.

There is an additional API exposed as GraphQL for non-synchronization tasks. This exposes mutations like updating a page with Markdown, downloading and renaming pages, and other page-editing features.

## Frontend

The edit-text client is written in Rust and can be run both in the browser (to power the editor) or from the command line (for tools like the client proxy, and client replay).

The frontend invokes the client over a `wasm-bindgen` bridge, exchanging JSON messages ("commands"). The frontend exposes an editor interface using React. The client instructs the frontend on what text styling options to expose, and responds to keypresses with updated HTML to render in the editor.

## Crate/Module overview

The top-level crates/modules are these:

* oatie, the operational transform crate
* simple-ws, a thin websocket wrapper
* edit-common, the shared code crate
* edit-client, the client crate
* edit-server, the server crate
* edit-frontend, the TypeScript module using webpack

<!--
# API

The API between two layers is defined in several enums representing payloads across RPC boundaries.

## Interop Sync <-> User

Defined in `edit-client/src/client.rs`.

From Sync -> User:

```
pub enum SyncToUserCommand {
    // Client id assignment, initial doc, initial version
    Init(String, DocSpan, usize),

    // New document, version, client-id, operation
    Update(DocSpan, usize, String, Op),
}
```

And from User -> Sync:

```
pub enum UserToSyncCommand {
    // Connect(String),
    Commit(String, Op, usize),
    TerminateProxy,
}
```

## Intop: User <-> Frontend

Defined in `edit-client/src/client.rs`.

From User -> Frontend:

```
pub enum UserToFrontendCommand {
    Init(String),
    Controls {
        keys: Vec<(u32, bool, bool)>,
        buttons: Vec<(usize, String, bool)>,
    },
    PromptString(String, String, FrontendToUserCommand),
    Update(String, Option<Op>),
    Error(String),
    UserToSyncCommand(UserToSyncCommand),
}
```

And from Frontend -> User:

```
pub enum FrontendToUserCommand {
    // Connect(String),
    Keypress(u32, bool, bool, bool), // code, meta, shift, alt
    Button(u32),
    Character(u32),
    RenameGroup(String, CurSpan),
    // Load(DocSpan),
    Target(CurSpan),
    RandomTarget(f64),
    Monkey(bool),
}
```
-->