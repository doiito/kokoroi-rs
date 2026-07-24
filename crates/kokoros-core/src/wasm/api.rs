use crate::tts::chinese::ChineseG2P;
use crate::tts::vocab::MODEL_VOCAB;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[cfg(feature = "oxionnx")]
use crate::onn::OnnxInference;

#[derive(Serialize, Deserialize)]
pub struct KokoroConfig {
    #[serde(rename = "usePolyphonic")]
    pub use_polyphonic: Option<bool>,
}

#[derive(Serialize, Deserialize)]
pub struct VoiceInfo {
    pub id: String,
    pub name: String,
    pub language: String,
    pub gender: String,
}

lazy_static::lazy_static! {
    static ref TOKEN_TO_BOPOMOFO: std::collections::HashMap<char, &'static str> = {
        let mut m = std::collections::HashMap::new();
        m.insert('月', "ㄩㄝ");
        m.insert('十', "ㄭ");
        m.insert('压', "ㄧㄚ");
        m.insert('言', "ㄧㄢ");
        m.insert('阳', "ㄧㄤ");
        m.insert('要', "ㄧㄠ");
        m.insert('阴', "ㄧㄣ");
        m.insert('应', "ㄧㄥ");
        m.insert('用', "ㄩㄥ");
        m.insert('又', "ㄧㄡ");
        m.insert('中', "ㄨㄥ");
        m.insert('穵', "ㄨㄚ");
        m.insert('外', "ㄨㄞ");
        m.insert('万', "ㄨㄢ");
        m.insert('王', "ㄨㄤ");
        m.insert('为', "ㄨㄟ");
        m.insert('文', "ㄨㄣ");
        m.insert('瓮', "ㄨㄥ");
        m.insert('我', "ㄨㄛ");
        m.insert('元', "ㄩㄢ");
        m.insert('云', "ㄩㄣ");
        m
    };
}

fn convert_tokens_to_bopomofo(tokens: &str) -> String {
    tokens
        .chars()
        .map(|c| {
            if let Some(bpmf) = TOKEN_TO_BOPOMOFO.get(&c) {
                bpmf.to_string()
            } else {
                c.to_string()
            }
        })
        .collect()
}

#[wasm_bindgen]
pub struct KokoroWASM {
    #[allow(dead_code)]
    chinese_g2p: ChineseG2P,
    chinese_g2p_ipa: ChineseG2P,
    #[cfg(feature = "oxionnx")]
    model: Option<OnnxInference>,
    sample_rate: u32,
}

#[wasm_bindgen]
impl KokoroWASM {
    #[wasm_bindgen(constructor)]
    pub fn new(config: JsValue) -> Result<KokoroWASM, JsValue> {
        let config: KokoroConfig = serde_wasm_bindgen::from_value(config)?;

        let chinese_g2p = if config.use_polyphonic.unwrap_or(true) {
            ChineseG2P::new()
        } else {
            ChineseG2P::without_polyphonic()
        };
        
        let chinese_g2p_ipa = ChineseG2P::with_ipa();

        Ok(Self {
            chinese_g2p,
            chinese_g2p_ipa,
            #[cfg(feature = "oxionnx")]
            model: None,
            sample_rate: 24000,
        })
    }

    #[cfg(feature = "oxionnx")]
    #[wasm_bindgen(js_name = loadModel)]
    pub fn load_model(&mut self, model_bytes: &[u8]) -> Result<(), JsValue> {
        let model = OnnxInference::from_bytes(model_bytes)
            .map_err(|e| JsValue::from_str(&format!("Failed to load model: {}", e)))?;
        self.model = Some(model);
        Ok(())
    }

    #[cfg(feature = "oxionnx")]
    #[wasm_bindgen(js_name = isModelLoaded)]
    pub fn is_model_loaded(&self) -> bool {
        self.model.is_some()
    }

    #[wasm_bindgen(js_name = phonemize)]
    pub fn phonemize(&self, text: &str, lang: Option<String>) -> Result<String, JsValue> {
        let language = lang.unwrap_or_else(|| "zh".to_string());

        if language == "zh" || language == "zh-CN" || language == "zh-TW" {
            // Use IPA mode for compatibility with the public kokoro-v1.0.onnx model
            Ok(self.chinese_g2p_ipa.process(text))
        } else {
            Ok(text.to_string())
        }
    }
    
