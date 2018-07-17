# [WIP] Deploying

[macOS Instructions] To deploy to a Dokku instance, you'll have to install a linux cross-compiler. This will cross-compile the Linux binary locally and then package it in a Docker container for distribution.

First, to install the cross compiler:

```
brew install qinyao-he/homebrew-gcc_cross_compilers/x64-elf-gcc
```

Then add a new Rust target:

```
rustup target add x86_64-unknown-linux-gnu
```

Then modify `./x.rs` (for the moment) to add your hostname and credentials, then:

```
./x.rs deploy
```
