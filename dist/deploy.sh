#!/bin/bash

set -e
set -x

BASEDIR=$(dirname $0)
cd $BASEDIR

cd ..

./x.rs js-build

./x.rs wasm

vagrant up
vagrant ssh -c "cd /vagrant/mercutio; rustup override set `cat rust-toolchain`; cargo build --release --target=x86_64-unknown-linux-gnu --bin mercutio-sync"
cp target/x86_64-unknown-linux-gnu/release/mercutio-sync dist

cd dist

docker build -t dokku/edit-text:latest .; docker save dokku/edit-text:latest | bzip2 | ssh root@sandbox.edit.io "bunzip2 | docker load"; ssh root@sandbox.edit.io "dokku tags:deploy edit-text latest"
