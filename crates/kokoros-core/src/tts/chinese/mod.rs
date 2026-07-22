pub mod polyphonic;
pub mod tone_sandhi;
pub mod transcription;

use lazy_static::lazy_static;
use pinyin::ToPinyin;
use regex::Regex;
use std::collections::HashSet;

pub use transcription::ZH_MAP;
pub use polyphonic::PolyphonicDisambiguator;

#[cfg(not(target_arch = "wasm32"))]
lazy_static::lazy_static! {
    static ref JIEBA: std::sync::Mutex<jieba_rs::Jieba> = std::sync::Mutex::new(jieba_rs::Jieba::new());
}

lazy_static! {
    static ref ZH_CHAR_PATTERN: Regex = Regex::new(r"[\u4E00-\u9FFF]").unwrap();
    static ref ZH_SEGMENT_PATTERN: Regex = Regex::new(r"[\u4E00-\u9FFF]+|[^\u4E00-\u9FFF]+").unwrap();

    static ref PUNCTUATION_MAP: Vec<(&'static str, &'static str)> = vec![
        ("\u{3001}", ", "),
        ("\u{FF0C}", ", "),
        ("\u{3002}", ". "),
        ("\u{FF0E}", ". "),
        ("\u{FF01}", "! "),
        ("\u{FF1A}", ": "),
        ("\u{FF1B}", "; "),
        ("\u{FF1F}", "? "),
        ("\u{00AB}", " \""),
        ("\u{00BB}", "\" "),
        ("\u{300A}", " \""),
        ("\u{300B}", "\" "),
        ("\u{300C}", " \""),
        ("\u{300D}", "\" "),
        ("\u{3010}", " \""),
        ("\u{3011}", "\" "),
        ("\u{FF08}", " ("),
        ("\u{FF09}", ") "),
    ];

    static ref MUST_ERHUA: HashSet<&'static str> = {
        let mut s = HashSet::new();
        for word in &[
            "小院儿", "胡同儿", "范儿", "老汉儿", "撒欢儿",
            "寻老礼儿", "妥妥儿", "媳妇儿"
        ] {
            s.insert(*word);
        }
        s
    };

    static ref NOT_ERHUA: HashSet<&'static str> = {
        let mut s = HashSet::new();
        for word in &[
            "虐儿", "为儿", "护儿", "瞒儿", "救儿", "替儿", "有儿", "一儿",
            "我儿", "俺儿", "妻儿", "拐儿", "聋儿", "乞儿", "患儿", "幼儿",
            "孤儿", "婴儿", "婴幼儿", "连体儿", "脑瘫儿", "流浪儿", "体弱儿",
            "混血儿", "蜜雪儿", "舫儿", "祖儿", "美儿", "应采儿", "可儿",
            "侄儿", "孙儿", "侄孙儿", "女儿", "男儿", "红孩儿", "花儿",
            "虫儿", "马儿", "鸟儿", "猪儿", "猫儿", "狗儿", "少儿"
        ] {
            s.insert(*word);
        }
        s
    };
}

#[derive(Clone)]
pub struct ChineseG2P {
    use_bopomofo: bool,
    polyphonic: Option<PolyphonicDisambiguator>,
}

impl Default for ChineseG2P {
    fn default() -> Self {
        Self::new()
    }
}

impl ChineseG2P {
    pub fn new() -> Self {
        Self {
            use_bopomofo: true,
            polyphonic: Some(PolyphonicDisambiguator::new()),
        }
    }

    pub fn with_ipa() -> Self {
        Self {
            use_bopomofo: false,
            polyphonic: Some(PolyphonicDisambiguator::new()),
        }
    }

    pub fn without_polyphonic() -> Self {
        Self {
            use_bopomofo: true,
            polyphonic: None,
        }
    }

    fn map_punctuation(text: &str) -> String {
        let mut result = text.to_string();
        for (from, to) in PUNCTUATION_MAP.iter() {
            result = result.replace(from, to);
        }
        result.trim().to_string()
    }

