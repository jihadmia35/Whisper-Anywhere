#!/usr/bin/env bash
set -e

REPO_USER="jihadmia"
REPO_NAME="whisper-anywhere"
TARGET_DIR="$HOME/.local/bin"
CONFIG_DIR="$HOME/.config/whisper-anywhere"

echo "=========================================="
echo "      🎙️ Installing Whisper Anywhere      "
echo "=========================================="

mkdir -p "$TARGET_DIR"
mkdir -p "$CONFIG_DIR"

# 1. Check if cargo is available to build, or download prebuilt binary
if command -v cargo &> /dev/null; then
    echo "📦 Building from source with cargo..."
    cargo build --release
    cp "target/release/whisper-anywhere" "$TARGET_DIR/whisper-anywhere"
else
    echo "⬇️ Downloading latest release binary..."
    LATEST_URL="https://github.com/$REPO_USER/$REPO_NAME/releases/latest/download/whisper-anywhere"
    curl -fsSL "$LATEST_URL" -o "$TARGET_DIR/whisper-anywhere"
fi

chmod +x "$TARGET_DIR/whisper-anywhere"

# 2. Setup default config if not exists
if [ ! -f "$CONFIG_DIR/config.json" ]; then
    echo ""
    read -p "🔑 Enter your free Groq API key: " USER_KEY
    if [ -z "$USER_KEY" ]; then
        USER_KEY="YOUR_GROQ_API_KEY_HERE"
    fi

    cat <<EOF > "$CONFIG_DIR/config.json"
{
  "groq_api_keys": [
    "$USER_KEY"
  ],
  "model": "whisper-large-v3-turbo",
  "language": "",
  "duration": 0
}
EOF
    echo "Config saved to: $CONFIG_DIR/config.json"
fi

# 3. ydotool check for auto-paste
echo ""
echo "🔍 Checking auto-paste helper (ydotool)..."
if ! command -v ydotool &> /dev/null; then
    echo "⚠️ ydotool is not installed. For automatic hands-free paste on Fedora, run:"
    echo "   sudo dnf install ydotool"
    echo "   sudo systemctl enable --now ydotool"
fi

echo ""
echo "=========================================="
echo "      ✅ Installation Complete!           "
echo "=========================================="
echo "Binary location: $TARGET_DIR/whisper-anywhere"
echo ""
echo "Next step: Set your shortcut (Super + X):"
echo "1. Open GNOME Settings -> Keyboard -> Keyboard Shortcuts -> Custom Shortcuts"
echo "2. Add Shortcut: Name='Whisper Anywhere', Command='$TARGET_DIR/whisper-anywhere', Key='Super+X'"
