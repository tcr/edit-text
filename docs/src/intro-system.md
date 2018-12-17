# System Overview

edit-text is built from the ground up as a collaborative text editor. It uses
operational transform to merge updates from multiple clients, so it requires a
synchronizing server. The server is also in charge of storing page content, so
that every page can be shared via its URL.

There are four components in the system:

* The **Server**, which serves HTTP content, a GraphQL endpoint for performing
  page-level commands, and a WebSocket endpoint for synchronizing document
  content.

* The **Client**, which can connect to a server and synchronize its document
  content. It sends client-side modifications (in the form of operations) to the
  server, and receives updated content (in the form of operations) from the server
  after any client submits an update.

* The **Controller**, which receives UI-level event updates from the frontend
  and converts it into operations on the client document.

* The **Frontend**, which is the editor UI. The current document is rendered
  as a component inside the frontend, and interactions with this component are
  forwarded to the controller. The frontend also manages the toolbar,
  notifications, and dialog boxes.

Each of these four components can be controlled by their **commands**, defined
in `commands.rs`, effectively providing an asynchronous API for each component
in the system.

Here is a diagram representing communication between the Server, Client,
Controller, and Frontend:

```svgbob
+----------+         +--------------+--------------+        +--------------+
|          |         |              |              |        |              |
|  Server  | <--+--->|   Client A   |  Controller  |<------>|   Frontend   |
|          |    |    |              |              |        |              |
+----------+    |    +--------------+--------------+        +--------------+
   (Rust)       |          (Rust + WebAssembly)               (TypeScript)
                |
                |
                |-----> Client B
                |
                |-----> Client C
                |
                +-----> ...
```

Notice that **Client** and **Controller** are part of the same component. This is useful
from an API perspective: commands that are addressed to the client will always
originate from the server, and commands addressed to the Controller will always
originate from the frontend. On the implementation level, however, Client and
Controller are the same process.

The server is a command-line program called `edit-server`. In release mode, it
bundles all client-side code and can be uploaded to a server to run the program
directly.

If you use edit-text in its normal configuration, the Client, Controller, and Frontend all run in your browser as WebAssembly and JavaScript. In proxy mode, the Client and Controller run as a command line program.

The Frontend is written in TypeScript.

## Crate/Module overview

The top-level crates/modules are these:

* `oatie/` is the operational transform library
* `edit-common/` contains code shared by all edit-* crates
* `edit-client/` contains the Client and Controller
* `edit-server/` contains the Server binary
* `edit-frontend/` contains the Frontend code as a Node module compiled with webpack
