mod audio;
mod config;

use std::sync::Arc;

use axum::{
    extract::State,
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse,
    },
    routing::{get, post},
    Json, Router,
};
use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tower_http::cors::{Any, CorsLayer};

use config::Config;
use kokoros::pipeline::{
    spawn_generator_workers, GeneratorConfig, PreprocessorConfig,
    run_preprocessor_with_config, SortedAudioQueue, TextChunk, TextChunkQueue, TokenQueue,
    TTSBackend,
};
use kokoros::tts::{koko::TTSKokoParallel, Phonemizer};

#[derive(Clone)]
struct AppState {
    tts: Arc<TTSKokoParallel>,
    config: Arc<Config>,
}

#[derive(Debug, Deserialize)]
struct TTSRequest {
    text: String,
    #[serde(default = "default_voice")]
    voice: String,
    #[serde(default = "default_speed")]
    speed: f32,
}

fn default_voice() -> String { "zf_xiaobei".to_string() }
fn default_speed() -> f32 { 1.0 }

#[derive(Debug, Serialize)]
struct TTSResponse {
    success: bool,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    audio_base64: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    duration: Option<f32>,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
enum SSEData {
    Progress { chunk_index: usize, total_chunks: usize, message: String },
    AudioChunk { chunk_index: usize, audio_base64: String },
    Complete { total_duration: f32 },
    Error { message: String },
}

#[derive(Clone)]
struct TTSBackendImpl {
    tts: Arc<TTSKokoParallel>,
}

impl TTSBackend for TTSBackendImpl {
    fn generate(&self, text: &str, style: &str, lan: &str, speed: f32, instance_id: usize) -> Option<Vec<f32>> {
        let model_instance = self.tts.get_model_instance(instance_id);
        self.tts.tts_raw_audio_with_instance(
            text, lan, style, speed, None, None, None, Some(instance_id), model_instance,
        ).ok()
    }
}

#[tokio::main]
async fn main() {
    let config = Arc::new(Config::load());
    
    let host = config.host.clone();
    let port = config.port;
    let num_threads = config.threads;
    
    let tts = Arc::new(TTSKokoParallel::new_with_instances(
        config.model_path.to_str().unwrap(),
        config.voices_path.to_str().unwrap(),
        num_threads,
    ).await);
    
    let state = AppState {
        tts: Arc::clone(&tts),
        config,
    };
    
    let app = Router::new()
        .route("/", get(serve_index))
        .route("/tts", post(tts_handler))
        .route("/tts/stream", post(tts_stream_handler))
        .route("/voices", get(list_voices))
        .route("/health", get(health))
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any))
        .with_state(state);
    
    let addr = format!("{}:{}", host, port);
    
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn serve_index() -> impl IntoResponse {
    let html = include_str!("../static/index.html");
    ([("Content-Type", "text/html; charset=utf-8")], html)
}

async fn health() -> impl IntoResponse {
    Json(serde_json::json!({ "status": "ok" }))
}

async fn list_voices() -> impl IntoResponse {
    let voices = vec![
        "af_alloy", "af_aoede", "af_bella", "af_heart", "af_jessica", "af_kore", "af_nicole", "af_nova", "af_river", "af_sarah", "af_sky",
        "am_adam", "am_echo", "am_eric", "am_fenrir", "am_liam", "am_michael", "am_onyx", "am_puck", "am_santa",
        "bf_alice", "bf_emma", "bf_isabella", "bf_lily",
        "bm_daniel", "bm_fable", "bm_george", "bm_lewis",
        "ef_dora",
        "em_alex", "em_santa",
        "ff_siwis",
        "hf_alpha", "hf_beta",
        "hm_omega", "hm_psi",
        "if_sara",
        "im_nicola",
        "jf_alpha", "jf_gongitsune", "jf_nezumi", "jf_tebukuro",
        "jm_kumo",
        "pf_dora",
        "pm_alex", "pm_santa",
        "zf_xiaobei", "zf_xiaoni", "zf_xiaoxiao", "zf_xiaoyi",
        "zm_yunjian", "zm_yunxi", "zm_yunxia", "zm_yunyang",
    ];
    Json(voices)
}

