use crate::pipeline::{ChunkTask, PipelineMessage, TextChunk, TextChunkQueue, TokenQueue, TextSplitter, TextSplitterConfig};
use crate::tts::Phonemizer;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

pub struct PreprocessorConfig {
    pub lan: String,
    pub min_phonemes: usize,
    pub max_phonemes: usize,
    pub max_chars: usize,
    pub num_workers: usize,
}

impl Default for PreprocessorConfig {
    fn default() -> Self {
        Self {
            lan: "zh".to_string(),
            min_phonemes: 3,
            max_phonemes: 510,
            max_chars: 150,
            num_workers: 2,
        }
    }
}

pub struct Preprocessor {
    config: PreprocessorConfig,
    text_queue: TextChunkQueue,
    token_queue: TokenQueue,
    total_output: Option<Arc<AtomicUsize>>,
}

impl Preprocessor {
    pub fn new(config: PreprocessorConfig, text_queue: TextChunkQueue, token_queue: TokenQueue) -> Self {
        Self {
            config,
            text_queue,
            token_queue,
            total_output: None,
        }
    }

    pub fn with_total_output(mut self, total: Arc<AtomicUsize>) -> Self {
        self.total_output = Some(total);
        self
    }

    pub fn spawn(self) -> (thread::JoinHandle<()>, PreprocessStats) {
        let stats = PreprocessStats::new();
        let stats_clone = stats.clone();
        
        let handle = thread::spawn(move || {
            let phonemizer = Phonemizer::new();
            let splitter_config = TextSplitterConfig {
                max_phonemes: self.config.max_phonemes,
                max_chars: self.config.max_chars,
                ..Default::default()
            };
            let splitter = TextSplitter::new(splitter_config);
            
            let mut total_input_chunks = 0usize;
            let mut valid_output_chunks = 0usize;
            let mut skipped_empty = 0usize;
            let mut skipped_short = 0usize;
            let mut skipped_long = 0usize;
            let mut split_count = 0usize;
            let mut next_output_index = 0usize;
            
            loop {
                match self.text_queue.recv() {
                    Ok(PipelineMessage::Data(chunk)) => {
                        total_input_chunks += 1;
                        
                        let cleaned_text = Self::clean_text(&chunk.text);
                        
                        if cleaned_text.is_empty() {
                            skipped_empty += 1;
                            continue;
                        }
                        
                        let lan = self.config.lan.clone();
                        let sub_texts = splitter.split_by_phoneme_limit(&cleaned_text, |t| {
                            phonemizer.phonemize(t, &lan).chars().count()
                        });
                        
                        if sub_texts.len() > 1 {
                            split_count += sub_texts.len() - 1;
                        }
                        
                        for sub_text in sub_texts {
                            let phonemes = phonemizer.phonemize(&sub_text, &self.config.lan);
                            let phoneme_len = phonemes.chars().count();
                            
                            if phoneme_len < self.config.min_phonemes {
                                let preview: String = sub_text.chars().take(30).collect();
                                eprintln!("Warning: Chunk has too few phonemes ({}), skipping: '{}'", 
                                    phoneme_len, &preview);
                                skipped_short += 1;
                                continue;
                            }
                            
                            if phoneme_len > self.config.max_phonemes {
                                let preview: String = sub_text.chars().take(30).collect();
                                eprintln!("Warning: Chunk still exceeds max phonemes ({} > {}), skipping: '{}...'", 
                                    phoneme_len, self.config.max_phonemes, &preview);
                                skipped_long += 1;
                                continue;
                            }
                            
                            let task = ChunkTask {
                                index: next_output_index,
                                text: Arc::new(sub_text),
                            };
                            
                            if self.token_queue.send(task).is_err() {
                                break;
                            }
                            next_output_index += 1;
                            valid_output_chunks += 1;
                        }
                    }
                    Ok(PipelineMessage::End) => {
                        if let Some(ref total) = self.total_output {
                            total.store(valid_output_chunks, Ordering::SeqCst);
                        }
                        for _ in 0..self.config.num_workers {
                            let _ = self.token_queue.send_end();
                        }
                        break;
                    }
                    Err(_) => {
                        if let Some(ref total) = self.total_output {
                            total.store(valid_output_chunks, Ordering::SeqCst);
                        }
                        for _ in 0..self.config.num_workers {
                            let _ = self.token_queue.send_end();
                        }
                        break;
                    }
                }
            }
            
            stats.set(PreprocessStatsData {
                total_input_chunks,
                valid_output_chunks,
                skipped_empty,
                skipped_short,
                skipped_long,
                split_count,
            });
        });
        
        (handle, stats_clone)
    }

    fn clean_text(text: &str) -> String {
        let cleaned: String = text
            .chars()
            .filter(|c| !c.is_control() || *c == '\n')
            .collect();
        
        let trimmed = cleaned.trim();
        
        let result: String = trimmed
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        
        result
    }
}

#[derive(Clone, Default)]
pub struct PreprocessStatsData {
    pub total_input_chunks: usize,
    pub valid_output_chunks: usize,
    pub skipped_empty: usize,
    pub skipped_short: usize,
    pub skipped_long: usize,
    pub split_count: usize,
}

#[derive(Clone)]
pub struct PreprocessStats {
    data: Arc<std::sync::Mutex<Option<PreprocessStatsData>>>,
}

impl PreprocessStats {
    pub fn new() -> Self {
        Self {
            data: Arc::new(std::sync::Mutex::new(None)),
        }
    }
    
    pub fn set(&self, data: PreprocessStatsData) {
        *self.data.lock().unwrap() = Some(data);
    }
    
    pub fn get(&self) -> Option<PreprocessStatsData> {
        self.data.lock().unwrap().clone()
    }
}

impl Default for PreprocessStats {
    fn default() -> Self {
        Self::new()
    }
}

pub fn run_preprocessor(
    lan: String, 
    text_queue: TextChunkQueue, 
    token_queue: TokenQueue
) -> (thread::JoinHandle<()>, PreprocessStats) {
    let config = PreprocessorConfig {
        lan,
        ..Default::default()
    };
    Preprocessor::new(config, text_queue, token_queue).spawn()
}

pub fn run_preprocessor_with_config(
    config: PreprocessorConfig,
    text_queue: TextChunkQueue, 
    token_queue: TokenQueue
) -> (thread::JoinHandle<()>, PreprocessStats) {
    Preprocessor::new(config, text_queue, token_queue).spawn()
}

pub fn run_preprocessor_with_total(
    config: PreprocessorConfig,
    text_queue: TextChunkQueue, 
    token_queue: TokenQueue,
    total_output: Arc<AtomicUsize>
) -> (thread::JoinHandle<()>, PreprocessStats) {
    Preprocessor::new(config, text_queue, token_queue)
        .with_total_output(total_output)
        .spawn()
}
