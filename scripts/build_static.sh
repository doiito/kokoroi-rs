#!/bin/bash
set -e
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

echo "Building Kokoro TTS (Fully Static Linked)..."
cd "$PROJECT_DIR"

export CC=/usr/local/gcc-14.2.0/bin/gcc
export CXX=/usr/local/gcc-14.2.0/bin/g++
export AR=/usr/local/gcc-14.2.0/bin/gcc-ar
export RANLIB=/usr/local/gcc-14.2.0/bin/gcc-ranlib
export CCACHE_DISABLE=1

export LD_LIBRARY_PATH=/usr/local/gcc-14.2.0/lib64:$LD_LIBRARY_PATH
export LIBRARY_PATH=/usr/local/gcc-14.2.0/lib64:/usr/local/gcc-14.2.0/lib/gcc/x86_64-pc-linux-gnu/14.2.0:$LIBRARY_PATH
export ORT_LIB_LOCATION="${PROJECT_DIR}/onnxruntime"
export RUSTFLAGS="-C link-arg=-L/usr/local/gcc-14.2.0/lib64 \
  -C link-arg=-L/usr/local/gcc-14.2.0/lib/gcc/x86_64-pc-linux-gnu/14.2.0 \
  -C link-arg=-Wl,-Bstatic -C link-arg=-lstdc++ -C link-arg=-lgcc \
  -C link-arg=-Wl,-Bdynamic"

cargo build --release -p koko -p kokoros-server
echo "Build complete!"
ls -lh target/release/koko target/release/kokoros-server
