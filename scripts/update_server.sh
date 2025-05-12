#!/bin/bash

set -e

# Set output directory
OUTPUT_DIR = "mirrox/server"
mkdir -p "$OUTPUT_DIR"

echo "Fetching latest scrcpy version..."