#!/bin/bash

set -e

# Set output directory
OUTPUT_DIR = "mirrox/server"
mkdir -p "$OUTPUT_DIR"

echo "Fetching latest scrcpy version..."

# Fetch latest version tag using GitHub API
LATEST_TAG = $(curl -s https://api.github.com/repos/Genymobile/scrcpy/releases/latest | grep -oP ' "tag_name": "\K(.*)(?=")')