use crate::tts::vocab::{MODEL_VOCAB, VOCAB, ZH_VOCAB};

/// Tokenize using the public model's exact vocabulary (kokoro-v1.0.onnx).
/// Characters not in the model's vocab are silently skipped.
pub fn tokenize(phonemes: &str) -> Vec<i64> {
    phonemes
        .chars()
        .filter_map(|c| MODEL_VOCAB.get(&c))
        .map(|&idx| idx as i64)
        .collect()
}

/// Tokenize using the bopomofo ZH_VOCAB (for v1.1-zh models).
pub fn tokenize_zh(phonemes: &str) -> Vec<i64> {
    phonemes
        .chars()
        .filter_map(|c| ZH_VOCAB.get(&c))
        .map(|&idx| idx as i64)
        .collect()
}

/// Tokenize using the positional VOCAB (for legacy models).
pub fn tokenize_ipa(phonemes: &str) -> Vec<i64> {
    phonemes
        .chars()
        .filter_map(|c| VOCAB.get(&c))
        .map(|&idx| idx as i64)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize() {
        let text = "heɪ ðɪs ɪz ˈlʌvliː!";
        let tokens = tokenize(text);

        assert!(!tokens.is_empty(), "Should produce tokens for IPA text");
        assert!(tokens.len() > 5, "Should have multiple tokens for IPA text");

        let empty = "";
        let empty_tokens = tokenize(empty);
        assert!(empty_tokens.is_empty());

        let punct = "...";
        let punct_tokens = tokenize(punct);
        assert_eq!(punct_tokens.len(), 3);
    }

    #[test]
    fn test_tokenize_basic_chars() {
        let text = "nil!";
        let tokens = tokenize(text);
        assert!(!tokens.is_empty());
        // n=56 i=51 l=54 !=5 = 4 tokens (all in MODEL_VOCAB)
        assert!(tokens.len() == 4, "Expected 4 tokens, got {:?}", tokens);
    }
}

use crate::tts::vocab::REVERSE_MODEL_VOCAB;

pub fn tokens_to_phonemes(tokens: &[i64]) -> String {
    tokens
        .iter()
        .filter_map(|&t| REVERSE_MODEL_VOCAB.get(&(t as usize)))
        .collect()
}

#[cfg(test)]
mod tests2 {
    use super::*;

    #[test]
    fn test_tokens_to_phonemes_roundtrip() {
        // Use IPA chars that are all in MODEL_VOCAB
        let text = "nɪlɔ!";
        let tokens = tokenize(text);
        let recovered = tokens_to_phonemes(&tokens);
        assert_eq!(recovered, text);
    }

    #[test]
    fn test_tokens_to_phonemes_empty() {
        let empty_tokens: Vec<i64> = vec![];
        assert_eq!(tokens_to_phonemes(&empty_tokens), "");
    }
}
