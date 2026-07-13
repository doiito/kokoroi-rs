#!/bin/bash
# Build aarch64 (ARM64) musl static binaries (koko + kokoros-server)
# Produces fully static ELF binaries.
# Uses aarch64 ORT with component libs + _deps/ (abseil, protobuf, etc.)
# Circular static lib deps (abseil) resolved via --start-group linker wrapper
#
# Prerequisites (set one of):
#   KOKOROS_OLD_DIR  — path to Kokoros-main directory (contains toolchain + prebuilt ORT)
#   TOOLCHAIN_DIR    — path to aarch64-linux-musl toolchain bin/ parent
#   ORT_MUSL_DIR     — path to aarch64 musl ORT build output (with lib/libonnxruntime.a)
set -e

PROJECT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
TOOLCHAIN_DIR="${TOOLCHAIN_DIR:-${KOKOROS_OLD_DIR}/aarch64-linux-musl-14.2.0/aarch64-linux-musl}"
ORT_MUSL_DIR="${ORT_MUSL_DIR:-${KOKOROS_OLD_DIR}/onnxruntime-aarch64-musl-built}"

if [ ! -d "$TOOLCHAIN_DIR" ]; then
  echo "ERROR: Toolchain dir not found at '$TOOLCHAIN_DIR'"
  echo "Set KOKOROS_OLD_DIR or TOOLCHAIN_DIR to your aarch64 musl toolchain path."
  exit 1
fi
if [ ! -d "$ORT_MUSL_DIR" ]; then
  echo "ERROR: ORT dir not found at '$ORT_MUSL_DIR'"
  echo "Set KOKOROS_OLD_DIR or ORT_MUSL_DIR to your aarch64 ORT build output."
  exit 1
fi

echo "=============================================="
echo "Building aarch64 musl static"
echo "  Toolchain: $TOOLCHAIN_DIR"
echo "  ORT:       $ORT_MUSL_DIR"
echo "=============================================="

export PATH="${TOOLCHAIN_DIR}/bin:${PATH}"
export ORT_LIB_LOCATION="${ORT_MUSL_DIR}"
export CCACHE_DISABLE=1

# Cargo config.toml already sets linker to aarch64-musl-linker.sh wrapper
# which injects --start-group/--end-group for circular abseil deps.
# Unset env vars so cargo uses .cargo/config.toml's linker setting.
unset CC_aarch64_unknown_linux_musl
unset CXX_aarch64_unknown_linux_musl
unset AR_aarch64_unknown_linux_musl
unset RANLIB_aarch64_unknown_linux_musl

# Toolchain library paths for musl libstdc++ etc.
MUSL_CROSS_DIR="$(dirname "$TOOLCHAIN_DIR")"
MUSL_LIB_DIR="${TOOLCHAIN_DIR}/aarch64-linux-musl/lib"
MUSL_GCC_DIR=$(ls -d "${MUSL_CROSS_DIR}/lib/gcc/aarch64-linux-musl/"*/ 2>/dev/null | head -1)
export RUSTFLAGS="-C target-feature=+crt-static -L ${ORT_MUSL_DIR}/lib -L ${MUSL_LIB_DIR} -L ${MUSL_GCC_DIR%/}"

cd "${PROJECT_DIR}"

echo "Building koko (aarch64 musl, no audio)..."
cargo build --release -p koko --target aarch64-unknown-linux-musl --no-default-features

echo "Building kokoros-server (aarch64 musl)..."
cargo build --release -p kokoros-server --target aarch64-unknown-linux-musl --no-default-features

BUILD_DIR="${PROJECT_DIR}/target/aarch64-unknown-linux-musl/release"
echo ""
echo "=============================================="
echo "Build Complete!"
echo "=============================================="
echo ""
echo "Output:"
ls -lh ${BUILD_DIR}/koko ${BUILD_DIR}/kokoros-server 2>/dev/null
echo ""
echo "=== koko ==="
file ${BUILD_DIR}/koko 2>/dev/null || echo "Not built"
echo ""
echo "=== kokoros-server ==="
file ${BUILD_DIR}/kokoros-server 2>/dev/null || echo "Not built"
