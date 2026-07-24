use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::LazyLock;

pub struct Vocab {
    token_to_id: HashMap<String, u32>,
    id_to_token: HashMap<u32, String>,
}

impl Vocab {
    pub fn new() -> Self {
        Self {
            token_to_id: HashMap::new(),
            id_to_token: HashMap::new(),
        }
    }

    pub fn from_tokens(tokens: &[&str]) -> Self {
        let mut vocab = Self::new();
        for (id, token) in tokens.iter().enumerate() {
            vocab.add_token(token, id as u32);
        }
        vocab
    }

    pub fn add_token(&mut self, token: &str, id: u32) {
        self.token_to_id.insert(token.to_string(), id);
        self.id_to_token.insert(id, token.to_string());
    }

    pub fn token_to_id(&self, token: &str) -> Option<u32> {
        self.token_to_id.get(token).copied()
    }

    pub fn id_to_token(&self, id: u32) -> Option<&str> {
        self.id_to_token.get(&id).map(|s| s.as_str())
    }

    pub fn len(&self) -> usize {
        self.token_to_id.len()
    }

    pub fn is_empty(&self) -> bool {
        self.token_to_id.is_empty()
    }
}

impl Default for Vocab {
    fn default() -> Self {
        Self::new()
    }
}

fn build_vocab() -> HashMap<char, usize> {
    let pad = "$";
    let punctuation = ";:,.!?\u{00A1}\u{00BF}\u{2014}\u{2026}\"\u{00AB}\u{00BB}\u{201C}\u{201D} ";
    let letters = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
    let letters_ipa = "ɑɐɒæɓʙβɔɕçɗɖðʤəɘɚɛɜɝɞɟʄɡɠɢʛɦɧħɥʜɨɪʝɭɬɫɮʟɱɯɰŋɳɲɴøɵɸθœɶʘɹɺɾɻʀʁɽʂʃʈʧʉʊʋⱱʌɣɤʍχʎʏʑʐʒʔʡʕʢǀǁǂǃˈˌːˑʼʴʰʱʲʷˠˤ˞↓↑→↗↘'̩'ᵻ";

    let symbols: String = [pad, punctuation, letters, letters_ipa].concat();

    symbols
        .chars()
        .enumerate()
        .collect::<HashMap<_, _>>()
        .into_iter()
        .map(|(idx, c)| (c, idx))
        .collect()
}

