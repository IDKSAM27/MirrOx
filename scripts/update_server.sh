# #!/bin/bash

# set -e

# # Set output directory
# OUTPUT_DIR="mirrox/server"
# mkdir -p "$OUTPUT_DIR"

# echo "Fetching latest scrcpy version..."

# # Fetch latest version tag using GitHub API
# LATEST_TAG=$(curl -s https://api.github.com/repos/Genymobile/scrcpy/releases/latest | grep -oP ' "tag_name": "\K(.*)(?=")')
# if [ -z "$LATEST_TAG" ]; then
#     echo "Failed to fetch latest version tag."
#     exit 1
# fi

# # Build URL
# JAR_URL="https://github.com/Genymobile/scrcpy/releases/download/${LATEST_TAG}/scrcpy-server-${LATEST_TAG#v}"

# # Output path
# OUTPUT_JAR="${OUTPUT_DIR}/scrcpy-server.jar

# echo "Downloading scrcpy-server from $JAR_URL..."
# curl -L "$JAR_URL" -o "#OUTPUT_JAR"

# echo "Download complete: $OUTPUT_JAR"


#!/bin/bash

set -e

SCRCPY_VERSION=$(curl -s https://api.github.com/repos/Genymobile/scrcpy/releases/latest | grep tag_name | cut -d '"' -f 4)
SCRCPY_BASE_URL="https://github.com/Genymobile/scrcpy/releases/download/${SCRCPY_VERSION}"
JAR_FILENAME="scrcpy-server"

echo "Fetching scrcpy server version: ${SCRCPY_VERSION}"

mkdir -p server

# Download scrcpy-server file
wget -O server/${JAR_FILENAME}.jar "${SCRCPY_BASE_URL}/${JAR_FILENAME}-${SCRCPY_VERSION:1}"

echo "scrcpy-server.jar downloaded to server/"
