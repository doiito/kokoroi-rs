use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    static ref INITIAL_TO_IPA: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("b", "p");
        m.insert("p", "pʰ");
        m.insert("m", "m");
        m.insert("f", "f");
        m.insert("d", "t");
        m.insert("t", "tʰ");
        m.insert("n", "n");
        m.insert("l", "l");
        m.insert("g", "k");
        m.insert("k", "kʰ");
        m.insert("h", "x");
        m.insert("j", "ʨ");
        m.insert("q", "ʨʰ");
        m.insert("x", "ɕ");
        m.insert("zh", "ʈʂ");
        m.insert("ch", "ʈʂʰ");
        m.insert("sh", "ʂ");
        m.insert("r", "ɻ");
        m.insert("z", "ts");
        m.insert("c", "tsʰ");
        m.insert("s", "s");
        m
    };

    static ref FINAL_TO_IPA: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("a", "a");
        m.insert("ai", "ai̯");
        m.insert("an", "an");
        m.insert("ang", "aŋ");
        m.insert("ao", "au̯");
        m.insert("e", "ɤ");
        m.insert("ei", "ei̯");
        m.insert("en", "ən");
        m.insert("eng", "əŋ");
        m.insert("er", "ɚ");
        m.insert("i", "i");
        m.insert("ia", "ja");
        m.insert("ian", "jɛn");
        m.insert("iang", "jaŋ");
        m.insert("iao", "jau̯");
        m.insert("ie", "je");
        m.insert("in", "in");
        m.insert("ing", "iŋ");
        m.insert("iong", "jʊŋ");
        m.insert("iu", "jou̯");
        m.insert("iou", "jou̯");
        m.insert("o", "wo");
        m.insert("ong", "ʊŋ");
        m.insert("ou", "ou̯");
        m.insert("u", "u");
        m.insert("ua", "wa");
        m.insert("uai", "wai̯");
        m.insert("uan", "wan");
        m.insert("uang", "waŋ");
        m.insert("ue", "ɥe");
        m.insert("uei", "wei̯");
        m.insert("ui", "wei̯");
        m.insert("un", "wən");
        m.insert("uen", "wən");
        m.insert("ueng", "wəŋ");
        m.insert("uo", "wo");
        m.insert("v", "y");
        m.insert("ve", "ɥe");
        m.insert("van", "ɥɛn");
        m.insert("vn", "yn");
        m.insert("ii", "ɻ̩");
        m.insert("iii", "ɹ̩");
        m
    };

    static ref TONE_TO_IPA: HashMap<u8, &'static str> = {
        let mut m = HashMap::new();
        m.insert(1, "↗");
        m.insert(2, "→");
        m.insert(3, "↓");
        m.insert(4, "↘");
        m.insert(5, "");
        m
    };

    pub static ref ZH_MAP: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("b", "ㄅ");
        m.insert("p", "ㄆ");
        m.insert("m", "ㄇ");
        m.insert("f", "ㄈ");
        m.insert("d", "ㄉ");
        m.insert("t", "ㄊ");
        m.insert("n", "ㄋ");
        m.insert("l", "ㄌ");
        m.insert("g", "ㄍ");
        m.insert("k", "ㄎ");
        m.insert("h", "ㄏ");
        m.insert("j", "ㄐ");
        m.insert("q", "ㄑ");
        m.insert("x", "ㄒ");
        m.insert("zh", "ㄓ");
        m.insert("ch", "ㄔ");
        m.insert("sh", "ㄕ");
        m.insert("r", "ㄖ");
        m.insert("z", "ㄗ");
        m.insert("c", "ㄘ");
        m.insert("s", "ㄙ");
        m.insert("a", "ㄚ");
        m.insert("o", "ㄛ");
        m.insert("e", "ㄜ");
        m.insert("ie", "ㄝ");
        m.insert("ai", "ㄞ");
        m.insert("ei", "ㄟ");
        m.insert("ao", "ㄠ");
        m.insert("ou", "ㄡ");
        m.insert("an", "ㄢ");
        m.insert("en", "ㄣ");
        m.insert("ang", "ㄤ");
        m.insert("eng", "ㄥ");
        m.insert("er", "ㄦ");
        m.insert("i", "ㄧ");
        m.insert("u", "ㄨ");
        m.insert("v", "ㄩ");
        m.insert("ii", "ㄭ");
        m.insert("iii", "十");
        m.insert("ve", "月");
        m.insert("ia", "压");
        m.insert("ian", "言");
        m.insert("iang", "阳");
        m.insert("iao", "要");
        m.insert("in", "阴");
        m.insert("ing", "应");
        m.insert("iong", "用");
        m.insert("iou", "又");
        m.insert("iu", "又");
        m.insert("ong", "中");
        m.insert("ua", "穵");
        m.insert("uai", "外");
        m.insert("uan", "万");
        m.insert("uang", "王");
        m.insert("uei", "为");
        m.insert("ui", "为");
        m.insert("un", "文");
        m.insert("uen", "文");
        m.insert("ueng", "瓮");
        m.insert("uo", "我");
        m.insert("van", "元");
        m.insert("vn", "云");
        m.insert("1", "1");
        m.insert("2", "2");
        m.insert("3", "3");
        m.insert("4", "4");
        m.insert("5", "5");
        m.insert(";", ";");
        m.insert(":", ":");
        m.insert(",", ",");
        m.insert(".", ".");
        m.insert("!", "!");
        m.insert("?", "?");
        m.insert("/", "/");
        m.insert(" ", " ");
        m
    };

    static ref INITIALS: Vec<&'static str> = vec![
        "zh", "ch", "sh",
        "b", "p", "m", "f", "d", "t", "n", "l",
        "g", "k", "h", "j", "q", "x", "r", "z", "c", "s",
        "y", "w"
    ];
}

