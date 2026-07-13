#!/bin/bash
set -e
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
cd "$PROJECT_DIR"

VERSION=${1:-"0.1.0"}
DIST_BASE="dist"

echo "Creating distribution packages v${VERSION}..."
rm -rf "${DIST_BASE}/kokoros-${VERSION}"
mkdir -p "${DIST_BASE}/kokoros-${VERSION}"/{models,data,lib}

# Copy binaries
cp target/release/koko "${DIST_BASE}/kokoros-${VERSION}/"
cp target/release/kokoros-server "${DIST_BASE}/kokoros-${VERSION}/"

# Bundle GCC libraries for static builds
GCC_LIB="/usr/local/gcc-14.2.0/lib64"
for lib in libstdc++.so.6.0.33 libgcc_s.so.1; do
    [ -f "$GCC_LIB/$lib" ] && cp "$GCC_LIB/$lib" "${DIST_BASE}/kokoros-${VERSION}/lib/"
done
ln -sf libstdc++.so.6.0.33 "${DIST_BASE}/kokoros-${VERSION}/lib/libstdc++.so.6" 2>/dev/null || true

# Set RPATH
if command -v patchelf &> /dev/null; then
    patchelf --set-rpath '$ORIGIN/lib' --force-rpath "${DIST_BASE}/kokoros-${VERSION}/koko" 2>/dev/null || true
    patchelf --set-rpath '$ORIGIN/lib' --force-rpath "${DIST_BASE}/kokoros-${VERSION}/kokoros-server" 2>/dev/null || true
fi

# Copy config
cat > "${DIST_BASE}/kokoros-${VERSION}/config.toml" << EOF
host = "0.0.0.0"
port = 3000
threads = 2
model_path = "models/kokoro-v1.0.onnx"
voices_path = "data/voices-v1.0.bin"
max_chars = 400
EOF

# Copy model and voices if they exist
[ -f "models/kokoro-v1.0.onnx" ] && cp "models/kokoro-v1.0.onnx" "${DIST_BASE}/kokoros-${VERSION}/models/"
[ -f "data/voices-v1.0.bin" ] && cp "data/voices-v1.0.bin" "${DIST_BASE}/kokoros-${VERSION}/data/"

# Create tarball
cd "${DIST_BASE}"
tar -czvf "kokoros-${VERSION}-linux-x64.tar.gz" "kokoros-${VERSION}"
cd ..
echo "Package created: ${DIST_BASE}/kokoros-${VERSION}-linux-x64.tar.gz"
