use std::sync::Arc;

pub struct TextChunk {
    pub index: usize,
    pub text: Arc<String>,
}

pub struct ChunkTask {
    pub index: usize,
    pub text: Arc<String>,
}

pub struct AudioChunk {
    pub index: usize,
    pub samples: Vec<f32>,
}

pub enum PipelineMessage<T> {
    Data(T),
    End,
}