    fn convert_numbers(text: &str) -> String {
        use chinese_number::{ChineseCase, ChineseCountMethod, ChineseVariant, NumberToChinese};

        let mut result = String::new();
        let mut num_buffer = String::new();

        for ch in text.chars() {
            if ch.is_ascii_digit() {
                num_buffer.push(ch);
            } else {
                if !num_buffer.is_empty() {
                    if let Ok(num) = num_buffer.parse::<i64>() {
                        if let Ok(chinese) = num.to_chinese(
                            ChineseVariant::Simple,
                            ChineseCase::Lower,
                            ChineseCountMethod::Low,
                        ) {
                            result.push_str(&chinese);
                        } else {
                            result.push_str(&num_buffer);
                        }
                    } else {
                        result.push_str(&num_buffer);
                    }
                    num_buffer.clear();
                }
                result.push(ch);
            }
        }

        if !num_buffer.is_empty() {
            if let Ok(num) = num_buffer.parse::<i64>() {
                if let Ok(chinese) = num.to_chinese(
                    ChineseVariant::Simple,
                    ChineseCase::Lower,
                    ChineseCountMethod::Low,
                ) {
                    result.push_str(&chinese);
                } else {
                    result.push_str(&num_buffer);
                }
            } else {
                result.push_str(&num_buffer);
            }
        }

        result
    }

