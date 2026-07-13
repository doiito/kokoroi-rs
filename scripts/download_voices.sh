#!/bin/bash
set -e
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

mkdir -p "$PROJECT_DIR/data"

echo "Downloading voice style data..."
wget -c "https://github.com/thewh1teagle/kokoro-onnx/releases/download/model-files-v1.0/voices-v1.0.bin" \
  -O "$PROJECT_DIR/data/voices-v1.0.bin" 2>/dev/null || \
curl -L -o "$PROJECT_DIR/data/voices-v1.0.bin" \
  "https://github.com/thewh1teagle/kokoro-onnx/releases/download/model-files-v1.0/voices-v1.0.bin"

echo "Voices data downloaded to $PROJECT_DIR/data/"
ls -lh "$PROJECT_DIR/data/"
