@echo off

REM Calling .\tools <COMMAND> is equivalent to cargo run --bin build-tools -- <COMMAND>.
REM This convenience script will build the tool binary if it does not yet exist;
REM otherwise it will quiety rebuild the binary and then run it with your arguments.

if not exist .\target\debug\build-tools.exe cargo build --bin build-tools
cargo run --bin build-tools --quiet -- %*
