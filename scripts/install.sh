#!/usr/bin/env bash
# File: install.sh
# Author: Michal Švrček
# LogLens Linux installer.
# ver. 0.6.0

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

SOURCE="${1:-}"

if [[ -z "$SOURCE" ]]; then
    if [[ -f "$SCRIPT_DIR/lens" ]]; then
        SOURCE="$SCRIPT_DIR/lens"
    else
        SOURCE="$PROJECT_ROOT/target/release/lens"
    fi
fi

if [[ ! -f "$SOURCE" ]]; then
    echo "[ERROR] lens binary not found: $SOURCE" >&2
    exit 1
fi

INSTALL_DIR="$HOME/.local/bin"

mkdir -p "$INSTALL_DIR"
install -m 755 "$SOURCE" "$INSTALL_DIR/lens"

echo "[ OK ] Installed: $INSTALL_DIR/lens"
"$INSTALL_DIR/lens" --version

echo "Make sure ~/.local/bin is in PATH."