async fn tts_handler(
    State(state): State<AppState>,
    Json(req): Json<TTSRequest>,
) -> impl IntoResponse {
    if req.text.trim().is_empty() {
        return Json(TTSResponse {
            success: false,
            message: "Text is empty".to_string(),
            audio_base64: None,
            duration: None,
        });
    }
    
    let text_queue = TextChunkQueue::new(100);
    let token_queue = TokenQueue::new(100);
    let audio_queue = Arc::new(SortedAudioQueue::new());
    
    let preprocessor_config = PreprocessorConfig {
        lan: "zh".to_string(),
        min_phonemes: 3,
        max_phonemes: 510,
        max_chars: state.config.max_chars,
        num_workers: state.config.threads,
    };
    
    let (preprocessor_handle, _stats) = run_preprocessor_with_config(
        preprocessor_config,
        text_queue.clone(),
        token_queue.clone(),
    );
    
    let backend = TTSBackendImpl {
        tts: Arc::clone(&state.tts),
    };
    
    let generator_config = GeneratorConfig {
        num_workers: state.config.threads,
        style: req.voice.clone(),
        lan: "zh".to_string(),
        speed: req.speed,
    };
    
    let generator_handles = spawn_generator_workers(
        Arc::new(backend),
        generator_config,
        token_queue.clone(),
        Arc::clone(&audio_queue),
    );
    
    let phonemizer = Phonemizer::new();
    let splitter = kokoros::pipeline::TextSplitter::default();
    let sub_texts = splitter.split_by_phoneme_limit(&req.text, |t| phonemizer.phonemize(t, "zh").chars().count());
    
    let total_chunks = sub_texts.len();
    for (idx, sub_text) in sub_texts.iter().enumerate() {
        let text = Arc::new(sub_text.clone());
        let _ = text_queue.send(TextChunk { index: idx, text });
    }
    let _ = text_queue.send_end();
    
    let mut all_samples = Vec::new();
    let mut idx = 0;
    while idx < total_chunks {
        if let Some(samples) = audio_queue.pop_next(5000) {
            all_samples.extend(samples);
            idx += 1;
        } else {
            break;
        }
    }
    
    audio_queue.mark_ended();
    
    let _ = preprocessor_handle.join();
    for handle in generator_handles {
        let _ = handle.join();
    }
    
    let duration = all_samples.len() as f32 / audio::SAMPLE_RATE as f32;
    
    match audio::float_samples_to_wav(&all_samples) {
        Ok(wav_data) => {
            let base64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &wav_data);
            Json(TTSResponse {
                success: true,
                message: "Success".to_string(),
                audio_base64: Some(base64),
                duration: Some(duration),
            })
        }
        Err(e) => Json(TTSResponse {
            success: false,
            message: format!("Failed to encode audio: {}", e),
            audio_base64: None,
            duration: None,
        }),
    }
}

async fn tts_stream_handler(
    State(state): State<AppState>,
    Json(req): Json<TTSRequest>,
) -> Sse<impl Stream<Item = Result<Event, axum::Error>>> {
    let (tx, rx) = mpsc::channel(100);
    
    let text_queue = TextChunkQueue::new(100);
    let token_queue = TokenQueue::new(100);
    let audio_queue = Arc::new(SortedAudioQueue::new());
    
    let preprocessor_config = PreprocessorConfig {
        lan: "zh".to_string(),
        min_phonemes: 3,
        max_phonemes: 510,
        max_chars: state.config.max_chars,
        num_workers: state.config.threads,
    };
    
    let (preprocessor_handle, _stats) = run_preprocessor_with_config(
        preprocessor_config,
        text_queue.clone(),
        token_queue.clone(),
    );
    
    let backend = TTSBackendImpl {
        tts: Arc::clone(&state.tts),
    };
    
    let generator_config = GeneratorConfig {
        num_workers: state.config.threads,
        style: req.voice.clone(),
        lan: "zh".to_string(),
        speed: req.speed,
    };
    
    let generator_handles = spawn_generator_workers(
        Arc::new(backend),
        generator_config,
        token_queue.clone(),
        Arc::clone(&audio_queue),
    );
    
    let phonemizer = Phonemizer::new();
    let splitter = kokoros::pipeline::TextSplitter::default();
    let sub_texts = splitter.split_by_phoneme_limit(&req.text, |t| phonemizer.phonemize(t, "zh").chars().count());
    
    let total_chunks = sub_texts.len();
    for (idx, sub_text) in sub_texts.iter().enumerate() {
        let text = Arc::new(sub_text.clone());
        let _ = text_queue.send(TextChunk { index: idx, text });
    }
    let _ = text_queue.send_end();
    
    let tx_clone = tx.clone();
    let audio_queue_clone = Arc::clone(&audio_queue);
    
    std::thread::spawn(move || {
        let mut idx = 0;
        while idx < total_chunks {
            if let Some(samples) = audio_queue_clone.pop_next(5000) {
                let pcm = audio::float_samples_to_pcm(&samples);
                let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &pcm);
                let msg = SSEData::AudioChunk { chunk_index: idx, audio_base64: b64 };
                if tx_clone.blocking_send(Event::default().json_data(msg)).is_err() {
                    break;
                }
                idx += 1;
            } else {
                break;
            }
        }
        
        let total_samples: usize = idx * 48000;
        let total_duration = total_samples as f32 / audio::SAMPLE_RATE as f32;
        let complete_msg = SSEData::Complete { total_duration };
        let _ = tx_clone.blocking_send(Event::default().json_data(complete_msg));
        
        audio_queue_clone.mark_ended();
        
        let _ = preprocessor_handle.join();
        for handle in generator_handles {
            let _ = handle.join();
        }
    });
    
    Sse::new(ReceiverStream::new(rx)).keep_alive(KeepAlive::default())
}
