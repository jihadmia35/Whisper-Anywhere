use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize)]
pub struct RecordingState {
    pub pid: u32,
    pub audio_path: PathBuf,
    pub start_time: u64,
}

pub fn get_state_path() -> PathBuf {
    env::temp_dir().join("whisper_anywhere_state.json")
}

pub fn get_audio_path() -> PathBuf {
    env::temp_dir().join("whisper_anywhere_audio.wav")
}

pub fn is_pid_alive(pid: u32) -> bool {
    Command::new("kill")
        .arg("-0")
        .arg(pid.to_string())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn command_exists(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Start background audio recording (First toggle press)
pub fn start_recording() -> Result<(), String> {
    let audio_path = get_audio_path();
    let state_path = get_state_path();

    // Remove previous audio file if exists
    if audio_path.exists() {
        let _ = fs::remove_file(&audio_path);
    }

    let mut cmd = if command_exists("pw-record") {
        let mut c = Command::new("pw-record");
        c.arg("--channels=1").arg("--rate=16000").arg(&audio_path);
        c
    } else if command_exists("ffmpeg") {
        let mut c = Command::new("ffmpeg");
        c.arg("-y")
            .arg("-f")
            .arg("pulse")
            .arg("-i")
            .arg("default")
            .arg("-ar")
            .arg("16000")
            .arg("-ac")
            .arg("1")
            .arg(&audio_path);
        c
    } else if command_exists("arecord") {
        let mut c = Command::new("arecord");
        c.arg("-q").arg("-f").arg("cd").arg("-t").arg("wav").arg(&audio_path);
        c
    } else {
        return Err("No supported audio recording tool found (pw-record, ffmpeg, or arecord).".to_string());
    };

    let child = cmd
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to spawn audio recorder: {}", e))?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let state = RecordingState {
        pid: child.id(),
        audio_path,
        start_time: now,
    };

    let json_data = serde_json::to_string(&state).map_err(|e| e.to_string())?;
    fs::write(&state_path, json_data).map_err(|e| format!("Failed to write state file: {}", e))?;

    Ok(())
}

/// Stop background audio recording (Second toggle press)
pub fn stop_recording(state: &RecordingState) -> Result<PathBuf, String> {
    let pid = state.pid;
    let audio_path = state.audio_path.clone();
    let state_path = get_state_path();

    // Stop audio recorder process
    if is_pid_alive(pid) {
        // Send SIGINT (2) so pw-record/ffmpeg/arecord flushes and writes valid WAV header
        let _ = Command::new("kill").arg("-2").arg(pid.to_string()).status();

        for _ in 0..25 {
            if !is_pid_alive(pid) {
                break;
            }
            thread::sleep(Duration::from_millis(80));
        }

        // Force kill if it didn't exit in 2 seconds
        if is_pid_alive(pid) {
            let _ = Command::new("kill").arg("-9").arg(pid.to_string()).status();
        }
    }

    // Always clean up state file
    if state_path.exists() {
        let _ = fs::remove_file(state_path);
    }

    if !audio_path.exists() {
        return Err("Audio file was not created.".to_string());
    }

    let metadata = fs::metadata(&audio_path).map_err(|e| e.to_string())?;
    if metadata.len() < 2000 {
        let _ = fs::remove_file(&audio_path);
        return Err("Recording was too short or silent.".to_string());
    }

    Ok(audio_path)
}

/// Run fixed duration recording
pub fn record_fixed_duration(seconds: u64) -> Result<PathBuf, String> {
    let audio_path = get_audio_path();
    if audio_path.exists() {
        let _ = fs::remove_file(&audio_path);
    }

    if command_exists("arecord") {
        let status = Command::new("arecord")
            .arg("-q")
            .arg("-f")
            .arg("cd")
            .arg("-t")
            .arg("wav")
            .arg("-d")
            .arg(seconds.to_string())
            .arg(&audio_path)
            .status();

        if status.map(|s| s.success()).unwrap_or(false) && audio_path.exists() {
            return Ok(audio_path);
        }
    }

    // Fallback: spawn and sleep
    start_recording()?;
    thread::sleep(Duration::from_secs(seconds));

    if let Ok(content) = fs::read_to_string(get_state_path()) {
        if let Ok(state) = serde_json::from_str::<RecordingState>(&content) {
            return stop_recording(&state);
        }
    }

    Err("Failed to record audio.".to_string())
}
