mod audio;
mod config;
mod groq;
mod notify;
mod paste;

use clap::Parser;
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "whisper-anywhere")]
#[command(author = "Jihad Mia")]
#[command(version = "0.1.0")]
#[command(about = "Voice dictation anywhere using Groq Whisper with multi-key failover and auto-paste")]
struct Cli {
    /// Fixed recording duration in seconds (0 for toggle mode)
    #[arg(short, long, default_value_t = 0)]
    duration: u64,

    /// Force toggle mode (press once to start, again to stop)
    #[arg(short, long, default_value_t = true)]
    toggle: bool,

    /// Optional path to custom config.json
    #[arg(short, long)]
    config: Option<String>,

    /// Temporary Groq API key for this run only
    #[arg(short, long)]
    key: Option<String>,

    /// Save a Groq API key permanently to your configuration
    #[arg(long)]
    add_key: Option<String>,
}

fn main() {
    let args = Cli::parse();

    // 1. Handle --add-key to save a key permanently
    if let Some(new_key) = args.add_key {
        let trimmed = new_key.trim().to_string();
        if trimmed.is_empty() {
            eprintln!("Error: API key cannot be empty.");
            std::process::exit(1);
        }

        let mut cfg = config::Config::load(args.config.as_deref());
        if !cfg.groq_api_keys.contains(&trimmed) {
            cfg.groq_api_keys.push(trimmed);
        }

        let target_path = args.config.map(PathBuf::from).unwrap_or_else(config::Config::default_config_path);
        match cfg.save(&target_path) {
            Ok(_) => {
                println!("✅ Key added successfully to: {:?}", target_path);
                println!("Total active keys: {}", cfg.groq_api_keys.len());
                std::process::exit(0);
            }
            Err(e) => {
                eprintln!("Failed to save config: {}", e);
                std::process::exit(1);
            }
        }
    }

    let mut cfg = config::Config::load(args.config.as_deref());

    // Add temporary CLI key if provided
    if let Some(cli_key) = args.key {
        let trimmed = cli_key.trim().to_string();
        if !trimmed.is_empty() && !cfg.groq_api_keys.contains(&trimmed) {
            cfg.groq_api_keys.insert(0, trimmed);
        }
    }

    let valid_keys = cfg.get_valid_keys();
    if valid_keys.is_empty() {
        notify::notify("❌ Error", "No Groq API keys found. Add keys with: whisper-anywhere --add-key <key>", "dialog-error", 4000);
        eprintln!("\n❌ No Groq API keys found!");
        eprintln!("Get a free key at: https://console.groq.com/keys");
        eprintln!("Then add it with:  whisper-anywhere --add-key <YOUR_KEY>\n");
        std::process::exit(1);
    }

    // 2. Fixed Duration Mode
    if args.duration > 0 || cfg.duration > 0 {
        let sec = if args.duration > 0 { args.duration } else { cfg.duration };
        match audio::record_fixed_duration(sec) {
            Ok(audio_path) => {
                process_transcription(&audio_path, &cfg, valid_keys);
            }
            Err(e) => {
                notify::notify("❌ Recording Error", &e, "dialog-error", 3000);
                eprintln!("Recording error: {}", e);
            }
        }
        return;
    }

    // 3. Toggle Mode
    let state_path = audio::get_state_path();
    if state_path.exists() {
        if let Ok(content) = fs::read_to_string(&state_path) {
            if let Ok(state) = serde_json::from_str::<audio::RecordingState>(&content) {
                if audio::is_pid_alive(state.pid) {
                    // STOP and TRANSCRIBE
                    match audio::stop_recording(&state) {
                        Ok(audio_path) => {
                            process_transcription(&audio_path, &cfg, valid_keys);
                        }
                        Err(e) => {
                            eprintln!("Dictation cancelled: {}", e);
                        }
                    }
                    return;
                }
            }
        }
        // If state was stale, clean it up
        let _ = fs::remove_file(&state_path);
    }

    // START RECORDING
    if let Err(e) = audio::start_recording() {
        notify::notify("❌ Microphone Error", &e, "dialog-error", 3000);
        eprintln!("Failed to start recording: {}", e);
    }
}

fn process_transcription(audio_path: &PathBuf, cfg: &config::Config, valid_keys: Vec<String>) {
    let client = groq::GroqClient::new(valid_keys, cfg.model.clone(), cfg.language.clone());

    match client.transcribe(audio_path) {
        Ok(text) => {
            notify::notify("✅ Done", &text, "emblem-default", 2500);
            let pasted = paste::copy_and_paste(&text);
            if !pasted {
                eprintln!("Auto-paste failed. Text is in clipboard (Press Ctrl+V).");
            }
        }
        Err(e) => {
            notify::notify("❌ Transcription Failed", &e, "dialog-error", 4000);
            eprintln!("Transcription failed: {}", e);
        }
    }

    // Clean up temporary audio file
    if audio_path.exists() {
        let _ = fs::remove_file(audio_path);
    }
}
