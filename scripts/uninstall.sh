#!/usr/bin/env bash
# File: uninstall.sh
# Author: Michal Švrček
# LogLens Linux uninstaller.
# ver. 0.6.0

set -euo pipefail

rm -f -- "$HOME/.local/bin/lens"

echo "[ OK ] LogLens uninstalled."
echo "Device profiles were preserved."
