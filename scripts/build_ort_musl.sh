#!/bin/bash
# Build ONNX Runtime from source for musl targets, produce combined libonnxruntime.a.
# Usage:
#   ./build_ort_musl.sh <target> <toolchain_dir> <ort_source> <install_dir>
#
# Examples:
#   ./build_ort_musl.sh x86_64-unknown-linux-musl /opt/x86_64-linux-musl-cross /opt/onnxruntime-1.23.2 /opt/ort-x86_64
#   ./build_ort_musl.sh aarch64-unknown-linux-musl /opt/aarch64-linux-musl-cross /opt/onnxruntime-1.23.2 /opt/ort-aarch64
set -euo pipefail

if [ $# -lt 4 ]; then
    echo "Usage: $0 <target> <toolchain_dir> <ort_source> <install_dir>"
    echo "  target:       x86_64-unknown-linux-musl | aarch64-unknown-linux-musl"
    echo "  toolchain_dir: musl-cross toolchain root (parent of bin/)"
    echo "  ort_source:    ONNX Runtime source tree (parent of cmake/)"
    echo "  install_dir:   output directory for combined libonnxruntime.a + headers"
    exit 1
fi

TARGET="$1"
TOOLCHAIN="$2"
ORT_SRC="$3"
INSTALL_DIR="$4"

# Map Rust target -> musl-cross toolchain name
case "$TARGET" in
    x86_64-unknown-linux-musl)
        TOOLCHAIN_NAME="x86_64-linux-musl"
        ARCH_CFLAGS="-march=x86-64-v2 -mtune=generic -O2 -pipe"
        NEED_MUSL_STUBS=1
        ;;
    aarch64-unknown-linux-musl)
        TOOLCHAIN_NAME="aarch64-linux-musl"
        ARCH_CFLAGS="-march=armv8-a -mtune=cortex-a72 -O2 -pipe"
        NEED_MUSL_STUBS=0
        ;;
    *)
        echo "Unsupported target: $TARGET"
        exit 1
        ;;
esac

if [ ! -d "$ORT_SRC/cmake" ]; then
    echo "ERROR: ONNX Runtime source not found at $ORT_SRC (missing cmake/)"
    exit 1
fi

if [ ! -d "$TOOLCHAIN/bin" ]; then
    echo "ERROR: musl toolchain not found at $TOOLCHAIN (missing bin/)"
    exit 1
fi

CC="${TOOLCHAIN}/bin/${TOOLCHAIN_NAME}-gcc"
CXX="${TOOLCHAIN}/bin/${TOOLCHAIN_NAME}-g++"
AR="${TOOLCHAIN}/bin/${TOOLCHAIN_NAME}-ar"
RANLIB="${TOOLCHAIN}/bin/${TOOLCHAIN_NAME}-ranlib"

for tool in "$CC" "$CXX" "$AR" "$RANLIB"; do
    if [ ! -x "$tool" ]; then
        echo "ERROR: Tool not found or not executable: $tool"
        exit 1
    fi
done

BUILD_DIR="${ORT_SRC}/build-musl-${TARGET}"
mkdir -p "$BUILD_DIR"
mkdir -p "$INSTALL_DIR/include" "$INSTALL_DIR/lib"

NUM_JOBS="${NUM_JOBS:-$(nproc)}"

echo "========================================================"
echo "Building ONNX Runtime for $TARGET"
echo "  Toolchain:  $TOOLCHAIN"
echo "  Source:     $ORT_SRC"
echo "  Build:      $BUILD_DIR"
echo "  Install:    $INSTALL_DIR"
echo "  Jobs:       $NUM_JOBS"
echo "========================================================"

# Write CMake toolchain file -------------------------------------------------
cat > "/tmp/ort-toolchain-${TARGET//\//-}.cmake" << 'CMAKE_EOF'
set(CMAKE_SYSTEM_NAME Linux)
if(DEFINED ENV{TOOLCHAIN_NAME})
    if($ENV{TOOLCHAIN_NAME} STREQUAL "aarch64-linux-musl" )
        set(CMAKE_SYSTEM_PROCESSOR aarch64)
    elseif($ENV{TOOLCHAIN_NAME} STREQUAL "x86_64-linux-musl" )
        set(CMAKE_SYSTEM_PROCESSOR x86_64)
    else()
        message(FATAL_ERROR "TOOLCHAIN_NAME: $ENV{TOOLCHAIN_NAME} not supported!")
    endif()
