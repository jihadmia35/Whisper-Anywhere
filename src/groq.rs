use reqwest::blocking::{multipart, Client};
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::time::Duration;

pub struct GroqClient {
    keys: Vec<String>,
    model: String,
    language: String,
    client: Client,
}

impl GroqClient {
    pub fn new(keys: Vec<String>, model: String, language: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            keys,
            model,
            language,
            client,
        }
    }

    /// Transcribe audio file with automatic API key rotation on rate limits (HTTP 429)
    pub fn transcribe(&self, audio_path: &Path) -> Result<String, String> {
        if self.keys.is_empty() {
            return Err("No Groq API keys found. Add keys in config.json or export GROQ_API_KEY.".to_string());
        }

        let audio_bytes = fs::read(audio_path).map_err(|e| format!("Failed to read audio file: {}", e))?;
        if audio_bytes.len() < 2000 {
            return Err("Audio file is too short or empty.".to_string());
        }

        let total_keys = self.keys.len();

        for (attempt, key) in self.keys.iter().enumerate() {
            println!("⚡ [Attempt {}/{}] Calling Groq Whisper with key #{}...", attempt + 1, total_keys, attempt + 1);

            let mut form = multipart::Form::new()
                .text("model", self.model.clone())
                .part("file", multipart::Part::bytes(audio_bytes.clone())
                    .file_name("audio.wav")
                    .mime_str("audio/wav")
                    .map_err(|e| e.to_string())?);

            if !self.language.is_empty() {
                form = form.text("language", self.language.clone());
            }

            let response = self.client
                .post("https://api.groq.com/openai/v1/audio/transcriptions")
                .header("Authorization", format!("Bearer {}", key))
                .multipart(form)
                .send();

            match response {
                Ok(resp) => {
                    let status = resp.status();
                    if status.is_success() {
                        let json: Value = resp.json().map_err(|e| format!("Invalid JSON response: {}", e))?;
                        if let Some(text) = json.get("text").and_then(|t| t.as_str()) {
                            let trimmed = text.trim();
                            if !trimmed.is_empty() {
                                println!("✅ Transcription succeeded with key #{}.", attempt + 1);
                                return Ok(trimmed.to_string());
                            } else {
                                return Err("No speech detected.".to_string());
                            }
                        } else {
                            return Err("Missing 'text' field in API response.".to_string());
                        }
                    } else if status.as_u16() == 429 {
                        eprintln!("⚠️ Groq API key #{} hit rate limit (HTTP 429). Rotating to next key...", attempt + 1);
                        // Continue loop to try next key!
                        continue;
                    } else if status.as_u16() == 401 {
                        eprintln!("⚠️ Groq API key #{} is invalid (HTTP 401 Unauthorized). Rotating to next key...", attempt + 1);
                        continue;
                    } else {
                        let err_text = resp.text().unwrap_or_default();
                        eprintln!("❌ Groq API error ({}) with key #{}: {}", status, attempt + 1, err_text);
                        // If it's a 5xx server error, also retry with next key
                        if status.is_server_error() {
                            continue;
                        }
                    }
                }
                Err(err) => {
                    eprintln!("⚠️ Network error connecting to Groq with key #{}: {}", attempt + 1, err);
                    continue;
                }
            }
        }

        Err("All Groq API keys in the pool failed or hit rate limits (429). Please add more keys in config.json or wait for limits to reset.".to_string())
    }
}
