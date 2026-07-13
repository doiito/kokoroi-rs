use crate::fallback::{Fallback, FallbackError, CharFallback};
use crate::language::Language;
use crate::languages::{LanguageRules, english::English};
use crate::lexicon::Lexicon;
use thiserror::Error;
use crate::tagger::PerceptronTagger;
use crate::token::MToken;
use num2words::Num2Words;
use regex::Regex;
use std::collections::HashMap;

#[derive(Error, Debug)]
pub enum G2PError {
    #[error("fallback error: {0}")]
    Fallback(#[from] FallbackError),
}

pub struct G2P {
    pub lexicon: Lexicon,
    pub unk: String,
    subtoken_regex: Regex,
    tagger: PerceptronTagger,
    rules: Box<dyn LanguageRules>,
    fallback: Option<Box<dyn Fallback>>,
}

impl G2P {
    pub fn new(lang: Language) -> Self {
        let subtoken_regex = Regex::new(
            r"(?x)
            ^['‘’]+ |
            (?:^-)?(?:\d?[,.]?\d)+ |
            [\-_]+ |
            ['‘’]{2,} |
            \p{L}+(?:[''']\p{L}+)* |
            [^\s\-_0-9\p{L}''] |
            ['‘’]+$
        ",
        )
        .unwrap();

        let weights_json = include_str!("resources/tagger/weights.json");
        let classes_txt = include_str!("resources/tagger/classes.txt");
        let tags_json = include_str!("resources/tagger/tags.json");

        let rules: Box<dyn LanguageRules> = match lang {
            Language::EnglishUS | Language::EnglishGB => Box::new(English),
        };

        let fallback: Option<Box<dyn Fallback>> = Some(Box::new(CharFallback::new()));

        Self {
            lexicon: Lexicon::new(lang),
            unk: "❓".to_string(),
            subtoken_regex,
            tagger: PerceptronTagger::new(weights_json, classes_txt, tags_json),
            rules,
            fallback,
        }
    }

    pub fn preprocess(&self, text: &str) -> (String, Vec<String>, HashMap<usize, String>) {
        let tokens: Vec<String> = text.split_whitespace().map(|s| s.to_string()).collect();
        (text.to_string(), tokens, HashMap::new())
    }

    pub fn tokenize(&self, text: &str) -> Vec<MToken> {
        let word_boundary_regex = Regex::new(r"\S+").unwrap();
        let mut tokens = Vec::new();

        for mat in word_boundary_regex.find_iter(text) {
            let word = mat.as_str();
            let subtokens: Vec<&str> = self
                .subtoken_regex
                .find_iter(word)
                .map(|m| m.as_str())
                .collect();

            if subtokens.is_empty() {
                let tk = MToken::new(word.to_string(), "NN".to_string(), " ".to_string());
                tokens.push(tk);
            } else {
                for sub in subtokens {
                    let tk = MToken::new(sub.to_string(), "NN".to_string(), " ".to_string());
                    tokens.push(tk);
                }
            }
        }

        tokens
    }

    pub fn g2p(&self, text: &str) -> Result<(String, Vec<MToken>), G2PError> {
        let (processed_text, _, _) = self.preprocess(text);
        let mut tokens = self.tokenize(&processed_text);

        let words_owned: Vec<String> = tokens.iter().map(|tk| tk.text.clone()).collect();
        let words: Vec<&str> = words_owned.iter().map(|s| s.as_str()).collect();
        let tags = self.tagger.tag(&words);

        for (i, tk) in tokens.iter().enumerate() {
            log::debug!("token[{}]: '{}'", i, tk.text);
        }

        let mut contexts: Vec<crate::lexicon::TokenContext> =
            vec![crate::lexicon::TokenContext::default(); tokens.len()];

        for (tk, tag) in tokens.iter_mut().zip(tags.iter()) {
            tk.tag = tag.tag.clone();
        }

        for i in (0..tokens.len()).rev() {
            let word = tokens[i].text.clone();
            let tag = tokens[i].tag.clone();
            let stress = if word == word.to_lowercase() {
                None
            } else {
                Some(if word == word.to_uppercase() {
                    self.lexicon.cap_stresses.1
                } else {
                    self.lexicon.cap_stresses.0
                })
            };

            if i < tokens.len() - 1 {
                let next_word = &tokens[i + 1].text;
                if let Some(first_char) = next_word.chars().next() {
                    let first_lower = first_char.to_lowercase().next().unwrap();
                    if "aeiou".contains(first_lower) {
                        contexts[i].future_vowel = Some(true);
                    } else if first_char.is_alphabetic() {
                        contexts[i].future_vowel = Some(false);
                    }
                }

                if next_word.to_lowercase() == "to" {
                    contexts[i].future_to = true;
                }
            }

            if tokens[i].phonemes.is_none() {
                let ctx = Some(&contexts[i]);

                if let Some((ps, _)) = self.lexicon.get_word(&word, &tag, stress, ctx) {
                    tokens[i].phonemes = Some(ps);
                }

                if tokens[i].phonemes.is_none() {
                    if word.contains('-') && word.len() > 1 {
                        let parts: Vec<&str> = word.split('-').filter(|s| !s.is_empty()).collect();
                        let mut sub_ps = Vec::new();
                        for part in parts {
                            let (p, _) = self.g2p(part)?;
                            sub_ps.push(p);
                        }
                        tokens[i].phonemes = Some(sub_ps.join(" "));
                    } else if self.is_number(&word) {
                        let spoken = self.convert_number(&word);
                        if spoken != word {
                            let (p, _) = self.g2p(&spoken)?;
                            tokens[i].phonemes = Some(p);
                        }
                    }
                }

                if tokens[i].phonemes.is_none()
                    && let Some(ps) = self.rules.apply_rules(&word, &tag, &self.lexicon) {
                        tokens[i].phonemes = Some(ps);
                    }

                if tokens[i].phonemes.is_none() {
                    if word.chars().count() > 1 {
                        if let Some(ref fallback) = self.fallback {
                            match fallback.phonemize(&word) {
                                Ok(ps) => {
                                    tokens[i].phonemes = Some(ps);
                                }
                                Err(e) => {
                                    log::warn!("fallback error for '{}': {}", word, e);
                                    let mut char_ps = Vec::new();
                                    for c in word.chars() {
                                        let (p, _) = self.g2p(&c.to_string())?;
                                        char_ps.push(p);
                                    }
                                    tokens[i].phonemes = Some(char_ps.join(" "));
                                }
                            }
                        } else {
                            let mut char_ps = Vec::new();
                            for c in word.chars() {
                                let (p, _) = self.g2p(&c.to_string())?;
                                char_ps.push(p);
                            }
                            tokens[i].phonemes = Some(char_ps.join(" "));
                        }
                    } else {
                        let normalized: String = word
                            .chars()
                            .map(|c| match c {
                                'é' | 'è' | 'ê' | 'ë' => 'e',
                                'á' | 'à' | 'â' | 'ä' | 'ã' | 'å' => 'a',
                                'í' | 'ì' | 'î' | 'ï' => 'i',
                                'ó' | 'ò' | 'ô' | 'ö' | 'õ' => 'o',
                                'ú' | 'ù' | 'û' | 'ü' => 'u',
                                'ñ' => 'n',
                                'ç' => 'c',
                                '—' | '–' => ' ',
                                _ => c,
                            })
                            .collect();

                        if normalized != word {
                            let (p, _) = self.g2p(&normalized)?;
                            tokens[i].phonemes = Some(p);
                        } else {
                            if word.chars().count() == 1 {
                                let c = word.chars().next().unwrap();
                                if c.is_ascii_punctuation() || "—–…".contains(c) {
                                    tokens[i].phonemes = Some(" ".to_string());
                                } else {
                                    tokens[i].phonemes = Some(self.unk.clone());
                                }
                            } else {
                                tokens[i].phonemes = Some(self.unk.clone());
                            }
                        }
                    }
                }
            }

            if i > 0 && tokens[i].phonemes.is_some() {
                let vowels = "AIOQWYaiuæɑɒɔəɛɜɪʊʌᵻ";
                let phonemes = tokens[i].phonemes.as_ref().unwrap();
                for c in phonemes.chars() {
                    if vowels.contains(c) {
                        contexts[i - 1].future_vowel = Some(true);
                        break;
                    }
                }
                let consonants = "bdfhjklmnpstvwzðŋɡɹɾʃʒʤʧθ";
                if contexts[i - 1].future_vowel.is_none() {
                    for c in phonemes.chars() {
                        if consonants.contains(c) {
                            contexts[i - 1].future_vowel = Some(false);
                            break;
                        }
                    }
                }
            }
        }

        let result = tokens
            .iter()
            .map(|tk| tk.phonemes.as_ref().unwrap_or(&self.unk).clone() + &tk.whitespace)
            .collect::<String>();

        Ok((result, tokens))
    }

    fn is_number(&self, word: &str) -> bool {
        let clean = word.replace(",", "");
        clean.parse::<i64>().is_ok()
    }

    fn convert_number(&self, word: &str) -> String {
        let clean = word.replace(",", "");
        if let Ok(val) = clean.parse::<i64>() {
            let n2w = match self.lexicon.lang {
                Language::EnglishUS | Language::EnglishGB => Num2Words::new(val),
            };
            if let Ok(spoken) = n2w.to_words() {
                return spoken;
            }
        }
        word.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_g2p_basic() {
        let _ = env_logger::try_init();
        let g2p = G2P::new(Language::EnglishUS);
        let (phonemes, _) = g2p.g2p("Hello, world!").unwrap();
        println!("Phonemes: {}", phonemes);
        assert!(!phonemes.contains("❓"));
    }

    #[test]
    fn test_ebook() {
        let g2p = G2P::new(Language::EnglishUS);
        let (phonemes, _) = g2p.g2p("eBook").unwrap();
        println!("eBook: {}", phonemes);
        assert!(!phonemes.is_empty(), "eBook should produce phonemes");
    }

    #[test]
    fn test_english_abbreviations() {
        let g2p = G2P::new(Language::EnglishUS);
        let cases = vec![
            "I'll", "I've", "it's", "he's", "she's", "we're", "they're",
            "isn't", "aren't", "wasn't", "weren't", "don't", "doesn't",
            "didn't", "can't", "couldn't", "shouldn't", "wouldn't", "won't",
            "hasn't", "haven't", "hadn't", "let's", "that's", "what's",
            "who's", "here's", "there's", "where's", "how's",
        ];
        for text in cases {
            let (p, _) = g2p.g2p(text).unwrap();
            println!("'{}' -> '{}'", text, p);
            assert!(!p.contains("❓"), "Failed for '{}'", text);
        }
    }

    #[test]
    fn test_casing_and_special_chars() {
        let g2p = G2P::new(Language::EnglishUS);

        let (playing, _) = g2p.g2p("PLAYING").unwrap();
        println!("PLAYING: {}", playing);
        assert!(!playing.contains("❓"), "PLAYING should be resolved, got: {}", playing);

        let (ive, _) = g2p.g2p("I've").unwrap();
        println!("I've: {}", ive);
        assert!(!ive.contains("❓"), "I've should be resolved, got: {}", ive);

        let (dash, _) = g2p.g2p("word - word — word").unwrap();
        println!("Dash: {}", dash);
        assert!(!dash.contains("❓"), "Dashes should be handled gracefully, got: {}", dash);
    }

    #[test]
    fn test_kokoros_basic() {
        let g2p = G2P::new(Language::EnglishUS);
        let cases = vec![
            "hello", "world", "the quick brown fox", "testing phonemization",
            "Hello, world!", "123", "restriction", "restrictions", "",
        ];
        for text in cases {
            let (p, _) = g2p.g2p(text).unwrap();
            println!("'{}' -> '{}'", text, p);
            if !text.is_empty() {
                assert!(!p.is_empty(), "Failed for '{}'", text);
            }
        }
    }

    #[test]
    fn test_kokoros_numbers() {
        let g2p = G2P::new(Language::EnglishUS);
        let cases = vec![
            "CHAPTER XIV", "CHAPTER 14", "CHAPTER 123",
            "I have 5 apples and 42 oranges", "The year 2024",
            "1234567890", "CHAPTER I", "CHAPTER II", "CHAPTER III",
            "CHAPTER IV", "CHAPTER V", "CHAPTER X", "CHAPTER XX", "CHAPTER XXX",
            "In 2024, CHAPTER XIV had 42 pages.", "The price is $123.45",
            "Temperature: -5°C", "Score: 100%", "Version 2.0", "3.14159",
        ];
        for text in cases {
            let (p, _) = g2p.g2p(text).unwrap();
            println!("'{}' -> '{}'", text, p);
            assert!(!p.is_empty(), "Failed for '{}'", text);
        }
    }

    #[test]
    fn test_kokoros_utf8_and_special() {
        let g2p = G2P::new(Language::EnglishUS);
        let cases = vec![
            "café", "naïve", "résumé", "Zürich", "São Paulo", "Müller",
            "北京", "こんにちは", "Здравствуй", "مرحبا", "🎉🎊🎈",
            "\x00\x01\x02",
            "Hello 世界", "123中文", "English123中文",
            "hello\u{200B}world", "hello\u{200C}world", "hello\u{200D}world",
            "caf\u{00E9}", "na\u{00EF}ve",
        ];
        for text in cases {
            let (p, _) = g2p.g2p(text).unwrap();
            println!("'{}' -> '{}'", text, p);
        }
    }

    #[test]
    fn test_kokoros_punctuation() {
        let g2p = G2P::new(Language::EnglishUS);
        let cases = vec![
            "Hello—world", "Hello–world", "Hello…world",
            "\"quoted text\"", "'single quotes'",
            "«French quotes»", "„German quotes„", "「Japanese quotes」",
            "Dr. Smith", "Mr. Jones", "Mrs. Brown", "Ms. Davis",
            "etc.", "U.S.A.", "Ph.D.", "A.I.", "NASA", "FBI",
            "   ", "\n\n", "\t\t", "\r\n",
        ];
        for text in cases {
            let (p, _) = g2p.g2p(text).unwrap();
            println!("'{}' -> '{}'", text, p);
        }
    }

    #[test]
    fn test_kokoros_long_text() {
        let g2p = G2P::new(Language::EnglishUS);
        let long_text = "a".repeat(1000);
        let (p, _) = g2p.g2p(&long_text).unwrap();
        assert!(!p.is_empty());
    }
}
