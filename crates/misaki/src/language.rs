use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum Language {
    EnglishUS,
    EnglishGB,
}

impl Language {
    pub fn is_english(&self) -> bool {
        matches!(self, Language::EnglishUS | Language::EnglishGB)
    }
}
