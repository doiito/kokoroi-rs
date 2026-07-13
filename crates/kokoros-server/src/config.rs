use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_threads")]
    pub threads: usize,
    #[serde(default = "default_model_path")]
    pub model_path: PathBuf,
    #[serde(default = "default_voices_path")]
    pub voices_path: PathBuf,
    #[serde(default = "default_max_chars")]
    pub max_chars: usize,
}

fn default_host() -> String { "0.0.0.0".to_string() }
fn default_port() -> u16 { 3000 }
fn default_threads() -> usize { 4 }
fn default_model_path() -> PathBuf { PathBuf::from("examples/web/models/onnx/model.onnx") }
fn default_voices_path() -> PathBuf { PathBuf::from("data/voices-v1.0.bin") }
fn default_max_chars() -> usize { 400 }

impl Default for Config {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
            threads: default_threads(),
            model_path: default_model_path(),
            voices_path: default_voices_path(),
            max_chars: default_max_chars(),
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let config_path = std::env::var("KOKOROS_CONFIG")
            .unwrap_or_else(|_| "config.toml".to_string());
        
        if std::path::Path::new(&config_path).exists() {
            match std::fs::read_to_string(&config_path) {
                Ok(content) => {
                    match toml::from_str(&content) {
                        Ok(config) => {
                            tracing::info!("Loaded config from {}", config_path);
                            return config;
                        }
                        Err(e) => {
                            tracing::warn!("Failed to parse config file: {}, using defaults", e);
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to read config file: {}, using defaults", e);
                }
            }
        }
        
        Self::default()
    }
}
