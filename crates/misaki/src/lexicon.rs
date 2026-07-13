use crate::data;
use crate::language::Language;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const LEXICON_ORDS: &[u32] = &[
    39, 45, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86,
    87, 88, 89, 90, 91, 97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111,
    112, 113, 114, 115, 116, 117, 118, 119, 120, 121, 122,
];
const US_TAUS: &str = "AIOWYiuæɑəɛɪɹʊʌ";

fn get_add_symbols() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    m.insert(".", "dot");
    m.insert("/", "slash");
    m
}

fn get_symbols() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    m.insert("%", "percent");
    m.insert("&", "and");
    m.insert("+", "plus");
    m.insert("@", "at");
    m
}

#[derive(Debug, Clone, Default)]
pub struct TokenContext {
    pub future_vowel: Option<bool>,
    pub future_to: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum PhonemeEntry {
    Simple(String),
    Tagged(HashMap<String, Option<String>>),
}

pub struct Lexicon {
    pub lang: Language,
    pub cap_stresses: (f64, f64),
    pub golds: HashMap<String, PhonemeEntry>,
    pub silvers: HashMap<String, PhonemeEntry>,
}

impl Lexicon {
    pub fn new(lang: Language) -> Self {
        let (golds_raw, silvers_raw) = match lang {
            Language::EnglishGB => (data::load_gb_gold(), data::load_gb_silver()),
            Language::EnglishUS => (data::load_us_gold(), data::load_us_silver()),
        };

        let golds = Lexicon::grow_dictionary(golds_raw);
        let silvers = Lexicon::grow_dictionary(silvers_raw);

        Self {
            lang,
            cap_stresses: (0.5, 2.0),
            golds,
            silvers,
        }
    }

    fn grow_dictionary(d: HashMap<String, PhonemeEntry>) -> HashMap<String, PhonemeEntry> {
        let mut e = HashMap::new();
        for (k, v) in d.iter() {
            if k.len() < 2 {
                continue;
            }
            let lower = k.to_lowercase();
            let capitalized = {
                let mut chars = lower.chars();
                match chars.next() {
                    None => String::new(),
                    Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
                }
            };

            if k == &lower {
                if k != &capitalized {
                    e.insert(capitalized, v.clone());
                }
            } else if k == &capitalized {
                e.insert(lower, v.clone());
            }
        }
        let mut result = d;
        result.extend(e);
        result
    }

    fn resolve_phonemes(
        &self,
        entry: &PhonemeEntry,
        tag: &str,
        ctx: Option<&TokenContext>,
    ) -> Option<String> {
        match entry {
            PhonemeEntry::Simple(ps) => Some(ps.clone()),
            PhonemeEntry::Tagged(map) => {
                let mut current_tag = tag;
                if let Some(context) = ctx
                    && context.future_vowel.is_none() && map.contains_key("None") {
                        current_tag = "None";
                    }

                if let Some(Some(ps)) = map.get(current_tag) {
                    return Some(ps.clone());
                }
                let parent = Lexicon::get_parent_tag(current_tag);
                if let Some(Some(ps)) = map.get(parent) {
                    return Some(ps.clone());
                }
                map.get("DEFAULT").and_then(|opt| opt.clone())
            }
        }
    }

    pub fn get_parent_tag(tag: &str) -> &str {
        if tag.starts_with("VB") {
            "VERB"
        } else if tag.starts_with("NN") {
            "NOUN"
        } else if tag.starts_with("ADV") || tag.starts_with("RB") {
            "ADV"
        } else if tag.starts_with("ADJ") || tag.starts_with("JJ") {
            "ADJ"
        } else {
            tag
        }
    }

