use crate::pipeline::types::{ChunkTask, TextChunk, PipelineMessage};
use crossbeam::channel::{bounded, Receiver, Sender};
use std::collections::BTreeMap;
use std::sync::{Condvar, Mutex};

pub struct TextChunkQueue {
    sender: Sender<PipelineMessage<TextChunk>>,
    receiver: Receiver<PipelineMessage<TextChunk>>,
}

impl Clone for TextChunkQueue {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            receiver: self.receiver.clone(),
        }
    }
}

impl TextChunkQueue {
    pub fn new(capacity: usize) -> Self {
        let (sender, receiver) = bounded(capacity);
        Self { sender, receiver }
    }

    pub fn send(&self, chunk: TextChunk) -> Result<(), crossbeam::channel::SendError<PipelineMessage<TextChunk>>> {
        self.sender.send(PipelineMessage::Data(chunk))
    }

    pub fn send_end(&self) -> Result<(), crossbeam::channel::SendError<PipelineMessage<TextChunk>>> {
        self.sender.send(PipelineMessage::End)
    }

    pub fn recv(&self) -> Result<PipelineMessage<TextChunk>, crossbeam::channel::RecvError> {
        self.receiver.recv()
    }

    pub fn sender(&self) -> Sender<PipelineMessage<TextChunk>> {
        self.sender.clone()
    }
}

pub struct TokenQueue {
    sender: Sender<PipelineMessage<ChunkTask>>,
    receiver: Receiver<PipelineMessage<ChunkTask>>,
}

impl Clone for TokenQueue {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            receiver: self.receiver.clone(),
        }
    }
}

impl TokenQueue {
    pub fn new(capacity: usize) -> Self {
        let (sender, receiver) = bounded(capacity);
        Self { sender, receiver }
    }

    pub fn send(&self, task: ChunkTask) -> Result<(), crossbeam::channel::SendError<PipelineMessage<ChunkTask>>> {
        self.sender.send(PipelineMessage::Data(task))
    }

    pub fn send_end(&self) -> Result<(), crossbeam::channel::SendError<PipelineMessage<ChunkTask>>> {
        self.sender.send(PipelineMessage::End)
    }

    pub fn recv(&self) -> Result<PipelineMessage<ChunkTask>, crossbeam::channel::RecvError> {
        self.receiver.recv()
    }

    pub fn receiver(&self) -> Receiver<PipelineMessage<ChunkTask>> {
        self.receiver.clone()
    }
}

pub struct SortedAudioQueue {
    data: Mutex<BTreeMap<usize, Vec<f32>>>,
    condvar: Condvar,
    next_index: Mutex<usize>,
    ended: Mutex<bool>,
}

impl SortedAudioQueue {
    pub fn new() -> Self {
        Self {
            data: Mutex::new(BTreeMap::new()),
            condvar: Condvar::new(),
            next_index: Mutex::new(0),
            ended: Mutex::new(false),
        }
    }

    pub fn push(&self, index: usize, samples: Vec<f32>) {
        let mut data = self.data.lock().unwrap();
        data.insert(index, samples);
        self.condvar.notify_all();
    }

    pub fn pop_next(&self, timeout_ms: u64) -> Option<Vec<f32>> {
        let mut data = self.data.lock().unwrap();
        let start = std::time::Instant::now();
        
        loop {
            let next_idx = *self.next_index.lock().unwrap();
            
            if let Some(samples) = data.remove(&next_idx) {
                *self.next_index.lock().unwrap() += 1;
                return Some(samples);
            }
            
            if *self.ended.lock().unwrap() && data.is_empty() {
                return None;
            }
            
            let elapsed = start.elapsed().as_millis() as u64;
            if elapsed >= timeout_ms {
                return None;
            }
            
            let remaining = timeout_ms.saturating_sub(elapsed);
            let result = self.condvar.wait_timeout(data, std::time::Duration::from_millis(remaining.min(100))).unwrap();
            data = result.0;
        }
    }

    pub fn mark_ended(&self) {
        *self.ended.lock().unwrap() = true;
        self.condvar.notify_all();
    }

    pub fn mark_failed(&self, index: usize) {
        let mut data = self.data.lock().unwrap();
        data.insert(index, Vec::new());
        self.condvar.notify_all();
    }

    pub fn is_complete(&self, total_chunks: usize) -> bool {
        let next_idx = *self.next_index.lock().unwrap();
        next_idx >= total_chunks
    }

    pub fn is_empty(&self) -> bool {
        self.data.lock().unwrap().is_empty()
    }

    pub fn has_chunk(&self, index: usize) -> bool {
        self.data.lock().unwrap().contains_key(&index)
    }
}

impl Default for SortedAudioQueue {
    fn default() -> Self {
        Self::new()
    }
}
