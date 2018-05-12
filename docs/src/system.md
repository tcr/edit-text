# System Diagram

Basic system diagram:

```

       /-> User <-> Frontend
Sync <-+-> User <-> Frontend
       |-> User <-> Frontend
       \-> ...
```

## Sync

The Sync server performs document synchronization. It is the "server" component that orchestrates simultaneous document modifications which happen on several Users.

It has a websocket component for clients to orchestrate simultaneous document modifications.

It also has a GraphQL endpoint to (TODO: fill this out) make modifications outside the client API.


## User

The User represents a consumer of the document. They can make changes to the document and apply modifications. This is performed over the WebSocket API.

## Frontend

The frontend is the UX that interfaces with the editor. This is split out from the User node for two reasons:

1. Rust components intended to run in the browser have a mode in which they can run in the command line as a "proxy".
2. It should be possible for a frontend to be written for any environment, not just the web. For example, GTK+ or Qt could be a frontend instead of HTML is that were desirable.

# API

The API between two layers is defined in several enums representing payloads across RPC boundaries.

## Interop Sync <-> User

Defined in `mercutio-client/src/client.rs`.

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

Defined in `mercutio-client/src/client.rs`.

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
    MarkdownUpdate(String),
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
    RequestMarkdown,
}
```