    pub fn lookup(
        &self,
        word: &str,
        tag: &str,
        stress: Option<f64>,
        ctx: Option<&TokenContext>,
    ) -> Option<(String, i32)> {
        let mut current_word = word.to_string();
        let mut is_nnp = false;

        if word == word.to_uppercase() && !self.golds.contains_key(word) {
            current_word = word.to_lowercase();
            is_nnp = tag == "NNP";
        }

        let mut ps = None;
        let mut rating = 0;

        if let Some(entry) = self.golds.get(&current_word) {
            ps = self.resolve_phonemes(entry, tag, ctx);
            rating = 4;
        }

        if ps.is_none() && !is_nnp
            && let Some(entry) = self.silvers.get(&current_word) {
                ps = self.resolve_phonemes(entry, tag, ctx);
                rating = 3;
            }

        if (ps.is_none() || (is_nnp && !ps.as_ref()?.contains('ˈ')))
            && is_nnp
                && let Some((nnp_ps, nnp_rating)) = self.get_nnp(&current_word) {
                    ps = Some(nnp_ps);
                    rating = nnp_rating;
                }

        ps.map(|p| (self.apply_stress(&p, stress), rating))
    }

    fn get_nnp(&self, word: &str) -> Option<(String, i32)> {
        let mut ps_parts = Vec::new();
        for c in word.chars() {
            if c.is_alphabetic() {
                if let Some(entry) = self.golds.get(&c.to_uppercase().to_string()) {
                    if let PhonemeEntry::Simple(p) = entry {
                        ps_parts.push(p.clone());
                    }
                } else {
                    return None;
                }
            }
        }
        if ps_parts.is_empty() {
            return None;
        }

        let combined = ps_parts.join("");
        let stressed = self.apply_stress(&combined, Some(0.0));

        let secondary = 'ˌ';
        let primary = 'ˈ';
        if let Some(idx) = stressed.rfind(secondary) {
            let mut result = stressed.clone();
            result.replace_range(idx..idx + secondary.len_utf8(), &primary.to_string());
            Some((result, 3))
        } else {
            Some((stressed, 3))
        }
    }

    pub fn apply_stress(&self, ps: &str, stress: Option<f64>) -> String {
        let primary = 'ˈ';
        let secondary = 'ˌ';
        let vowels = "AIOQWYaiuæɑɒɔəɛɜɪʊʌᵻ";

        if stress.is_none() {
            return ps.to_string();
        }
        let s = stress.unwrap();

        if s < -1.0 {
            return ps.replace([primary, secondary], "");
        } else if s == -1.0 || ((-0.5..=0.0).contains(&s) && ps.contains(primary)) {
            return ps
                .replace(secondary, "")
                .replace(primary, &secondary.to_string());
        } else if (s == 0.0 || s == 0.5 || s == 1.0)
            && !ps.contains(primary)
            && !ps.contains(secondary)
        {
            if !ps.chars().any(|c| vowels.contains(c)) {
                return ps.to_string();
            }
            return self.restress(&format!("{}{}", secondary, ps));
        } else if s >= 1.0 && !ps.contains(primary) && ps.contains(secondary) {
            return ps.replace(secondary, &primary.to_string());
        } else if s > 1.0 && !ps.contains(primary) && !ps.contains(secondary) {
            if !ps.chars().any(|c| vowels.contains(c)) {
                return ps.to_string();
            }
            return self.restress(&format!("{}{}", primary, ps));
        }
        ps.to_string()
    }

    fn restress(&self, ps: &str) -> String {
        let primary = 'ˈ';
        let secondary = 'ˌ';
        let vowels = "AIOQWYaiuæɑɒɔəɛɜɪʊʌᵻ";

        let mut parts: Vec<(f64, char)> =
            ps.chars().enumerate().map(|(i, c)| (i as f64, c)).collect();
        let mut stresses = Vec::new();

        for (i, &(_, c)) in parts.iter().enumerate() {
            if (c == primary || c == secondary)
                && let Some(j) = parts[i..].iter().position(|&(_, vc)| vowels.contains(vc)) {
                    stresses.push((i, i + j));
                }
        }

        for (si, vi) in stresses {
            let (_s_pos, s_char) = parts[si];
            parts[si] = (parts[vi].0 - 0.5, s_char);
        }

        parts.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        parts.into_iter().map(|(_, c)| c).collect()
    }

