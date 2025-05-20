#!/bin/bash
set -e

LATEST_TAG=$(curl -s https://api.github.com/repos/Genymobile/scrcpy/releases/latest | grep '"tag_name":' | cut -d '"' -f 4)
echo "Latest version: $LATEST_TAG"

SERVER_URL="https://github.com/Genymobile/scrcpy/releases/download/$LATEST_TAG/scrcpy-server"
OUT_DIR="server"
OUT_PATH="$OUT_DIR/scrcpy-server-$LATEST_TAG"

mkdir -p "$OUT_DIR"
curl -sSL -o "$OUT_PATH" "$SERVER_URL"

# Check if file exists and is not empty
if [ ! -s "$OUT_PATH" ]; then
    echo "❌ Download failed: File is empty or missing"
    rm -f "$OUT_PATH"
    exit 1
fi

# Save version to file
echo "$LATEST_TAG" > "$OUT_DIR/version.txt"

echo "[*] Downloaded scrcpy-server-$LATEST_TAG"