endif()
set(TOOLCHAIN_NAME $ENV{TOOLCHAIN_NAME} CACHE STRING "toolchain name")
file(TO_CMAKE_PATH $ENV{TOOLCHAIN_PATH} TOOLCHAIN_PATH)
set(TOOLCHAIN_PATH ${TOOLCHAIN_PATH} CACHE STRING "root path to toolchain")
set(CMAKE_C_COMPILER "${TOOLCHAIN_PATH}/bin/${TOOLCHAIN_NAME}-gcc")
set(CMAKE_CXX_COMPILER "${TOOLCHAIN_PATH}/bin/${TOOLCHAIN_NAME}-g++")
set(CMAKE_LINKER "${TOOLCHAIN_PATH}/bin/${TOOLCHAIN_NAME}-ld")
set(CMAKE_OBJDUMP "${TOOLCHAIN_PATH}/bin/${TOOLCHAIN_NAME}-objdump")
set(CMAKE_AR "${TOOLCHAIN_PATH}/bin/${TOOLCHAIN_NAME}-ar")
set(CMAKE_NM "${TOOLCHAIN_PATH}/bin/${TOOLCHAIN_NAME}-nm")
set(CMAKE_RANLIB "${TOOLCHAIN_PATH}/bin/${TOOLCHAIN_NAME}-ranlib")
set(CMAKE_FIND_ROOT_PATH_MODE_PROGRAM NEVER)
set(CMAKE_FIND_ROOT_PATH_MODE_LIBRARY ONLY)
set(CMAKE_FIND_ROOT_PATH_MODE_INCLUDE ONLY)
set(CMAKE_FIND_ROOT_PATH_MODE_PACKAGE ONLY)
CMAKE_EOF

export TOOLCHAIN_NAME="${TOOLCHAIN_NAME}"
export TOOLCHAIN_PATH="${TOOLCHAIN}"

# Add toolchain to PATH for cmake find_program
export PATH="${TOOLCHAIN}/bin:${PATH}"

# Detect available cmake generator
GENERATOR="Unix Makefiles"
if command -v ninja &>/dev/null; then
    GENERATOR="Ninja"
fi

echo ""
echo "--- Configuring ORT ---"

# Minimal ORT build: CPU inference only, no backends, no python, no tests
cmake -S "$ORT_SRC/cmake" -B "$BUILD_DIR" \
    -G "$GENERATOR" \
    -DCMAKE_TOOLCHAIN_FILE="/tmp/ort-toolchain-${TARGET//\//-}.cmake" \
    -DCMAKE_BUILD_TYPE=Release \
    -DCMAKE_INSTALL_PREFIX="$INSTALL_DIR" \
    -DCMAKE_C_FLAGS="$ARCH_CFLAGS" \
    -DCMAKE_CXX_FLAGS="$ARCH_CFLAGS" \
    -DCMAKE_CXX_STANDARD=17 \
    -DCMAKE_POSITION_INDEPENDENT_CODE=ON \
    -DPROTOBUF_PROTOC_EXECUTABLE="/usr/bin/protoc" \
    -Dprotobuf_BUILD_TESTS=OFF \
    -Donnxruntime_ENABLE_PYTHON=OFF \
    -Donnxruntime_BUILD_SHARED_LIB=OFF \
    -Donnxruntime_BUILD_UNIT_TESTS=OFF \
    -Donnxruntime_RUN_ONNX_TESTS=OFF \
    -Donnxruntime_ENABLE_LTO=ON \
    -Donnxruntime_USE_CUDA=OFF \
    -Donnxruntime_USE_DNNL=OFF \
    -Donnxruntime_USE_TENSORRT=OFF \
    -Donnxruntime_USE_OPENVINO=OFF \
    -Donnxruntime_USE_COREML=OFF \
    -Donnxruntime_USE_NNAPI=OFF \
    -Donnxruntime_USE_RKNPU=OFF \
    -Donnxruntime_USE_JSEP=OFF \
    -Donnxruntime_USE_WEBGPU=OFF \
    -Donnxruntime_USE_WEBNN=OFF \
    -Donnxruntime_USE_XNNPACK=OFF \
    -Donnxruntime_USE_TELEMETRY=OFF \
    -Donnxruntime_USE_WINML=OFF \
    -Donnxruntime_USE_OPENBLAS=OFF \
    -Donnxruntime_USE_BLAS=OFF \
    -Donnxruntime_USE_MPI=OFF \
    -Donnxruntime_USE_OPENMP=OFF \
    -Donnxruntime_DISABLE_ML_OPS=ON \
    -Donnxruntime_DISABLE_RTTI=ON \
    -Donnxruntime_MINIMAL_BUILD=ON \
    -Donnxruntime_EXTENDED_MINIMAL_BUILD=ON

echo ""
echo "--- Building ORT ---"
cmake --build "$BUILD_DIR" --config Release -j "$NUM_JOBS"

echo ""
echo "--- Installing ORT ---"
cmake --install "$BUILD_DIR" --config Release

echo ""
echo "--- Creating combined libonnxruntime.a ---"

