# System Diagram

Basic system diagram:

```

     v-> User <-> Frontend
Sync <-> User <-> Frontend
     ^-> User <-> Frontend
```

## Sync

The Sync server performs document synchronization. It is the "server" component that orchestrates simultaneous document modifications which happen on several Users.

It has a websocket component for clients to orchestrate simultaneous document modifications.

It also has a GraphQL endpoint to (TODO: fill this out) make modifications outside the client API.


## User

The User represents a consumer of the document. They can make changes to the document and apply modifications. This is performed over the WebSocket API.


## Interop Sync <-> User

```
pub enum SyncServerCommand {
    // Connect(String),
    Commit(String, Op, usize),
    TerminateProxy,
}

pub enum SyncClientCommand {
    // Client id assignment, initial doc, initial version
    Init(String, DocSpan, usize),

    // New document, version, client-id, operation
    Update(DocSpan, usize, String, Op),
}
```


## Frontend

The frontend is the UX that interfaces with the editor.

...

## Intop: User <-> Frontend

Defined in `mercutio-client/src/client.rs`.

From User -> Frontend:

```
pub enum ClientCommand {
    Init(String),
    Controls {
        keys: Vec<(u32, bool, bool)>,
        buttons: Vec<(usize, String, bool)>,
    },
    PromptString(String, String, NativeCommand),
    Update(String, Option<Op>),
    MarkdownUpdate(String),
    Error(String),
    SyncServerCommand(SyncServerCommand),
}
```

And from Frontend -> User:

```
pub enum NativeCommand {
    // Connect(String),
    Keypress(u32, bool, bool, bool), // code, meta, shift, alt
    Button(u32),
    Character(u32),
    RenameGroup(String, CurSpan),
    // Load(DocSpan),
    Target(CurSpan),
    RandomTarget(f64),
    Monkey(bool),
    RequestMarkdown,
}
```
