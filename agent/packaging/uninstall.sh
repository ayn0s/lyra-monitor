#!/usr/bin/env bash
set -euo pipefail

UNIT_NAME="lyra-agent.service"
UNIT_DEST="/etc/systemd/system/$UNIT_NAME"
BIN_DEST="/usr/local/bin/lyra-agent"

if [[ $EUID -ne 0 ]]; then
  echo "This script must be run as root (sudo $0)." >&2
  exit 1
fi

echo "==> Stopping and disabling the service"
systemctl disable --now "$UNIT_NAME" 2>/dev/null || true

echo "==> Removing files"
rm -f "$UNIT_DEST" "$BIN_DEST"

echo "==> Reloading systemd"
systemctl daemon-reload

echo "Agent uninstalled."
