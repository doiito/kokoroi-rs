#!/bin/bash
# Wrapper for x86_64-linux-musl-gcc that adds --start-group/--end-group
# to resolve circular static library dependencies (abseil, etc.)
#
# Toolchain resolution order:
#   1. $MUSL_TOOLCHAIN_GCC  — explicit full path to gcc
#   2. $MUSL_TOOLCHAIN_PATH — toolchain root (appends bin/x86_64-linux-musl-gcc)
#   3. PATH                 — x86_64-linux-musl-gcc found in PATH

TOOLCHAIN_GCC=""

# --- env var: explicit ---
if [ -n "${MUSL_TOOLCHAIN_GCC:-}" ] && [ -x "$MUSL_TOOLCHAIN_GCC" ]; then
    TOOLCHAIN_GCC="$MUSL_TOOLCHAIN_GCC"
# --- env var: root path ---
elif [ -n "${MUSL_TOOLCHAIN_PATH:-}" ] && [ -x "${MUSL_TOOLCHAIN_PATH}/bin/x86_64-linux-musl-gcc" ]; then
    TOOLCHAIN_GCC="${MUSL_TOOLCHAIN_PATH}/bin/x86_64-linux-musl-gcc"
# --- PATH ---
elif command -v x86_64-linux-musl-gcc &>/dev/null; then
    TOOLCHAIN_GCC="$(command -v x86_64-linux-musl-gcc)"
else
    echo "ERROR: x86_64-linux-musl-gcc not found. Set MUSL_TOOLCHAIN_GCC or MUSL_TOOLCHAIN_PATH." >&2
    exit 1
fi

exec "$TOOLCHAIN_GCC" -Wl,--start-group "$@" -Wl,--end-group
