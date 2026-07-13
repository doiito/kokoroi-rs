#!/bin/bash
set -e
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

echo "Building Kokoro WASM..."
cd "$PROJECT_DIR"

export CC=/usr/local/llvm-20/bin/clang
export CXX=/usr/local/llvm-20/bin/clang++
export CCACHE_DISABLE=1

wasm-pack build crates/kokoros-core \
    --target web \
    --out-dir ../../static/wasm-pkg \
    -- \
    --features wasm \
    --no-default-features

echo "WASM build complete!"
ls -lh static/wasm-pkg/
