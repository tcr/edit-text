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

# Doing this in one command may cause dokku to hang, from experience
tar c . | bzip2 | ssh root@sandbox.edit.io "bunzip2 > /tmp/mercutio.tar"
ssh root@sandbox.edit.io "cat /tmp/mercutio.tar | dokku tar:in edit-text"