    fn segment(text: &str) -> Vec<String> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let jieba = JIEBA.lock().unwrap();
            return jieba
                .cut(text, false)
                .into_iter()
                .map(|s| s.to_string())
                .collect();
        }
        #[cfg(target_arch = "wasm32")]
        {
            text.chars().map(|c| c.to_string()).collect()
        }
    }

    fn segment_with_pos(text: &str) -> Vec<(String, String)> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let jieba = JIEBA.lock().unwrap();
            return jieba
                .tag(text, false)
                .into_iter()
                .map(|t| (t.word.to_string(), t.tag.to_string()))
                .collect();
        }
        #[cfg(target_arch = "wasm32")]
        {
            text.chars().map(|c| (c.to_string(), "x".to_string())).collect()
        }
    }

    fn word_to_pinyin(word: &str) -> Vec<String> {
        word.chars()
            .filter_map(|c| {
                c.to_pinyin().map(|p| {
                    p.with_tone_num_end().to_string()
                })
            })
            .collect()
    }

    fn word_to_pinyin_with_disambiguation(&self, word: &str, sentence: &str, word_start_idx: usize) -> Vec<String> {
        let chars: Vec<char> = word.chars().collect();
        let mut pinyins = Vec::new();

        for (i, c) in chars.iter().enumerate() {
            let char_idx = word_start_idx + i;

            if let Some(ref disambiguator) = self.polyphonic {
                if let Some(pronunciation) = disambiguator.disambiguate(sentence, char_idx) {
                    pinyins.push(pronunciation);
                    continue;
                }
            }

            if let Some(pinyin) = c.to_pinyin() {
                pinyins.push(pinyin.with_tone_num_end().to_string());
            }
        }

        pinyins
    }

    #[allow(dead_code)]
    fn word_to_bopomofo(&self, word: &str, pos: &str) -> String {
        let pinyins = Self::word_to_pinyin(word);
        let modified_pinyins = tone_sandhi::apply_tone_sandhi(word, pos, &pinyins);
        modified_pinyins
            .iter()
            .map(|py| transcription::pinyin_to_bopomofo(py))
            .collect::<Vec<_>>()
            .join("")
    }

    fn word_to_ipa(&self, word: &str) -> String {
        let pinyins = Self::word_to_pinyin(word);
        pinyins
            .iter()
            .map(|py| transcription::pinyin_to_ipa(py))
            .collect::<Vec<_>>()
            .join("")
    }

    fn is_chinese_char(c: char) -> bool {
        ('\u{4E00}'..='\u{9FFF}').contains(&c)
    }

    pub fn process(&self, text: &str) -> String {
        if text.trim().is_empty() {
            return String::new();
        }

        let text = Self::convert_numbers(text);
        let text = Self::map_punctuation(&text);

        if self.use_bopomofo {
            self.process_bopomofo(&text)
        } else {
            self.process_ipa(&text)
        }
    }

    fn process_bopomofo(&self, text: &str) -> String {
        let mut result = String::new();
        let mut prev_was_chinese = false;

        for cap in ZH_SEGMENT_PATTERN.find_iter(text) {
            let segment = cap.as_str();
            let first_char = segment.chars().next().unwrap_or(' ');
            let is_chinese = Self::is_chinese_char(first_char);

            if is_chinese {
                let words_with_pos = Self::segment_with_pos(segment);
                let merged = tone_sandhi::pre_merge_for_modify(&words_with_pos);

                let mut char_offset = 0;

                for (i, (word, pos)) in merged.iter().enumerate() {
                    if i > 0 || prev_was_chinese {
                        result.push('/');
                    }

                    if pos == "x" {
                        result.push_str(word);
                    } else {
                        let pinyins = self.word_to_pinyin_with_disambiguation(word, segment, char_offset);
                        let modified_pinyins = tone_sandhi::apply_tone_sandhi(word, pos, &pinyins);

                        let bopomofo: String = modified_pinyins
                            .iter()
                            .map(|py| transcription::pinyin_to_bopomofo(py))
                            .collect();

                        result.push_str(&bopomofo);
                    }

                    char_offset += word.chars().count();
                }
                prev_was_chinese = true;
            } else {
                result.push_str(segment);
                prev_was_chinese = false;
            }
        }

        result
    }

    fn process_ipa(&self, text: &str) -> String {
        let mut result = String::new();

        for cap in ZH_SEGMENT_PATTERN.find_iter(text) {
            let segment = cap.as_str();
            let first_char = segment.chars().next().unwrap_or(' ');
            let is_chinese = Self::is_chinese_char(first_char);

            if is_chinese {
                let words = Self::segment(segment);
                let ipa_parts: Vec<String> = words.iter().map(|w| self.word_to_ipa(w)).collect();
                result.push_str(&ipa_parts.join(" "));
            } else {
                result.push_str(segment);
            }
        }

        result.replace('\u{032F}', "")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_punctuation_mapping() {
        let text = "你好，世界！";
        let mapped = ChineseG2P::map_punctuation(text);
        assert!(mapped.contains(", "));
        assert!(mapped.contains("!"));
    }

    #[test]
    fn test_number_conversion() {
        let text = "我有123个苹果";
        let converted = ChineseG2P::convert_numbers(text);
        assert!(converted.contains("一百二十三"));
    }

    #[test]
    fn test_basic_g2p() {
        let g2p = ChineseG2P::new();
        let result = g2p.process("你好");
        println!("你好 -> {}", result);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_specific_chars() {
        let g2p = ChineseG2P::new();
        
        let result = g2p.process("的");
        println!("的 -> {}", result);
        
        let result = g2p.process("事");
        println!("事 -> {}", result);
        
        let result = g2p.process("银行里有很多重要的事情");
        println!("银行里有很多重要的事情 -> {}", result);
        
        let result = g2p.process("今天天气很好");
        println!("今天天气很好 -> {}", result);
    }

    #[test]
    fn test_full_sentence_g2p() {
        let g2p = ChineseG2P::new();
        let sentences = [
            ("你好世界", "Hello world"),
            ("中国人民", "Chinese people"),
            ("我爱北京天安门", "I love Beijing Tiananmen"),
        ];

        for (chinese, _description) in sentences {
            let result = g2p.process(chinese);
            assert!(!result.is_empty(), "G2P should produce output for: {}", chinese);
            assert!(
                result.chars().any(|c| c.is_ascii_digit()),
                "Output should contain tone numbers for: {}",
                chinese
            );
        }
    }

    #[test]
    fn test_tone_sandhi_integration() {
        let g2p = ChineseG2P::new();

        let test_cases = [
            "不是",
            "一个",
            "你好",
        ];

        for text in test_cases {
            let result = g2p.process(text);
            assert!(!result.is_empty());
        }
    }

    #[test]
    fn test_polyphonic_disambiguation() {
        let g2p = ChineseG2P::new();

        let result1 = g2p.process("银行");
        println!("银行 -> {}", result1);
        assert!(result1.contains("ㄏㄤ"));

        let result2 = g2p.process("行走");
        println!("行走 -> {}", result2);
        assert!(result2.contains("ㄒ"));

        let result3 = g2p.process("重要");
        println!("重要 -> {}", result3);
        assert!(result3.contains("ㄓ中") || result3.contains("ㄓㄨㄥ"));

        let result4 = g2p.process("重复");
        println!("重复 -> {}", result4);
        assert!(result4.contains("ㄔ中") || result4.contains("ㄔㄨㄥ"));
    }

    #[test]
    fn test_polyphonic_sentence() {
        let g2p = ChineseG2P::new();

        let result = g2p.process("银行里有很多重要的事情");
        println!("银行里有很多重要的事情 -> {}", result);
        assert!(!result.is_empty());
    }
}
