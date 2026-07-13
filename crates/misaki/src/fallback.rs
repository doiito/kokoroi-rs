use thiserror::Error;

#[derive(Error, Debug)]
pub enum FallbackError {
    #[error("no phonemes matched for '{word}'")]
    NoPhonemes { word: String },
}

pub trait Fallback: Send + Sync {
    fn phonemize(&self, word: &str) -> Result<String, FallbackError>;
}

pub struct CharFallback;

impl Default for CharFallback {
    fn default() -> Self {
        Self::new()
    }
}

impl CharFallback {
    pub fn new() -> Self {
        Self
    }
}

impl Fallback for CharFallback {
    fn phonemize(&self, word: &str) -> Result<String, FallbackError> {
        if word.is_empty() {
            return Err(FallbackError::NoPhonemes { word: word.to_string() });
        }
        let mut char_ps = Vec::new();
        for c in word.chars() {
            char_ps.push(c.to_string());
        }
        Ok(char_ps.join(" "))
    }
}
