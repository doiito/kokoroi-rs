#!/bin/bash
set -e
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

mkdir -p "$PROJECT_DIR/data"

echo "Downloading voice style data..."
RELEASE_URL="https://github.com/doiito/kokoroi-rs/releases/download/model-files-v1.1"

wget -c "${RELEASE_URL}/voices-v1.0.bin" \
  -O "$PROJECT_DIR/data/voices-v1.0.bin" 2>/dev/null || \
curl -L -o "$PROJECT_DIR/data/voices-v1.0.bin" \
  "${RELEASE_URL}/voices-v1.0.bin"

echo "Voices data downloaded to $PROJECT_DIR/data/"
ls -lh "$PROJECT_DIR/data/"
