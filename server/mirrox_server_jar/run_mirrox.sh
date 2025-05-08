#!/bin/bash
# Exit script if any command fails
set -e

# === CONFIG ===
ARCH="arm64-v8a"
LIB_NAME="librust_binder.so"
JAR_NAME="mirrox_server.dex.jar"
DEVICE_PATH="/data/local/tmp"

# === STEP 1: Push .so to device ===
echo "
[*] Pushing native .so to device..."
# adb push rust_binder/target/$ARCH/$LIB_NAME $DEVICE_PATH/
adb push rust_binder/target/arm64-v8a/librust_binder.so /data/local/tmp/

# === STEP 2: Push .jar to device ===
echo "
[*] Pushing JAR to device..."
adb push build/dex/$JAR_NAME $DEVICE_PATH/

# === STEP 3: Run the server using app_process ===
echo "
[✅] Launching mirrox_server using app_process..."
adb shell CLASSPATH=$DEVICE_PATH/$JAR_NAME app_process / com.mirrox.server.StartMirrox
