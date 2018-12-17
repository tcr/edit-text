# Running as a binary

The `edit-server/` crate can export a binary that you can deploy to a remote
server, containing the source code of the client and the ability to talk to
a locally instantiated SQLite database.

```
./tools server
```

To build a standalone binary that bundles client files statically:

```
cd edit-server
cargo build --release --features standalone
../target/release/edit-server
```
