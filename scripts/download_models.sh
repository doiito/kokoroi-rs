#!/bin/bash
set -e
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

mkdir -p "$PROJECT_DIR/models"

echo "Downloading Kokoro v1.1-zh ONNX models..."
RELEASE_URL="https://github.com/doiito/kokoroi-rs/releases/download/model-files-v1.1"

# Download recommended model (M - int8, 79MB)
wget -c "${RELEASE_URL}/kokoro-v1.1-zh-m.onnx" \
  -O "$PROJECT_DIR/models/kokoro-v1.1-zh-m.onnx" 2>/dev/null || \
curl -L -o "$PROJECT_DIR/models/kokoro-v1.1-zh-m.onnx" \
  "${RELEASE_URL}/kokoro-v1.1-zh-m.onnx"

# Also download S model (int4, 47MB)
wget -c "${RELEASE_URL}/kokoro-v1.1-zh-s.onnx" \
  -O "$PROJECT_DIR/models/kokoro-v1.1-zh-s.onnx" 2>/dev/null || \
curl -L -o "$PROJECT_DIR/models/kokoro-v1.1-zh-s.onnx" \
  "${RELEASE_URL}/kokoro-v1.1-zh-s.onnx"

echo "Models downloaded to $PROJECT_DIR/models/"
ls -lh "$PROJECT_DIR/models/"
