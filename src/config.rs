use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

fn default_model() -> String {
    "whisper-large-v3-turbo".to_string()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    #[serde(default)]
    pub groq_api_keys: Vec<String>,
    #[serde(default)]
    pub groq_api_key: Option<String>,
    #[serde(default = "default_model")]
    pub model: String,
    #[serde(default)]
    pub language: String,
    #[serde(default)]
    pub duration: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            groq_api_keys: Vec::new(),
            groq_api_key: None,
            model: default_model(),
            language: String::new(),
            duration: 0,
        }
    }
}

impl Config {
    pub fn default_config_path() -> PathBuf {
        if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(".config/whisper-anywhere/config.json")
        } else {
            PathBuf::from("config.json")
        }
    }

    pub fn load(custom_path: Option<&str>) -> Self {
        let mut paths_to_check = Vec::new();

        if let Some(p) = custom_path {
            paths_to_check.push(PathBuf::from(p));
        }

        // 1. Next to the executable
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                paths_to_check.push(exe_dir.join("config.json"));
            }
        }

        // 2. User config directory ~/.config/whisper-anywhere/config.json
        if let Ok(home) = std::env::var("HOME") {
            paths_to_check.push(PathBuf::from(&home).join(".config/whisper-anywhere/config.json"));
            paths_to_check.push(PathBuf::from(&home).join(".whisper-anywhere.json"));
        }

        // 3. Current working directory config.json
        paths_to_check.push(PathBuf::from("config.json"));

        // 4. Known project directory fallback
        paths_to_check.push(PathBuf::from("/home/jihadmia/Projects/GPT Transcript Anywhere (Copy)/config.json"));

        for path in &paths_to_check {
            if path.exists() {
                match fs::read_to_string(path) {
                    Ok(content) => match serde_json::from_str::<Config>(&content) {
                        Ok(mut cfg) => {
                            if let Some(single_key) = cfg.groq_api_key.take() {
                                if !single_key.trim().is_empty() && !cfg.groq_api_keys.contains(&single_key) {
                                    cfg.groq_api_keys.push(single_key);
                                }
                            }
                            if !cfg.get_valid_keys().is_empty() {
                                return cfg;
                            }
                        }
                        Err(e) => {
                            eprintln!("⚠️ Warning: Found config at {:?}, but failed to parse: {}", path, e);
                        }
                    },
                    Err(e) => {
                        eprintln!("⚠️ Warning: Could not read config file {:?}: {}", path, e);
                    }
                }
            }
        }

        // Fallback to environment variable if set
        let mut cfg = Config::default();
        if let Ok(env_key) = std::env::var("GROQ_API_KEY") {
            if !env_key.trim().is_empty() {
                cfg.groq_api_keys.push(env_key);
            }
        }

        cfg
    }

    pub fn save(&self, path: &PathBuf) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let json_str = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(path, json_str).map_err(|e| format!("Failed to write config: {}", e))
    }

    pub fn get_valid_keys(&self) -> Vec<String> {
        let mut keys: Vec<String> = self.groq_api_keys
            .iter()
            .map(|k| k.trim().to_string())
            .filter(|k| !k.is_empty())
            .collect();

        if let Ok(env_key) = std::env::var("GROQ_API_KEY") {
            let env_trimmed = env_key.trim().to_string();
            if !env_trimmed.is_empty() && !keys.contains(&env_trimmed) {
                keys.push(env_trimmed);
            }
        }

        keys
    }
}
