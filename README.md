# 🎙️ Whisper Anywhere

> **Voice dictation anywhere on Linux powered by Groq Whisper (`whisper-large-v3-turbo`) with automatic multi-API-key failover and hands-free auto-paste.**

---

## ✨ Highlights

- ⚡ **Near-Instant Transcription**: Groq LPUs transcribe voice audio in roughly 0.2 to 0.4 seconds.
- 🎯 **Hands-Free Auto-Paste**: Immediately inputs text into the active window (`Ctrl+V`) without manual intervention.
- 🔄 **Smart Key Rotation & Auto-Failover**: Support for multiple free Groq API keys. If one key reaches rate limits, the tool rotates to the next key automatically.
- 🔕 **Minimal & Distraction-Free**: Runs quietly without notification spam.
- 🦀 **Single Native Binary**: Written in Rust for minimal memory usage and rapid startup.

---

## 🎁 Free Tier Usage & Capacity

Groq offers a generous free tier for speech-to-text without requiring a credit card:

- **2,000 requests per day** per API key
- **20 requests per minute** per key
- **8 hours of audio per day** per key

### Scaling with Multiple Keys
Because `whisper-anywhere` rotates across all configured keys when hitting rate limits:
- **1 Key**: 2,000 dictations / 8 hours of audio daily (Free)
- **3 Keys**: 6,000 dictations / 24 hours of audio daily (Free)
- **5 Keys**: 10,000 dictations / 40 hours of audio daily (Free)

For everyday writing, coding, and chat, this is effectively unlimited.

---

## 🚀 Getting Started

### 1. Build and Install
Clone the repository and run the build script:

```bash
git clone https://github.com/jihadmia35/Whisper-Anywhere.git
cd Whisper-Anywhere
./build.sh
```

This compiles the release binary and installs it to `~/.local/bin/whisper-anywhere`.

---

### 2. Add Your Groq API Key
Get a free API key from the Groq console (https://console.groq.com/keys) and run:

```bash
whisper-anywhere --add-key gsk_your_key_here
```

You can run this command multiple times to add backup keys to your pool.

---

### 3. Bind Global Shortcut (`Super + X`)

In GNOME (Fedora / Ubuntu / Debian):
1. Go to **Settings** -> **Keyboard** -> **Keyboard Shortcuts** -> **View and Customize Shortcuts**.
2. Select **Custom Shortcuts** and click **+** (Add Shortcut):
   - **Name**: `Whisper Anywhere`
   - **Command**: `/home/YOUR_USER/.local/bin/whisper-anywhere`
   - **Shortcut**: `Super + X`
3. Click **Add**.

---

### 4. Enable Hands-Free Auto-Paste on Wayland

Fedora and modern Linux distributions use Wayland. To allow automated key simulation via `ydotool`:

```bash
sudo dnf install ydotool
sudo systemctl enable --now ydotool
sudo mkdir -p /etc/systemd/system/ydotool.service.d/
sudo tee /etc/systemd/system/ydotool.service.d/override.conf << 'EOF'
[Service]
ExecStart=
ExecStart=/usr/bin/ydotoold --socket-perm 0666
EOF
sudo systemctl daemon-reload
sudo systemctl restart ydotool
```

*(If ydotool is not installed, the transcribed text is still copied to your clipboard, allowing you to paste manually with `Ctrl+V`)*.

---

## ⚙️ Configuration Reference

Configuration is stored at `~/.config/whisper-anywhere/config.json`:

```json
{
  "groq_api_keys": [
    "gsk_primary_key_here",
    "gsk_backup_key_here"
  ],
  "model": "whisper-large-v3-turbo",
  "language": "",
  "duration": 0
}
```

- `groq_api_keys`: Array of Groq API keys used in round-robin failover.
- `model`: Model name (`whisper-large-v3-turbo` for speed, or `whisper-large-v3`).
- `language`: Two-letter language code (e.g., `en`, `bn`, `es`), or empty for auto-detect.
- `duration`: Set to `0` for toggle mode (press to start, press to stop).
