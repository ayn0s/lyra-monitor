#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN_NAME="lyra-agent"
INSTALL_DIR="/usr/local/bin"
UNIT_NAME="lyra-agent.service"
UNIT_DEST="/etc/systemd/system/$UNIT_NAME"

if [[ $EUID -ne 0 ]]; then
  echo "This script must be run as root (sudo $0)." >&2
  exit 1
fi

if [[ -f "$SCRIPT_DIR/$BIN_NAME" && -f "$SCRIPT_DIR/$UNIT_NAME" ]]; then
  BUILT_BIN="$SCRIPT_DIR/$BIN_NAME"
  UNIT_SRC="$SCRIPT_DIR/$UNIT_NAME"
else
  REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
  BUILT_BIN="$REPO_ROOT/target/release/$BIN_NAME"
  UNIT_SRC="$REPO_ROOT/agent/packaging/$UNIT_NAME"

  for cargo_home in "$HOME/.cargo/bin" "${SUDO_USER:+/home/$SUDO_USER/.cargo/bin}"; do
    [[ -n "$cargo_home" && -d "$cargo_home" ]] && PATH="$cargo_home:$PATH"
  done
  export PATH

  ensure_build_deps() {
    command -v protoc >/dev/null 2>&1 && command -v clang >/dev/null 2>&1 && return

    if command -v apt-get >/dev/null 2>&1; then
      echo "==> Installing build dependencies (protobuf-compiler, clang, libpam0g-dev)"
      apt-get update
      apt-get install -y protobuf-compiler clang libclang-dev libpam0g-dev pkg-config
    else
      echo "Missing build dependencies: protobuf-compiler, clang, libclang-dev, libpam0g-dev." >&2
      echo "Install them with your distribution's package manager, then rerun this script." >&2
      exit 1
    fi
  }

  if [[ ! -x "$BUILT_BIN" ]]; then
    if ! command -v cargo >/dev/null 2>&1; then
      echo "cargo not found. Install Rust (https://rustup.rs) then rerun this script." >&2
      exit 1
    fi

    ensure_build_deps

    echo "==> Building the agent (cargo build --release -p agent)"
    (cd "$REPO_ROOT" && cargo build --release -p agent)
  fi
fi

echo "==> Stopping the existing service (if any)"
systemctl stop "$UNIT_NAME" 2>/dev/null || true

echo "==> Installing the binary to $INSTALL_DIR/$BIN_NAME"
install -m 755 "$BUILT_BIN" "$INSTALL_DIR/$BIN_NAME"

echo "==> Installing the systemd unit $UNIT_NAME"
install -m 644 "$UNIT_SRC" "$UNIT_DEST"

echo "==> Reloading systemd"
systemctl daemon-reload

echo "==> Enabling and starting the service"
systemctl enable --now "$UNIT_NAME"

sleep 1
systemctl --no-pager --full status "$UNIT_NAME" || true

cat <<EOF

Agent installed and started.

  Status   : systemctl status $UNIT_NAME
  Logs     : journalctl -u $UNIT_NAME -f
  Stop     : sudo systemctl stop $UNIT_NAME
  Remove   : sudo ./uninstall.sh
EOF
