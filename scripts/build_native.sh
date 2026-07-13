#!/bin/bash
set -e
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

echo "Building Kokoro TTS (native)..."
cd "$PROJECT_DIR"

export CC=/usr/bin/gcc
export CXX=/usr/bin/g++
export CCACHE_DISABLE=1

ORT_LIB_DIR="${PROJECT_DIR}/onnxruntime-linux-x64-1.23.0/lib"
if [ -d "$ORT_LIB_DIR" ]; then
    export ORT_LIB_LOCATION="${ORT_LIB_DIR}"
    export LD_LIBRARY_PATH="${ORT_LIB_DIR}:${LD_LIBRARY_PATH}"
    echo "ORT_LIB_LOCATION=$ORT_LIB_LOCATION"
fi

cargo build --release -p koko -p kokoros-server
echo "Build complete!"
ls -lh target/release/koko target/release/kokoros-server
