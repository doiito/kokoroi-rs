mod generator;
mod preprocessor;
mod queue;
mod text_splitter;
#[cfg(feature = "ort")]
mod tts_backend;
mod types;

pub use generator::{Generator, GeneratorConfig, TTSBackend, spawn_generator_workers};
pub use preprocessor::{Preprocessor, PreprocessorConfig, PreprocessStats, PreprocessStatsData, run_preprocessor, run_preprocessor_with_config, run_preprocessor_with_total};
pub use queue::{SortedAudioQueue, TextChunkQueue, TokenQueue};
pub use text_splitter::{TextSplitter, TextSplitterConfig, split_text, split_text_with_phoneme_limit};
pub use types::{AudioChunk, ChunkTask, PipelineMessage, TextChunk};

pub use crossbeam::channel;
