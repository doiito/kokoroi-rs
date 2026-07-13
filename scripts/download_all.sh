#!/bin/bash
set -e
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
bash "$SCRIPT_DIR/download_models.sh"
bash "$SCRIPT_DIR/download_voices.sh"
