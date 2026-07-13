use crate::lexicon::Lexicon;
use super::LanguageRules;

pub struct English;

impl LanguageRules for English {
    fn apply_rules(&self, word: &str, tag: &str, lexicon: &Lexicon) -> Option<String> {
        let ctx = None;
        if let Some((ps, _)) = lexicon.stem_s(word, tag, None, ctx) {
            Some(ps)
        } else if let Some((ps, _)) = lexicon.stem_ed(word, tag, None, ctx) {
            Some(ps)
        } else if let Some((ps, _)) = lexicon.stem_ing(word, tag, None, ctx) {
            Some(ps)
        } else {
            None
        }
    }
}
