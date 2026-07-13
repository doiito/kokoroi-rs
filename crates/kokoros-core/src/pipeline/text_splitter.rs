use std::collections::HashSet;

static SENTENCE_TERMINATORS: &[char] = &['.', '!', '?', '…', '。', '！', '？'];
static TRAILING_CHARS: &[char] = &['"', '\'', ')', ']', '}', '」', '』'];
static OPENING_CHARS: &[char] = &['"', '\'', '(', '[', '{', '「', '『', '《', '〈', '‹', '«', '〈', '〔', '【'];

lazy_static::lazy_static! {
    static ref ABBREVIATIONS: HashSet<&'static str> = {
        let mut set = HashSet::new();
        for abbr in &[
            "mr", "mrs", "ms", "dr", "prof", "sr", "jr", "sgt", "col", "gen",
            "rep", "sen", "gov", "lt", "maj", "capt", "st", "mt", "etc", "co",
            "inc", "ltd", "dept", "vs", "p", "pg", "jan", "feb", "mar", "apr",
            "jun", "jul", "aug", "sep", "sept", "oct", "nov", "dec", "sun",
            "mon", "tu", "tue", "tues", "wed", "th", "thu", "thur", "thurs",
            "fri", "sat"
        ] {
            set.insert(*abbr);
        }
        set
    };
    
    static ref MATCHING_PAIRS: std::collections::HashMap<char, char> = {
        let mut map = std::collections::HashMap::new();
        map.insert(')', '(');
        map.insert(']', '[');
        map.insert('}', '{');
        map.insert('》', '《');
        map.insert('〉', '〈');
        map.insert('›', '‹');
        map.insert('»', '«');
        map.insert('〉', '〈');
        map.insert('」', '「');
        map.insert('』', '『');
        map.insert('〕', '〔');
        map.insert('】', '【');
        map
    };
}

pub struct TextSplitterConfig {
    pub max_phonemes: usize,
    pub max_chars: usize,
    pub waterfall_priorities: Vec<String>,
}

impl Default for TextSplitterConfig {
    fn default() -> Self {
        Self {
            max_phonemes: 510,
            max_chars: 150,
            waterfall_priorities: vec![
                "!".to_string() + ".?…",
                ":;".to_string(),
                ",，、".to_string(),
            ],
        }
    }
}

pub struct TextSplitter {
    config: TextSplitterConfig,
}

impl TextSplitter {
    pub fn new(config: TextSplitterConfig) -> Self {
        Self { config }
    }

    pub fn split_into_sentences(&self, text: &str) -> Vec<String> {
        let chars: Vec<char> = text.chars().collect();
        let len = chars.len();
        
        if len == 0 {
            return vec![];
        }
        
        let mut sentences = Vec::new();
        let mut sentence_start = 0;
        let mut stack: Vec<char> = Vec::new();
        let mut i = 0;
        
        while i < len {
            let c = chars[i];
            Self::update_stack(c, &mut stack, i, &chars);
            
            if stack.is_empty() && Self::is_sentence_terminator(c) {
                let current_segment: String = chars[sentence_start..i].iter().collect();
                
                if Self::is_numbered_list_item(&current_segment) {
                    i += 1;
                    continue;
                }
                
                let (boundary_end, next_non_space) = self.scan_boundary(i, &chars);
                
                if i == next_non_space - 1 && c != '\n' {
                    i += 1;
                    continue;
                }
                
                if next_non_space == len {
                    break;
                }
                
                let token = Self::get_token_before(&chars, i, sentence_start);
                
                if let Some(ref tok) = token {
                    if Self::is_url_or_email(tok) {
                        i = sentence_start + tok.len();
                        continue;
                    }
                    
                    if Self::is_abbreviation(tok) {
                        i += 1;
                        continue;
                    }
                    
                    if Self::is_middle_initials(tok) && next_non_space < len {
                        if chars[next_non_space].is_uppercase() {
                            i += 1;
                            continue;
                        }
                    }
                }
                
                if c == '.' && next_non_space < len && chars[next_non_space].is_lowercase() {
                    i += 1;
                    continue;
                }
                
                let sentence: String = chars[sentence_start..=boundary_end].iter().collect();
                let trimmed = sentence.trim();
                
                if trimmed == "..." || trimmed == "…" {
                    i += 1;
                    continue;
                }
                
                if !trimmed.is_empty() {
                    sentences.push(trimmed.to_string());
                }
                
                i = boundary_end + 1;
                sentence_start = i;
                continue;
            }
            i += 1;
        }
        
        if sentence_start < len {
            let remainder: String = chars[sentence_start..].iter().collect();
            let trimmed = remainder.trim();
            if !trimmed.is_empty() {
                sentences.push(trimmed.to_string());
            }
        }
        
        sentences
    }

