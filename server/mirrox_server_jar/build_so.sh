#!/bin/bash
# Exit script if any commands fails
set -e

# CONFIG

echo "
[*] Adding aarch64-linux-android as target..."
rustup target add aarch64-linux-android

echo "
[*] Installing cargo-ndk..."
cargo install cargo-ndk

echo "
[*] Going to rust_binder..."
cd rust_binder

echo "
[*] Cleaning cargo ndk..."
cargo ndk clean

echo "
[*] Building cargo ndk"
cargo ndk -t arm64-v8a -o ./target/ --platform 21 build

cd ..

echo "
[✅] librust_binnder.so is ready !!!" 