# Find all .a files in the build directory
mapfile -t A_LIBS < <(find "$BUILD_DIR" -name "*.a" -type f 2>/dev/null | sort)

if [ ${#A_LIBS[@]} -eq 0 ]; then
    echo "WARNING: No .a files found in build dir; install may have already placed combined lib."
    echo "Checking ${INSTALL_DIR}/lib/..."
    if [ -f "$INSTALL_DIR/lib/libonnxruntime.a" ]; then
        echo "Combined lib at $INSTALL_DIR/lib/libonnxruntime.a ($(du -h "$INSTALL_DIR/lib/libonnxruntime.a" | cut -f1))"
    else
        echo "ERROR: No libonnxruntime.a found in install dir either."
        find "$BUILD_DIR" -name "*.a" -maxdepth 3
        exit 1
    fi
else
    echo "Found ${#A_LIBS[@]} static libs in build output"

    # Create MRI script for combining
    MRI_SCRIPT="/tmp/ort-combine-${TARGET//\//-}.mri"
    echo "create ${INSTALL_DIR}/lib/libonnxruntime.combined.a" > "$MRI_SCRIPT"

    # Check if build already produced libonnxruntime.a directly
    HAS_DIRECT_LIB=false
    DIRECT_LIB=""
    for lib in "${A_LIBS[@]}"; do
        basename "$lib"
        if [[ "$(basename "$lib")" == "libonnxruntime.a" ]]; then
            HAS_DIRECT_LIB=true
            DIRECT_LIB="$lib"
        fi
    done

    # If there's a direct libonnxruntime.a, it's already the combined one
    # We still need to add all leftover .a files because the cmake target
    # might not have merged everything
    if [ "$HAS_DIRECT_LIB" = true ]; then
        # Use direct lib as base, add remaining libs
        echo "addlib $DIRECT_LIB" >> "$MRI_SCRIPT"
        for lib in "${A_LIBS[@]}"; do
            if [ "$lib" != "$DIRECT_LIB" ]; then
                echo "addlib $lib" >> "$MRI_SCRIPT"
            fi
        done
    else
        for lib in "${A_LIBS[@]}"; do
            echo "addlib $lib" >> "$MRI_SCRIPT"
        done
    fi

    # For x86_64: compile musl_stubs and include in combined lib
    if [ "$NEED_MUSL_STUBS" = 1 ]; then
        echo ""
        echo "--- Compiling musl_stubs (x86_64) ---"
        MUSL_STUBS_C="${ORT_SRC}/musl_stubs.c"
        MUSL_STUBS_O="${BUILD_DIR}/musl_stubs.o"
        cat > "$MUSL_STUBS_C" << 'STUBS_EOF'
#include <stdint.h>
uint64_t __cpu_features2 = 0;
STUBS_EOF
        "$CC" $ARCH_CFLAGS -c "$MUSL_STUBS_C" -o "$MUSL_STUBS_O"
        echo "addmod $MUSL_STUBS_O" >> "$MRI_SCRIPT"
    fi

    echo "save" >> "$MRI_SCRIPT"
    echo "end" >> "$MRI_SCRIPT"

    echo "MRI script:"
    cat "$MRI_SCRIPT"

    # Run ar to combine all libs
    "$AR" -M < "$MRI_SCRIPT"

    # Replace with proper name
    if [ -f "${INSTALL_DIR}/lib/libonnxruntime.combined.a" ]; then
        mv "${INSTALL_DIR}/lib/libonnxruntime.combined.a" "${INSTALL_DIR}/lib/libonnxruntime.a"
    fi

    # Verify
    if [ ! -f "${INSTALL_DIR}/lib/libonnxruntime.a" ]; then
        echo "ERROR: Combined lib was not created"
        exit 1
    fi
fi

echo ""
echo "--- Verifying output ---"
echo "Headers:"
ls -la "$INSTALL_DIR/include/onnxruntime/"*.h 2>/dev/null || ls -la "$INSTALL_DIR/include/"*.h 2>/dev/null || echo "(headers not in standard location)"

echo ""
echo "Combined lib:"
ls -lh "${INSTALL_DIR}/lib/libonnxruntime.a"

echo ""
echo "Targets in combined lib:"
"$AR" t "${INSTALL_DIR}/lib/libonnxruntime.a" 2>/dev/null | head -20
echo "... ($("$AR" t "${INSTALL_DIR}/lib/libonnxruntime.a" 2>/dev/null | wc -l) total objects)"

# Create symlink at root level for ort-sys to find
ln -sf "lib/libonnxruntime.a" "${INSTALL_DIR}/libonnxruntime.a"

echo ""
echo "========================================================"
echo "ONNX Runtime for $TARGET built successfully!"
echo "Install dir: $INSTALL_DIR"
echo "Set ORT_LIB_LOCATION=$INSTALL_DIR"
echo "========================================================"
