use arboard::Clipboard;
use std::env;
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

fn command_exists(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Copies text to clipboard and simulates Ctrl+V
pub fn copy_and_paste(text: &str) -> bool {
    println!("📝 Transcribed text: \"{}\"", text);

    // 1. On Wayland, ALWAYS invoke wl-copy because it forks into the background
    // to keep the clipboard data alive after our CLI process exits.
    if command_exists("wl-copy") {
        let _ = Command::new("wl-copy").arg(text).status();
        let _ = Command::new("wl-copy").arg("--primary").arg(text).status();
    }

    // Also populate arboard for X11 / cross-platform
    if let Ok(mut clip) = Clipboard::new() {
        let _ = clip.set_text(text.to_owned());
    }

    // 2. Wait 250ms for clipboard to register and user to release physical hotkey
    thread::sleep(Duration::from_millis(250));

    // 3. Simulate Ctrl+V to paste at active cursor
    let pasted = simulate_paste();

    // 4. Brief delay so target window completes reading the clipboard before exit
    thread::sleep(Duration::from_millis(200));

    pasted
}

/// Attempt to simulate Ctrl+V via available input tools
fn simulate_paste() -> bool {
    // 1. Try ydotool (Standard for Wayland)
    if command_exists("ydotool") {
        let mut cmd = Command::new("ydotool");

        // -d 25: 25ms delay between key events (realistic physical keypress)
        // 125:0: release Super (just in case)
        // 29:1: press LeftCtrl
        // 47:1: press V
        // 47:0: release V
        // 29:0: release LeftCtrl
        cmd.args(["key", "-d", "25", "125:0", "29:1", "47:1", "47:0", "29:0"]);

        // Check common ydotool socket paths
        let candidate_sockets = [
            "/tmp/.ydotool_socket",
            &format!("/run/user/{}/.ydotool_socket", env::var("UID").unwrap_or_else(|_| "1000".to_string())),
        ];

        for sock in &candidate_sockets {
            if Path::new(sock).exists() {
                cmd.env("YDOTOOL_SOCKET", sock);
                break;
            }
        }
        if env::var("YDOTOOL_SOCKET").is_err() {
            cmd.env("YDOTOOL_SOCKET", "/tmp/.ydotool_socket");
        }

        match cmd.output() {
            Ok(output) => {
                if output.status.success() {
                    return true;
                } else {
                    let err = String::from_utf8_lossy(&output.stderr);
                    eprintln!("⚠️ ydotool error: {}", err.trim());
                }
            }
            Err(e) => {
                eprintln!("⚠️ Failed to execute ydotool: {}", e);
            }
        }
    }

    // 2. Fallback: xdotool (for X11 / Xwayland)
    if command_exists("xdotool") {
        if let Ok(output) = Command::new("xdotool").args(["key", "--clearmodifiers", "ctrl+v"]).output() {
            if output.status.success() {
                return true;
            }
        }
    }

    // 3. Fallback: wtype (for wlroots Wayland compositors)
    if command_exists("wtype") {
        if let Ok(output) = Command::new("wtype").args(["-M", "ctrl", "-k", "v", "-m", "ctrl"]).output() {
            if output.status.success() {
                return true;
            }
        }
    }

    false
}