fn build_zh_vocab() -> HashMap<char, usize> {
    let mut vocab = HashMap::new();

    let mappings: &[(&str, usize)] = &[
        ("$", 0),
        (";", 1),
        (":", 2),
        (",", 3),
        (".", 4),
        ("!", 5),
        ("?", 6),
        ("/", 7),
        ("—", 9),
        ("…", 10),
        ("\"", 11),
        ("(", 12),
        (")", 13),
        ("\u{201C}", 14),
        ("\u{201D}", 15),
        (" ", 16),
        ("\u{0303}", 17),
        ("ʣ", 18),
        ("ʥ", 19),
        ("ʦ", 20),
        ("ʨ", 21),
        ("ᵝ", 22),
        ("ㄓ", 23),
        ("A", 24),
        ("I", 25),
        ("ㄅ", 30),
        ("O", 31),
        ("ㄆ", 32),
        ("Q", 33),
        ("R", 34),
        ("S", 35),
        ("T", 36),
        ("ㄇ", 37),
        ("ㄈ", 38),
        ("W", 39),
        ("ㄉ", 40),
        ("Y", 41),
        ("ᵊ", 42),
        ("a", 43),
        ("b", 44),
        ("c", 45),
        ("d", 46),
        ("e", 47),
        ("f", 48),
        ("ㄊ", 49),
        ("h", 50),
        ("i", 51),
        ("j", 52),
        ("k", 53),
        ("l", 54),
        ("m", 55),
        ("n", 56),
        ("o", 57),
        ("p", 58),
        ("q", 59),
        ("r", 60),
        ("s", 61),
        ("t", 62),
        ("u", 63),
        ("v", 64),
        ("w", 65),
        ("x", 66),
        ("y", 67),
        ("z", 68),
        ("ɑ", 69),
        ("ɐ", 70),
        ("ɒ", 71),
        ("æ", 72),
        ("ㄋ", 73),
        ("ㄌ", 74),
        ("β", 75),
        ("ɔ", 76),
        ("ɕ", 77),
        ("ç", 78),
        ("ㄍ", 79),
        ("ɖ", 80),
        ("ð", 81),
        ("ʤ", 82),
        ("ə", 83),
        ("ㄎ", 84),
        ("ㄦ", 85),
        ("ɛ", 86),
        ("ɜ", 87),
        ("ㄏ", 88),
        ("ㄐ", 89),
        ("ɟ", 90),
        ("ㄑ", 91),
        ("ɡ", 92),
        ("ㄒ", 93),
        ("ㄔ", 94),
        ("ㄕ", 95),
        ("ㄗ", 96),
        ("ㄘ", 97),
        ("ㄙ", 98),
        ("月", 99),
        ("ㄚ", 100),
        ("ɨ", 101),
        ("ɪ", 102),
        ("ʝ", 103),
        ("ㄛ", 104),
        ("ㄝ", 105),
        ("ㄞ", 106),
        ("ㄟ", 107),
        ("ㄠ", 108),
        ("ㄡ", 109),
        ("ɯ", 110),
        ("ɰ", 111),
        ("ŋ", 112),
        ("ɳ", 113),
        ("ɲ", 114),
        ("ɴ", 115),
        ("ø", 116),
        ("ㄢ", 117),
        ("ɸ", 118),
        ("θ", 119),
        ("œ", 120),
        ("ㄣ", 121),
        ("ㄤ", 122),
        ("ɹ", 123),
        ("ㄥ", 124),
        ("ɾ", 125),
        ("ㄖ", 126),
        ("ㄧ", 127),
        ("ʁ", 128),
        ("ɽ", 129),
        ("ʂ", 130),
        ("ʃ", 131),
        ("ʈ", 132),
        ("ʧ", 133),
        ("ㄨ", 134),
        ("ʊ", 135),
        ("ʋ", 136),
        ("ㄩ", 137),
        ("ʌ", 138),
        ("ɣ", 139),
        ("ㄜ", 140),
        ("ㄭ", 141),
        ("χ", 142),
        ("ʎ", 143),
        ("十", 144),
        ("压", 145),
        ("言", 146),
        ("ʒ", 147),
        ("ʔ", 148),
        ("阳", 149),
        ("要", 150),
        ("阴", 151),
        ("应", 152),
        ("用", 153),
        ("又", 154),
        ("中", 155),
        ("ˈ", 156),
        ("ˌ", 157),
        ("ː", 158),
        ("穵", 159),
        ("外", 160),
        ("万", 161),
        ("ʰ", 162),
        ("王", 163),
        ("ʲ", 164),
        ("为", 165),
        ("文", 166),
        ("瓮", 167),
        ("我", 168),
        ("3", 169),
        ("5", 170),
        ("1", 171),
        ("2", 172),
        ("4", 173),
        ("元", 175),
        ("云", 176),
        ("ᵻ", 177),
    ];

    for (s, idx) in mappings {
        for c in s.chars() {
            vocab.insert(c, *idx);
        }
    }

    vocab
}

lazy_static! {
    pub static ref VOCAB: HashMap<char, usize> = build_vocab();
    pub static ref REVERSE_VOCAB: HashMap<usize, char> = VOCAB.iter().map(|(&c, &idx)| (idx, c)).collect();
    pub static ref ZH_VOCAB: HashMap<char, usize> = build_zh_vocab();
    pub static ref ZH_REVERSE_VOCAB: HashMap<usize, char> = ZH_VOCAB.iter().map(|(&c, &idx)| (idx, c)).collect();
}

