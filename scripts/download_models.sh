#!/bin/bash
set -e
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

mkdir -p "$PROJECT_DIR/models"

echo "Downloading Kokoro ONNX model..."
wget -c "https://github.com/thewh1teagle/kokoro-onnx/releases/download/model-files-v1.0/kokoro-v1.0.onnx" \
  -O "$PROJECT_DIR/models/kokoro-v1.0.onnx" 2>/dev/null || \
curl -L -o "$PROJECT_DIR/models/kokoro-v1.0.onnx" \
  "https://github.com/thewh1teagle/kokoro-onnx/releases/download/model-files-v1.0/kokoro-v1.0.onnx"

echo "Model downloaded to $PROJECT_DIR/models/"
ls -lh "$PROJECT_DIR/models/"
