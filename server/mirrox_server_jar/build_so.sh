#!/bin/bash
set -e

# CONFIG

rustup target add aarch64-linux-android

cargo install cargo-ndk

cd rust_binder

cargo ndk clean

cargo ndk -t arm64-v8a -o ./target/ --platform 21 build