    #[wasm_bindgen(js_name = phonemizeIPA)]
    pub fn phonemize_ipa(&self, text: &str, lang: Option<String>) -> Result<String, JsValue> {
        let language = lang.unwrap_or_else(|| "zh".to_string());

        if language == "zh" || language == "zh-CN" || language == "zh-TW" {
            Ok(self.chinese_g2p_ipa.process(text))
        } else {
            Ok(text.to_string())
        }
    }

    #[wasm_bindgen(js_name = phonemizeDisplay)]
    pub fn phonemize_display(&self, text: &str, lang: Option<String>) -> Result<String, JsValue> {
        let tokens = self.phonemize(text, lang)?;
        Ok(convert_tokens_to_bopomofo(&tokens))
    }

    #[wasm_bindgen(js_name = phonemizeBopomofo)]
    pub fn phonemize_bopomofo(&self, text: &str, lang: Option<String>) -> Result<String, JsValue> {
        let language = lang.unwrap_or_else(|| "zh".to_string());

        if language == "zh" || language == "zh-CN" || language == "zh-TW" {
            Ok(self.chinese_g2p.process(text))
        } else {
            Ok(text.to_string())
        }
    }

    #[wasm_bindgen(js_name = tokenizeV11)]
    pub fn tokenize_v11(&self, phonemes: &str) -> Result<Vec<u32>, JsValue> {
        use crate::tts::vocab::ZH_VOCAB;
        let tokens: Vec<u32> = phonemes
            .chars()
            .filter_map(|c| ZH_VOCAB.get(&c))
            .map(|&idx| idx as u32)
            .collect();
        Ok(tokens)
    }

    #[wasm_bindgen(js_name = tokenize)]
    pub fn tokenize_phonemes(&self, phonemes: &str, lang: Option<String>) -> Result<Vec<u32>, JsValue> {
        let language = lang.unwrap_or_else(|| "zh".to_string());

        let tokens: Vec<u32> = if language == "zh" || language == "zh-CN" || language == "zh-TW" {
            // Use MODEL_VOCAB for compatibility with the public kokoro-v1.0.onnx model
            phonemes
                .chars()
                .filter_map(|c| MODEL_VOCAB.get(&c))
                .map(|&idx| idx as u32)
                .collect()
        } else {
            phonemes
                .chars()
                .filter_map(|c| MODEL_VOCAB.get(&c))
                .map(|&idx| idx as u32)
                .collect()
        };

        Ok(tokens)
    }

