use crate::pipeline::{ChunkTask, PipelineMessage, SortedAudioQueue, TokenQueue};
use std::sync::Arc;

pub struct GeneratorConfig {
    pub num_workers: usize,
    pub style: String,
    pub lan: String,
    pub speed: f32,
}

pub struct Generator<TTS: Clone + Send + 'static> {
    tts: Arc<TTS>,
    config: GeneratorConfig,
    token_queue: TokenQueue,
    audio_queue: Arc<SortedAudioQueue>,
}

impl<TTS: Clone + Send + 'static> Generator<TTS> {
    pub fn new(
        tts: Arc<TTS>,
        config: GeneratorConfig,
        token_queue: TokenQueue,
        audio_queue: Arc<SortedAudioQueue>,
    ) -> Self {
        Self {
            tts,
            config,
            token_queue,
            audio_queue,
        }
    }
}

pub trait TTSBackend: Clone + Send + Sync + 'static {
    fn generate(&self, text: &str, style: &str, lan: &str, speed: f32, instance_id: usize) -> Option<Vec<f32>>;
}

pub fn spawn_generator_workers<TTS: TTSBackend>(
    tts: Arc<TTS>,
    config: GeneratorConfig,
    token_queue: TokenQueue,
    audio_queue: Arc<SortedAudioQueue>,
) -> Vec<std::thread::JoinHandle<()>> {
    let mut handles = Vec::new();
    
    for worker_id in 0..config.num_workers {
        let tts = Arc::clone(&tts);
        let style = config.style.clone();
        let lan = config.lan.clone();
        let speed = config.speed;
        let token_queue = token_queue.clone();
        let audio_queue = Arc::clone(&audio_queue);
        
        let handle = std::thread::spawn(move || {
            loop {
                match token_queue.recv() {
                    Ok(PipelineMessage::Data(task)) => {
                        let preview: String = task.text.chars().take(30).collect();
                        eprintln!("Worker {} processing chunk {}: '{}'", 
                            worker_id, task.index + 1, &preview);
                        
                        match tts.generate(&task.text, &style, &lan, speed, worker_id) {
                            Some(samples) => {
                                audio_queue.push(task.index, samples);
                            }
                            None => {
                                eprintln!("Worker {} failed to generate chunk {}", worker_id, task.index + 1);
                                audio_queue.mark_failed(task.index);
                            }
                        }
                    }
                    Ok(PipelineMessage::End) | Err(_) => {
                        break;
                    }
                }
            }
        });
        
        handles.push(handle);
    }
    
    handles
}
