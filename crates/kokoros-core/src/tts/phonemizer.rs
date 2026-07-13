use crate::tts::chinese::ChineseG2P;

#[derive(Clone)]
pub struct Phonemizer {
    chinese_g2p: ChineseG2P,
}

impl Phonemizer {
    pub fn new() -> Self {
        Self {
            chinese_g2p: ChineseG2P::new(),
        }
    }

    pub fn phonemize(&self, text: &str, lang: &str) -> String {
        match lang {
            "zh" | "zh-CN" | "zh-TW" => self.chinese_g2p.process(text),
            _ => text.to_string(),
        }
    }
}

impl Default for Phonemizer {
    fn default() -> Self {
        Self::new()
    }
}
