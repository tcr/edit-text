# Server APIs

The server performs document synchronization. It is the "server" component that orchestrates simultaneous document modifications which happen on several Users.

| Port | Path Mapping | Description 
|------|------|-------------
| 8000 | /    | HTML Server
| 8002 | /$/ws    | WebSocket
| 8003 | /$/graphql    | GraphQL

HTML is served from `/`. Static versions of each page are available before scripting is fully downloaded.

When the client-side script connects the WebSocket, the server recognizes it as a new synchronization client and reloads the content of the page. Editing is then enabled. Each edit made by the client is sent to the server as an operation, and the server computes and pushes push new deltas to the client.

There is an additional API exposed as GraphQL for non-synchronization tasks. This exposes mutations like updating a page with Markdown, downloading and renaming pages, and other page-editing features.# Server APIs