    #[wasm_bindgen(js_name = getVoices)]
    pub fn get_voices(&self) -> Result<JsValue, JsValue> {
        let voices = vec![
            VoiceInfo {
                id: "zf_xiaoxiao".to_string(),
                name: "小小".to_string(),
                language: "zh".to_string(),
                gender: "female".to_string(),
            },
            VoiceInfo {
                id: "zm_yunxi".to_string(),
                name: "云希".to_string(),
                language: "zh".to_string(),
                gender: "male".to_string(),
            },
        ];

        serde_wasm_bindgen::to_value(&voices).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    #[cfg(feature = "oxionnx")]
    #[wasm_bindgen(js_name = synthesize)]
    pub fn synthesize(
        &self,
        text: &str,
        style: &[f32],
        speed: f32,
    ) -> Result<JsValue, JsValue> {
        let model = self.model.as_ref()
            .ok_or_else(|| JsValue::from_str("Model not loaded. Call loadModel() first."))?;

        let phonemes = self.chinese_g2p_ipa.process(text);
        let phonemes_display = convert_tokens_to_bopomofo(&phonemes);
        
        let tokens: Vec<i64> = phonemes
            .chars()
            .filter_map(|c| MODEL_VOCAB.get(&c))
            .map(|&idx| idx as i64)
            .collect();

        if tokens.is_empty() {
            return Err(JsValue::from_str("No tokens generated from text"));
        }

        const STYLE_DIM: usize = 256;
        const MAX_TOKEN_LEN: usize = 510;
        
        if tokens.len() >= MAX_TOKEN_LEN - 2 {
            return Err(JsValue::from_str(&format!("Text too long: {} tokens (max {})", tokens.len(), MAX_TOKEN_LEN - 2)));
        }
        
        let token_len = tokens.len();
        let style_vec = if style.len() >= (token_len + 1) * STYLE_DIM {
            let offset = token_len * STYLE_DIM;
            style[offset..offset + STYLE_DIM].to_vec()
        } else if style.len() >= STYLE_DIM {
            style[..STYLE_DIM].to_vec()
        } else {
            let mut s = vec![0.1f32; STYLE_DIM];
            s[..style.len().min(STYLE_DIM)].copy_from_slice(&style[..style.len().min(STYLE_DIM)]);
            s
        };

        let mut padded_tokens = vec![0i64];
        padded_tokens.extend(tokens);
        padded_tokens.push(0);

        let audio = model.run(padded_tokens, style_vec, speed)
            .map_err(|e| JsValue::from_str(&format!("Inference failed: {}", e)))?;

        let result = js_sys::Object::new();
        js_sys::Reflect::set(&result, &JsValue::from_str("phonemes"), &JsValue::from_str(&phonemes))?;
        js_sys::Reflect::set(&result, &JsValue::from_str("phonemesDisplay"), &JsValue::from_str(&phonemes_display))?;
        js_sys::Reflect::set(&result, &JsValue::from_str("text"), &JsValue::from_str(text))?;
        js_sys::Reflect::set(&result, &JsValue::from_str("sampleRate"), &JsValue::from_f64(self.sample_rate as f64))?;
        
        let audio_array = js_sys::Float32Array::new_with_length(audio.len() as u32);
        audio_array.copy_from(&audio);
        js_sys::Reflect::set(&result, &JsValue::from_str("audio"), &audio_array)?;

        Ok(result.into())
    }

    #[cfg(feature = "oxionnx")]
    #[wasm_bindgen(js_name = synthesizeWithPhonemes)]
    pub fn synthesize_with_phonemes(
        &self,
        phonemes: &str,
        style: &[f32],
        speed: f32,
    ) -> Result<JsValue, JsValue> {
        let model = self.model.as_ref()
            .ok_or_else(|| JsValue::from_str("Model not loaded. Call loadModel() first."))?;

        let tokens: Vec<i64> = phonemes
            .chars()
            .filter_map(|c| MODEL_VOCAB.get(&c))
            .map(|&idx| idx as i64)
            .collect();

        if tokens.is_empty() {
            return Err(JsValue::from_str("No tokens generated from phonemes"));
        }

        const STYLE_DIM: usize = 256;
        const MAX_TOKEN_LEN: usize = 510;
        
        if tokens.len() >= MAX_TOKEN_LEN - 2 {
            return Err(JsValue::from_str(&format!("Phonemes too long: {} tokens (max {})", tokens.len(), MAX_TOKEN_LEN - 2)));
        }
        
        let token_len = tokens.len();
        let style_vec = if style.len() >= (token_len + 1) * STYLE_DIM {
            let offset = token_len * STYLE_DIM;
            style[offset..offset + STYLE_DIM].to_vec()
        } else if style.len() >= STYLE_DIM {
            style[..STYLE_DIM].to_vec()
        } else {
            let mut s = vec![0.1f32; STYLE_DIM];
            s[..style.len().min(STYLE_DIM)].copy_from_slice(&style[..style.len().min(STYLE_DIM)]);
            s
        };

        let mut padded_tokens = vec![0i64];
        padded_tokens.extend(tokens);
        padded_tokens.push(0);

        let audio = model.run(padded_tokens, style_vec, speed)
            .map_err(|e| JsValue::from_str(&format!("Inference failed: {}", e)))?;

        let audio_array = js_sys::Float32Array::new_with_length(audio.len() as u32);
        audio_array.copy_from(&audio);

        Ok(audio_array.into())
    }

    #[wasm_bindgen(js_name = getSampleRate)]
    pub fn get_sample_rate(&self) -> u32 {
        self.sample_rate
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}