    pub fn split_by_phoneme_limit(&self, text: &str, get_phoneme_count: impl Fn(&str) -> usize) -> Vec<String> {
        let phoneme_count = get_phoneme_count(text);
        
        if phoneme_count <= self.config.max_phonemes {
            return vec![text.to_string()];
        }
        
        let sentences = self.split_into_sentences(text);
        let mut chunks = Vec::new();
        let mut current_chunk = String::new();
        let mut current_phoneme_count = 0;
        
        for sentence in sentences {
            let sentence_phoneme_count = get_phoneme_count(&sentence);
            
            if sentence_phoneme_count > self.config.max_phonemes {
                if !current_chunk.is_empty() {
                    chunks.push(current_chunk.trim().to_string());
                    current_chunk.clear();
                    current_phoneme_count = 0;
                }
                
                for sub_chunk in self.split_long_sentence(&sentence, &get_phoneme_count) {
                    chunks.push(sub_chunk);
                }
            } else if current_phoneme_count + sentence_phoneme_count > self.config.max_phonemes {
                if !current_chunk.is_empty() {
                    chunks.push(current_chunk.trim().to_string());
                }
                current_chunk = sentence.clone();
                current_phoneme_count = sentence_phoneme_count;
            } else {
                if !current_chunk.is_empty() {
                    current_chunk.push_str("。");
                }
                current_chunk.push_str(&sentence);
                current_phoneme_count += sentence_phoneme_count;
            }
        }
        
        if !current_chunk.trim().is_empty() {
            chunks.push(current_chunk.trim().to_string());
        }
        
        chunks
    }

    fn split_long_sentence(&self, text: &str, get_phoneme_count: impl Fn(&str) -> usize) -> Vec<String> {
        let chars: Vec<char> = text.chars().collect();
        let mut chunks = Vec::new();
        let mut start = 0;
        
        while start < chars.len() {
            let mut end = (start + self.config.max_chars).min(chars.len());
            
            if end < chars.len() {
                if let Some(split_pos) = self.find_waterfall_split(&chars, start, end) {
                    end = split_pos;
                } else {
                    for i in (start..end).rev() {
                        if chars[i].is_whitespace() {
                            end = i;
                            break;
                        }
                    }
                }
            }
            
            if end <= start {
                end = (start + self.config.max_chars).min(chars.len());
            }
            
            let chunk: String = chars[start..end].iter().collect();
            let trimmed = chunk.trim();
            
            if !trimmed.is_empty() {
                let phoneme_count = get_phoneme_count(trimmed);
                if phoneme_count <= self.config.max_phonemes {
                    chunks.push(trimmed.to_string());
                } else {
                    let sub_chunks = self.split_recursive(trimmed, &get_phoneme_count);
                    chunks.extend(sub_chunks);
                }
            }
            
            start = end;
        }
        
        chunks
    }

    fn split_recursive(&self, text: &str, get_phoneme_count: impl Fn(&str) -> usize) -> Vec<String> {
        let mut result = Vec::new();
        let mut pending = vec![text.to_string()];
        
        while let Some(current) = pending.pop() {
            let phoneme_count = get_phoneme_count(&current);
            
            if phoneme_count <= self.config.max_phonemes {
                result.push(current);
                continue;
            }
            
            let chars: Vec<char> = current.chars().collect();
            let mid = chars.len() / 2;
            
            let mut split_pos = mid;
            for i in (0..mid).rev() {
                let c = chars[i];
                if c == '，' || c == ',' || c == '、' || c == ' ' || c == '；' || c == ';' {
                    split_pos = i + 1;
                    break;
                }
            }
            
            if split_pos == mid {
                for i in mid..chars.len() {
                    let c = chars[i];
                    if c == '，' || c == ',' || c == '、' || c == ' ' || c == '；' || c == ';' {
                        split_pos = i + 1;
                        break;
                    }
                }
            }
            
            if split_pos == 0 || split_pos >= chars.len() {
                split_pos = mid;
            }
            
            let left: String = chars[0..split_pos].iter().collect();
            let right: String = chars[split_pos..].iter().collect();
            
            if !right.trim().is_empty() {
                pending.push(right.trim().to_string());
            }
            if !left.trim().is_empty() {
                pending.push(left.trim().to_string());
            }
        }
        
        result.reverse();
        result
    }

