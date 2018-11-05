# Deploying to Dokku

[macOS Instructions] To deploy to a Dokku instance, you'll have to install a linux cross-compiler. This will cross-compile the Linux binary locally and then package it in a Docker container for distribution.

First, to install the cross compiler:

```
brew install qinyao-he/homebrew-gcc_cross_compilers/x64-elf-gcc
```

Then add a new Rust target:

```
rustup target add x86_64-unknown-linux-gnu
```

You can deploy edit-text to a Dokku server using `./x.rs deploy`.

```
./tools deploy
```

* This first cross-compiles the edit-server binary using a local Docker image.
* It then uploads the binary using the `dokku tar:in` command on a remote server (not the Git endpoint).
* You can configure the dokku URL using the `EDIT_DEPLOY_URL` environment variable.
* You can configure the dokku application name using the `EDIT_DOKKU_NAME` environment variable.