fn parse_pinyin(pinyin: &str) -> (Option<&str>, String, u8) {
    let pinyin = pinyin.to_lowercase();

    let (base, tone) = if let Some(last) = pinyin.chars().last() {
        if last.is_ascii_digit() {
            let tone = last.to_digit(10).unwrap_or(5) as u8;
            (&pinyin[..pinyin.len() - 1], tone)
        } else {
            (pinyin.as_str(), 5u8)
        }
    } else {
        return (None, String::new(), 5);
    };

    let mut initial: Option<&str> = None;
    let mut final_start = 0;

    for init in INITIALS.iter() {
        if base.starts_with(init) {
            initial = Some(init);
            final_start = init.len();
            break;
        }
    }

    let final_part = &base[final_start..];

    let final_part = match (initial, final_part) {
        (Some("z"), "i") | (Some("c"), "i") | (Some("s"), "i") => "ii",
        (Some("zh"), "i") | (Some("ch"), "i") | (Some("sh"), "i") | (Some("r"), "i") => "iii",
        (_, "iu") => "iou",
        (_, "ui") => "uei",
        (Some("j"), f) | (Some("q"), f) | (Some("x"), f)
            if f.starts_with('u') && !f.starts_with("ua") && !f.starts_with("uo") =>
        {
            &f.replacen('u', "v", 1).leak()
        }
        (Some("y"), "a") => "ia",
        (Some("y"), "ao") => "iao",
        (Some("y"), "an") => "ian",
        (Some("y"), "ang") => "iang",
        (Some("y"), "ou") => "iou",
        (Some("y"), "o") => "v",
        (Some("y"), "u") => "v",
        (Some("y"), "ue") => "ve",
        (Some("y"), "ve") => "ve",
        (Some("y"), "uan") => "van",
        (Some("y"), "un") => "vn",
        (Some("y"), "ong") => "iong",
        (Some("y"), "e") => "ie",
        (Some("w"), "a") => "ua",
        (Some("w"), "ai") => "uai",
        (Some("w"), "an") => "uan",
        (Some("w"), "ang") => "uang",
        (Some("w"), "ei") => "uei",
        (Some("w"), "en") => "uen",
        (Some("w"), "eng") => "ueng",
        (Some("w"), "o") => "uo",
        (_, "un") => "uen",
        _ => final_part,
    };

    (initial, final_part.to_string(), tone)
}

pub fn pinyin_to_ipa(pinyin: &str) -> String {
    let (initial, final_part, tone) = parse_pinyin(pinyin);

    let mut result = String::new();

    if let Some(init) = initial {
        if let Some(ipa) = INITIAL_TO_IPA.get(init) {
            result.push_str(ipa);
        }
    }

    if let Some(ipa) = FINAL_TO_IPA.get(final_part.as_str()) {
        result.push_str(ipa);
    } else {
        result.push_str(&final_part);
    }

    if let Some(tone_marker) = TONE_TO_IPA.get(&tone) {
        result.push_str(tone_marker);
    }

    result
}

pub fn pinyin_to_bopomofo(pinyin: &str) -> String {
    let (initial, final_part, tone) = parse_pinyin(pinyin);

    let mut result = String::new();

    if let Some(init) = initial {
        if init != "y" && init != "w" {
            if let Some(bpmf) = ZH_MAP.get(init) {
                result.push_str(bpmf);
            }
        }
    }

    if final_part == "ii" {
        if let Some(bpmf) = ZH_MAP.get("ii") {
            result.push_str(bpmf);
        }
    } else if final_part == "iii" {
        if let Some(bpmf) = ZH_MAP.get("iii") {
            result.push_str(bpmf);
        }
    } else if let Some(bpmf) = ZH_MAP.get(final_part.as_str()) {
        result.push_str(bpmf);
    } else {
        let mut s = String::new();
        for c in final_part.chars() {
            let cs = c.to_string();
            if let Some(b) = ZH_MAP.get(cs.as_str()) {
                s.push_str(b);
            }
        }
        result.push_str(&s);
    }

    result.push_str(&tone.to_string());

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_pinyin() {
        let (init, fin, tone) = parse_pinyin("zhong1");
        assert_eq!(init, Some("zh"));
        assert_eq!(fin, "ong");
        assert_eq!(tone, 1);

        let (init, fin, tone) = parse_pinyin("guo2");
        assert_eq!(init, Some("g"));
        assert_eq!(fin, "uo");
        assert_eq!(tone, 2);
    }

    #[test]
    fn test_pinyin_to_bopomofo() {
        let result = pinyin_to_bopomofo("zhong1");
        assert!(result.contains("ㄓ"), "Should contain ㄓ, got: {}", result);
        assert!(result.contains("ㄨㄥ"), "Should contain ㄨㄥ, got: {}", result);

        let result = pinyin_to_bopomofo("ni3");
        assert!(result.contains("ㄋ"), "Should contain ㄋ, got: {}", result);
        assert!(result.contains("ㄧ"), "Should contain ㄧ, got: {}", result);

        let result = pinyin_to_bopomofo("yin2");
        assert!(result.contains("ㄧ"), "Should contain ㄧ, got: {}", result);
        assert!(result.contains("ㄣ"), "Should contain ㄣ, got: {}", result);

        let result = pinyin_to_bopomofo("de5");
        assert!(result.contains("ㄉ"), "Should contain ㄉ, got: {}", result);
        assert!(result.contains("ㄜ"), "Should contain ㄜ, got: {}", result);
    }

    #[test]
    fn test_pinyin_to_ipa() {
        let result = pinyin_to_ipa("ma1");
        assert!(result.contains("m"));
        assert!(result.contains("a"));
        assert!(result.contains("↗"));
    }
}