    pub fn stem_s(
        &self,
        word: &str,
        tag: &str,
        stress: Option<f64>,
        ctx: Option<&TokenContext>,
    ) -> Option<(String, i32)> {
        let lower = word.to_lowercase();
        if lower.len() < 3 || !lower.ends_with('s') {
            return None;
        }

        let stem = if !lower.ends_with("ss") && self.is_known(&lower[..lower.len() - 1], tag) {
            &lower[..lower.len() - 1]
        } else if (lower.ends_with("'s")
            || (lower.len() > 4 && lower.ends_with("es") && !lower.ends_with("ies")))
            && self.is_known(&lower[..lower.len() - 2], tag)
        {
            &lower[..lower.len() - 2]
        } else if lower.len() > 4
            && lower.ends_with("ies")
            && self.is_known(&(lower[..lower.len() - 3].to_string() + "y"), tag)
        {
            &(lower[..lower.len() - 3].to_string() + "y")
        } else {
            return None;
        };

        let (stem_ps, rating) = self.lookup(stem, tag, stress, ctx)?;
        Some((self.append_s(&stem_ps), rating))
    }

    pub fn append_s(&self, stem: &str) -> String {
        if stem.is_empty() {
            return String::new();
        }
        let last = stem.chars().last().unwrap();
        let british = matches!(self.lang, Language::EnglishGB);
        if "ptkfθ".contains(last) {
            format!("{}s", stem)
        } else if "szʃʒʧʤ".contains(last) {
            format!("{}{}z", stem, if british { "ɪ" } else { "ᵻ" })
        } else {
            format!("{}z", stem)
        }
    }

    pub fn stem_ed(
        &self,
        word: &str,
        tag: &str,
        stress: Option<f64>,
        ctx: Option<&TokenContext>,
    ) -> Option<(String, i32)> {
        let lower = word.to_lowercase();
        if lower.len() < 4 || !lower.ends_with('d') {
            return None;
        }
        let stem = if !lower.ends_with("dd") && self.is_known(&lower[..lower.len() - 1], tag) {
            &lower[..lower.len() - 1]
        } else if lower.len() > 4
            && lower.ends_with("ed")
            && !lower.ends_with("eed")
            && self.is_known(&lower[..lower.len() - 2], tag)
        {
            &lower[..lower.len() - 2]
        } else {
            return None;
        };

        let (stem_ps, rating) = self.lookup(stem, tag, stress, ctx)?;
        Some((self.append_ed(&stem_ps), rating))
    }

    pub fn append_ed(&self, stem: &str) -> String {
        if stem.is_empty() {
            return String::new();
        }
        let british = matches!(self.lang, Language::EnglishGB);
        let last = stem.chars().last().unwrap();
        if "pkfθʃsʧ".contains(last) {
            format!("{}t", stem)
        } else if last == 'd' {
            format!("{}{}d", stem, if british { "ɪ" } else { "ᵻ" })
        } else if last != 't' {
            format!("{}d", stem)
        } else if british || stem.len() < 2 {
            format!("{}ɪd", stem)
        } else {
            let chars: Vec<char> = stem.chars().collect();
            if chars.len() >= 2 && US_TAUS.contains(chars[chars.len() - 2]) {
                format!(
                    "{}ɾᵻd",
                    &stem[..stem.len() - chars[chars.len() - 1].len_utf8()]
                )
            } else {
                format!("{}ᵻd", stem)
            }
        }
    }

    pub fn append_ing(&self, stem: &str) -> Option<String> {
        if stem.is_empty() {
            return None;
        }
        let british = matches!(self.lang, Language::EnglishGB);

        if british {
            let last = stem.chars().last().unwrap();
            if last == 'ə' || last == 'ː' {
                return None;
            }
        }

        if !british && stem.len() > 1 {
            let chars: Vec<char> = stem.chars().collect();
            if chars[chars.len() - 1] == 't'
                && chars.len() >= 2
                && US_TAUS.contains(chars[chars.len() - 2])
            {
                return Some(format!(
                    "{}ɾɪŋ",
                    &stem[..stem.len() - chars[chars.len() - 1].len_utf8()]
                ));
            }
        }

        Some(format!("{}ɪŋ", stem))
    }

