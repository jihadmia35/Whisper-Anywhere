use notify_rust::Notification;
use std::process::Command;

/// Send a desktop notification using native notify-rust, with fallback to notify-send CLI
pub fn notify(title: &str, body: &str, icon: &str, timeout_ms: u32) {
    let result = Notification::new()
        .summary(title)
        .body(body)
        .icon(icon)
        .timeout(notify_rust::Timeout::Milliseconds(timeout_ms))
        .show();

    if result.is_err() {
        // Fallback to notify-send
        let _ = Command::new("notify-send")
            .arg("-t")
            .arg(timeout_ms.to_string())
            .arg("-i")
            .arg(icon)
            .arg(title)
            .arg(body)
            .status();
    }
}
