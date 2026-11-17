#!/usr/bin/env bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "=========================================="
echo " Building whisper-anywhere (Rust Edition) "
echo "=========================================="

# Check if Rust / Cargo is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust/Cargo is not installed on your system."
    echo ""
    echo "To install Rust, run:"
    echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    echo "  source \$HOME/.cargo/env"
    echo ""
    echo "Or on Fedora:"
    echo "  sudo dnf install cargo rust"
    exit 1
fi

echo "📦 Compiling release binary with optimizations..."
cargo build --release

BIN_SRC="$SCRIPT_DIR/target/release/whisper-anywhere"
TARGET_DIR="$HOME/.local/bin"
CONFIG_DIR="$HOME/.config/whisper-anywhere"

mkdir -p "$TARGET_DIR"
mkdir -p "$CONFIG_DIR"

# Install binary
cp "$BIN_SRC" "$TARGET_DIR/whisper-anywhere"
chmod +x "$TARGET_DIR/whisper-anywhere"

# Copy config.json to both ~/.config and ~/.local/bin
cp "$SCRIPT_DIR/config.json" "$TARGET_DIR/config.json"
if [ ! -f "$CONFIG_DIR/config.json" ]; then
    cp "$SCRIPT_DIR/config.json" "$CONFIG_DIR/config.json"
fi

echo ""
echo "✅ Build & Installation Successful!"
echo "Binary:  $TARGET_DIR/whisper-anywhere"
echo "Config:  $CONFIG_DIR/config.json"
echo ""
echo "Press Super+X to speak, press Super+X again to paste!"