    pub fn stem_ing(
        &self,
        word: &str,
        tag: &str,
        stress: Option<f64>,
        ctx: Option<&TokenContext>,
    ) -> Option<(String, i32)> {
        let lower = word.to_lowercase();
        if lower.len() < 5 || !lower.ends_with("ing") {
            return None;
        }

        let stem = if lower.len() > 5 && self.is_known(&lower[..lower.len() - 3], tag) {
            lower[..lower.len() - 3].to_string()
        } else if self.is_known(&(lower[..lower.len() - 3].to_string() + "e"), tag) {
            lower[..lower.len() - 3].to_string() + "e"
        } else if lower.len() > 5 {
            let stem_candidate = &lower[..lower.len() - 4];
            if self.is_known(stem_candidate, tag) {
                let chars: Vec<char> = stem_candidate.chars().collect();
                if chars.len() >= 2 {
                    let last = chars[chars.len() - 1];
                    let second_last = chars[chars.len() - 2];
                    if (last == second_last && "bcdgklmnprstvxz".contains(last))
                        || (last == 'k' && second_last == 'c')
                    {
                        return Some((
                            self.append_ing(stem_candidate)?,
                            self.lookup(stem_candidate, tag, stress, ctx)?.1,
                        ));
                    }
                }
            }
            return None;
        } else {
            return None;
        };

        let (stem_ps, rating) = self.lookup(&stem, tag, stress, ctx)?;
        Some((self.append_ing(&stem_ps)?, rating))
    }

    pub fn get_special_case(
        &self,
        word: &str,
        tag: &str,
        stress: Option<f64>,
        ctx: Option<&TokenContext>,
    ) -> Option<(String, i32)> {
        let add_symbols = get_add_symbols();
        let symbols = get_symbols();

        if tag == "ADD" && add_symbols.contains_key(word) {
            return self.lookup(add_symbols[word], "NN", Some(-0.5), ctx);
        } else if symbols.contains_key(word) {
            return self.lookup(symbols[word], "NN", None, ctx);
        } else if word.contains('.') && word.replace('.', "").chars().all(|c| c.is_alphabetic()) {
            let max_len = word.split('.').map(|s| s.len()).max().unwrap_or(0);
            if max_len < 3 {
                return self.get_nnp(word);
            }
        } else if word == "a" || word == "A" {
            return Some((
                if tag == "DT" { "ɐ".to_string() } else { "ˈA".to_string() },
                4,
            ));
        } else if word == "am" || word == "Am" || word == "AM" {
            if tag.starts_with("NN") {
                return self.get_nnp(word);
            } else if (ctx.is_none()
                || ctx.and_then(|c| c.future_vowel).is_none()
                || word != "am"
                || stress.map(|s| s > 0.0).unwrap_or(false))
                && let Some(PhonemeEntry::Simple(ps)) = self.golds.get("am") {
                    return Some((ps.clone(), 4));
                }
            return Some(("ɐm".to_string(), 4));
        } else if word == "an" || word == "An" || word == "AN" {
            if word == "AN" && tag.starts_with("NN") {
                return self.get_nnp(word);
            }
            return Some(("ɐn".to_string(), 4));
        } else if word == "I" && tag == "PRP" {
            return Some(("ˌI".to_string(), 4));
        } else if (word == "by" || word == "By" || word == "BY")
            && Lexicon::get_parent_tag(tag) == "ADV"
        {
            return Some(("bˈI".to_string(), 4));
        } else if word == "to" || word == "To" || (word == "TO" && (tag == "TO" || tag == "IN")) {
            let future_vowel = ctx.and_then(|c| c.future_vowel);
            if let Some(PhonemeEntry::Simple(ps)) = self.golds.get("to") {
                return Some((
                    match future_vowel {
                        None => ps.clone(),
                        Some(false) => "tə".to_string(),
                        Some(true) => "tʊ".to_string(),
                    },
                    4,
                ));
            }
        } else if word == "in" || word == "In" || (word == "IN" && tag != "NNP") {
            let future_vowel = ctx.and_then(|c| c.future_vowel);
            let stress_mark = if future_vowel.is_none() || tag != "IN" {
                "ˈ"
            } else {
                ""
            };
            return Some((format!("{}{}", stress_mark, "ɪn"), 4));
        } else if word == "the" || word == "The" || (word == "THE" && tag == "DT") {
            let future_vowel = ctx.and_then(|c| c.future_vowel);
            return Some((
                if future_vowel == Some(true) { "ði".to_string() } else { "ðə".to_string() },
                4,
            ));
        } else if tag == "IN" && (word.to_lowercase() == "vs" || word.to_lowercase() == "vs.") {
            return self.lookup("versus", "NN", None, ctx);
        } else if word == "used" || word == "Used" || word == "USED" {
            if (tag == "VBD" || tag == "JJ") && ctx.map(|c| c.future_to).unwrap_or(false)
                && let Some(PhonemeEntry::Tagged(map)) = self.golds.get("used")
                    && let Some(Some(ps)) = map.get("VBD") {
                        return Some((ps.clone(), 4));
                    }
            if let Some(PhonemeEntry::Tagged(map)) = self.golds.get("used")
                && let Some(Some(ps)) = map.get("DEFAULT") {
                    return Some((ps.clone(), 4));
                }
        }
        None
    }

