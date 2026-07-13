use kokoros::pipeline::{TextChunk, TextChunkQueue};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::Arc;

pub struct StreamingFileReader;

impl StreamingFileReader {
    pub fn new(_max_chars: usize) -> Self {
        Self
    }

    pub fn read_file(&self, file_path: &str, queue: TextChunkQueue) -> std::io::Result<usize> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut chunk_index = 0;

        for line in reader.lines() {
            let line = line?;
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let text = Arc::new(line.to_string());
            let _ = queue.send(TextChunk { index: chunk_index, text });
            chunk_index += 1;
        }

        let _ = queue.send_end();
        Ok(chunk_index)
    }
}
