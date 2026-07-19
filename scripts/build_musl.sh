#!/bin/bash
# Build x86_64 musl static binaries (koko + kokoros-server)
# Produces fully static ELF binaries with no glibc dependency.
#
# Prerequisites (set one of):
#   KOKOROS_OLD_DIR  — path to Kokoros-main directory (containing toolchain + prebuilt ORT)
#   MUSL_CROSS_DIR   — path to x86_64-linux-musl-cross toolchain root
#   ORT_COMBINED_DIR — path to x86_64 combined libonnxruntime.a directory
set -e

PROJECT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
MUSL_CROSS_DIR="${MUSL_CROSS_DIR:-${KOKOROS_OLD_DIR}/x86_64-linux-musl-cross}"
ORT_COMBINED_DIR="${ORT_COMBINED_DIR:-${KOKOROS_OLD_DIR}/onnxruntime-musl/onnxruntime-v1.23.2-x86_64-linux-musl-static}"

if [ ! -d "$MUSL_CROSS_DIR" ]; then
  echo "ERROR: MUSL_CROSS_DIR not found at '$MUSL_CROSS_DIR'"
  echo "Set KOKOROS_OLD_DIR or MUSL_CROSS_DIR to your Kokoros-main/toolchain path."
  exit 1
fi
if [ ! -d "$ORT_COMBINED_DIR" ]; then
  echo "ERROR: ORT_COMBINED_DIR not found at '$ORT_COMBINED_DIR'"
  echo "Set KOKOROS_OLD_DIR or ORT_COMBINED_DIR to your prebuilt ORT directory."
  exit 1
fi

echo "============================================"
echo "Building x86_64 musl static"
echo "  Toolchain: $MUSL_CROSS_DIR"
echo "  ORT:       $ORT_COMBINED_DIR"
echo "============================================"

export PATH="${MUSL_CROSS_DIR}/bin:${PATH}"
export ORT_LIB_LOCATION="${ORT_COMBINED_DIR}"
export CCACHE_DISABLE=1
export PKG_CONFIG_ALLOW_CROSS=1
export PKG_CONFIG_ALL_STATIC=1

# Cargo config.toml already sets linker to wrapper; env vars override.
# Unset so cargo uses .cargo/config.toml's linker setting.
unset CC_x86_64_unknown_linux_musl
unset CXX_x86_64_unknown_linux_musl
unset AR_x86_64_unknown_linux_musl
unset RANLIB_x86_64_unknown_linux_musl

# Toolchain library paths for musl libstdc++ etc.
MUSL_GCC_DIR=$(ls -d "${MUSL_CROSS_DIR}/lib/gcc/x86_64-linux-musl/"*/ 2>/dev/null | head -1)
MUSL_LIB_DIR="${MUSL_CROSS_DIR}/x86_64-linux-musl/lib"
MUSL_STUBS_DIR="${MUSL_CROSS_DIR}/.."
export RUSTFLAGS="-C target-feature=+crt-static -L ${MUSL_LIB_DIR} -L ${MUSL_GCC_DIR%/} -L ${MUSL_STUBS_DIR} -l static=musl_stubs"

cd "${PROJECT_DIR}"

echo "Building koko (musl, no audio)..."
cargo build --release -p koko --target x86_64-unknown-linux-musl --no-default-features

echo "Building kokoros-server (musl)..."
cargo build --release -p kokoros-server --target x86_64-unknown-linux-musl --no-default-features



BUILD_DIR="${PROJECT_DIR}/target/x86_64-unknown-linux-musl/release"
echo ""
echo "============================================"
echo "Build Complete!"
echo "============================================"
echo ""
echo "Output:"
ls -lh ${BUILD_DIR}/koko ${BUILD_DIR}/kokoros-server
echo ""
echo "=== koko ==="
file ${BUILD_DIR}/koko
ldd ${BUILD_DIR}/koko 2>&1 || echo "(fully static)"
echo ""
echo "=== kokoros-server ==="
file ${BUILD_DIR}/kokoros-server
ldd ${BUILD_DIR}/kokoros-server 2>&1 || echo "(fully static)"