    fn find_waterfall_split(&self, chars: &[char], start: usize, end: usize) -> Option<usize> {
        for priority_chars in &self.config.waterfall_priorities {
            for i in (start..end).rev() {
                if priority_chars.contains(chars[i]) {
                    return Some(i + 1);
                }
            }
        }
        None
    }

    fn is_sentence_terminator(c: char) -> bool {
        SENTENCE_TERMINATORS.contains(&c) || c == '\n'
    }

    fn is_trailing_char(c: char) -> bool {
        TRAILING_CHARS.contains(&c)
    }

    fn is_numbered_list_item(segment: &str) -> bool {
        let re = regex::Regex::new(r"(^|\n)\d+$").unwrap();
        re.is_match(segment)
    }

    fn is_abbreviation(token: &str) -> bool {
        let clean: String = token
            .trim_end_matches('.')
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect();
        
        let lower = clean.to_lowercase();
        let lower_str: &str = &lower;
        
        if ABBREVIATIONS.contains(lower_str) {
            return true;
        }
        
        let re = regex::Regex::new(r"^([a-zA-Z]\.)+$").unwrap();
        re.is_match(token)
    }

    fn is_url_or_email(token: &str) -> bool {
        token.contains("://") || token.contains('@')
    }

    fn is_middle_initials(token: &str) -> bool {
        let re = regex::Regex::new(r"^([A-Za-z]\.)+$").unwrap();
        re.is_match(token)
    }

    fn update_stack(c: char, stack: &mut Vec<char>, i: usize, chars: &[char]) {
        if c == '"' || c == '\'' {
            if c == '\'' && i > 0 && i < chars.len() - 1 {
                let prev = chars[i - 1];
                let next = chars[i + 1];
                if prev.is_alphabetic() && next.is_alphabetic() {
                    return;
                }
            }
            if stack.last() == Some(&c) {
                stack.pop();
            } else {
                stack.push(c);
            }
            return;
        }
        
        if OPENING_CHARS.contains(&c) {
            stack.push(c);
            return;
        }
        
        if let Some(&expected_opening) = MATCHING_PAIRS.get(&c) {
            if stack.last() == Some(&expected_opening) {
                stack.pop();
            }
        }
    }

    fn scan_boundary(&self, start: usize, chars: &[char]) -> (usize, usize) {
        let len = chars.len();
        let mut end = start;
        
        while end + 1 < len && Self::is_sentence_terminator(chars[end + 1]) && chars[end + 1] != '\n' {
            end += 1;
        }
        
        while end + 1 < len && Self::is_trailing_char(chars[end + 1]) {
            end += 1;
        }
        
        let mut next_non_space = end + 1;
        while next_non_space < len && chars[next_non_space].is_whitespace() {
            next_non_space += 1;
        }
        
        (end, next_non_space)
    }

    fn get_token_before(chars: &[char], pos: usize, sentence_start: usize) -> Option<String> {
        if pos == 0 {
            return None;
        }
        
        let mut token_end = pos;
        let mut token_start = pos;
        
        while token_start > sentence_start && !chars[token_start - 1].is_whitespace() {
            token_start -= 1;
        }
        
        if token_start < token_end {
            Some(chars[token_start..token_end].iter().collect())
        } else {
            None
        }
    }
}

impl Default for TextSplitter {
    fn default() -> Self {
        Self::new(TextSplitterConfig::default())
    }
}

pub fn split_text(text: &str) -> Vec<String> {
    TextSplitter::default().split_into_sentences(text)
}

pub fn split_text_with_phoneme_limit(text: &str, max_phonemes: usize, get_phoneme_count: impl Fn(&str) -> usize) -> Vec<String> {
    let config = TextSplitterConfig {
        max_phonemes,
        ..Default::default()
    };
    TextSplitter::new(config).split_by_phoneme_limit(text, get_phoneme_count)
}
