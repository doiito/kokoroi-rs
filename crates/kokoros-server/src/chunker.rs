use regex::Regex;

pub struct TextChunker {
    sentence_pattern: Regex,
    max_chars: usize,
}

impl TextChunker {
    pub fn new(max_chars: usize) -> Self {
        let sentence_pattern = Regex::new(r"([。！？.!?]+)").unwrap();
        Self { sentence_pattern, max_chars }
    }
    
    pub fn split(&self, text: &str) -> Vec<String> {
        let mut chunks = Vec::new();
        
        let lines: Vec<&str> = text.split('\n').collect();
        
        for line in lines {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            
            let parts: Vec<&str> = self.sentence_pattern.split(line).collect();
            let mut current_chunk = String::new();
            
            let mut i = 0;
            while i < parts.len() {
                let mut sentence = parts[i].to_string();
                if i + 1 < parts.len() && self.sentence_pattern.is_match(parts[i + 1]) {
                    sentence.push_str(parts[i + 1]);
                    i += 1;
                }
                i += 1;
                
                if current_chunk.len() + sentence.len() <= self.max_chars {
                    current_chunk.push_str(&sentence);
                } else {
                    if !current_chunk.trim().is_empty() {
                        chunks.push(current_chunk.trim().to_string());
                    }
                    
                    if sentence.len() > self.max_chars {
                        for j in (0..sentence.len()).step_by(self.max_chars) {
                            let end = (j + self.max_chars).min(sentence.len());
                            let chunk = sentence[j..end].trim();
                            if !chunk.is_empty() {
                                chunks.push(chunk.to_string());
                            }
                        }
                        current_chunk = String::new();
                    } else {
                        current_chunk = sentence;
                    }
                }
            }
            
            if !current_chunk.trim().is_empty() {
                chunks.push(current_chunk.trim().to_string());
            }
        }
        
        chunks
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_split() {
        let chunker = TextChunker::new(400);
        let text = "银行里有很多重要的事情。今天天气很好，适合出去走走。";
        let chunks = chunker.split(text);
        assert!(!chunks.is_empty());
    }
}