/// Model vocabulary from kokoro-onnx config.json — exact character→index mappings
/// the public kokoro-v1.0.onnx model was trained with (n_token=178, 114 entries).
pub static MODEL_VOCAB: LazyLock<HashMap<char, usize>> = LazyLock::new(|| {
    let entries: &[(&str, usize)] = &[
        (";", 1),
        (":", 2),
        (",", 3),
        (".", 4),
        ("!", 5),
        ("?", 6),
        ("—", 9),
        ("…", 10),
        ("\"", 11),
        ("(", 12),
        (")", 13),
        ("\u{201C}", 14),
        ("\u{201D}", 15),
        (" ", 16),
        ("\u{0303}", 17),
        ("ʣ", 18),
        ("ʥ", 19),
        ("ʦ", 20),
        ("ʨ", 21),
        ("ᵝ", 22),
        ("ꭧ", 23),
        ("A", 24),
        ("I", 25),
        ("O", 31),
        ("Q", 33),
        ("S", 35),
        ("T", 36),
        ("W", 39),
        ("Y", 41),
        ("ᵊ", 42),
        ("a", 43),
        ("b", 44),
        ("c", 45),
        ("d", 46),
        ("e", 47),
        ("f", 48),
        ("h", 50),
        ("i", 51),
        ("j", 52),
        ("k", 53),
        ("l", 54),
        ("m", 55),
        ("n", 56),
        ("o", 57),
        ("p", 58),
        ("q", 59),
        ("r", 60),
        ("s", 61),
        ("t", 62),
        ("u", 63),
        ("v", 64),
        ("w", 65),
        ("x", 66),
        ("y", 67),
        ("z", 68),
        ("ɑ", 69),
        ("ɐ", 70),
        ("ɒ", 71),
        ("æ", 72),
        ("β", 75),
        ("ɔ", 76),
        ("ɕ", 77),
        ("ç", 78),
        ("ɖ", 80),
        ("ð", 81),
        ("ʤ", 82),
        ("ə", 83),
        ("ɚ", 85),
        ("ɛ", 86),
        ("ɜ", 87),
        ("ɟ", 90),
        ("ɡ", 92),
        ("ɥ", 99),
        ("ɨ", 101),
        ("ɪ", 102),
        ("ʝ", 103),
        ("ɯ", 110),
        ("ɰ", 111),
        ("ŋ", 112),
        ("ɳ", 113),
        ("ɲ", 114),
        ("ɴ", 115),
        ("ø", 116),
        ("ɸ", 118),
        ("θ", 119),
        ("œ", 120),
        ("ɹ", 123),
        ("ɾ", 125),
        ("ɻ", 126),
        ("ʁ", 128),
        ("ɽ", 129),
        ("ʂ", 130),
        ("ʃ", 131),
        ("ʈ", 132),
        ("ʧ", 133),
        ("ʊ", 135),
        ("ʋ", 136),
        ("ʌ", 138),
        ("ɣ", 139),
        ("ɤ", 140),
        ("χ", 142),
        ("ʎ", 143),
        ("ʒ", 147),
        ("ʔ", 148),
        ("ˈ", 156),
        ("ˌ", 157),
        ("ː", 158),
        ("ʰ", 162),
        ("ʲ", 164),
        ("↓", 169),
        ("→", 171),
        ("↗", 172),
        ("↘", 173),
        ("ᵻ", 177),
    ];
    let mut m = HashMap::new();
    for (s, idx) in entries {
        for c in s.chars() {
            m.insert(c, *idx);
        }
    }
    m
});

pub static REVERSE_MODEL_VOCAB: LazyLock<HashMap<usize, char>> = LazyLock::new(|| {
    MODEL_VOCAB.iter().map(|(&c, &idx)| (idx, c)).collect()
});