    pub fn is_known(&self, word: &str, _tag: &str) -> bool {
        let symbols = get_symbols();

        if self.golds.contains_key(word)
            || symbols.contains_key(word)
            || self.silvers.contains_key(word)
        {
            return true;
        }

        if !word.chars().all(|c| c.is_alphabetic()) {
            return false;
        }

        if !word.chars().all(|c| {
            let ord = c as u32;
            LEXICON_ORDS.contains(&ord)
        }) {
            return false;
        }

        if word.len() == 1 {
            return true;
        }

        if word == word.to_uppercase() && self.golds.contains_key(&word.to_lowercase()) {
            return true;
        }

        if word.len() > 1 {
            let rest: String = word.chars().skip(1).collect();
            if rest == rest.to_uppercase() {
                return true;
            }
        }

        false
    }

    pub fn get_word(
        &self,
        word: &str,
        tag: &str,
        stress: Option<f64>,
        ctx: Option<&TokenContext>,
    ) -> Option<(String, i32)> {
        if let Some(result) = self.get_special_case(word, tag, stress, ctx) {
            return Some(result);
        }

        let wl = word.to_lowercase();
        let mut current_word = word;

        if word.len() > 1
            && word.replace("'", "").chars().all(|c| c.is_alphabetic())
            && word != wl
            && (tag != "NNP" || word.len() > 7)
            && !self.golds.contains_key(word)
            && !self.silvers.contains_key(word)
            && (word == word.to_uppercase() || {
                let rest: String = word.chars().skip(1).collect();
                rest == rest.to_lowercase()
            })
            && (self.golds.contains_key(&wl)
                || self.silvers.contains_key(&wl)
                || self.stem_s(&wl, tag, stress, ctx).is_some()
                || self.stem_ed(&wl, tag, stress, ctx).is_some()
                || self.stem_ing(&wl, tag, stress, ctx).is_some())
        {
            current_word = &wl;
        }

        if self.is_known(current_word, tag) {
            return self.lookup(current_word, tag, stress, ctx);
        }

        if current_word.ends_with("s'")
            && self.is_known(&current_word[..current_word.len() - 2], tag)
        {
            return self.lookup(
                &format!("{}'s", &current_word[..current_word.len() - 2]),
                tag,
                stress,
                ctx,
            );
        }
        if current_word.ends_with("'")
            && self.is_known(&current_word[..current_word.len() - 1], tag)
        {
            return self.lookup(&current_word[..current_word.len() - 1], tag, stress, ctx);
        }

        if let Some(result) = self.stem_s(current_word, tag, stress, ctx) {
            return Some(result);
        }
        if let Some(result) = self.stem_ed(current_word, tag, stress, ctx) {
            return Some(result);
        }
        if let Some(result) = self.stem_ing(current_word, tag, Some(0.5).or(stress), ctx) {
            return Some(result);
        }

        None
    }
}
