use wasm_bindgen::prelude::*;

pub fn pcm_to_wav(pcm: &[f32], sample_rate: u32) -> Vec<u8> {
    let mut wav = Vec::new();
    
    wav.extend_from_slice(b"RIFF");
    let data_size = (pcm.len() * 2) as u32;
    let file_size = data_size + 36;
    wav.extend_from_slice(&file_size.to_le_bytes());
    wav.extend_from_slice(b"WAVE");
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&sample_rate.to_le_bytes());
    let byte_rate = sample_rate * 2;
    wav.extend_from_slice(&byte_rate.to_le_bytes());
    wav.extend_from_slice(&2u16.to_le_bytes());
    wav.extend_from_slice(&16u16.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_size.to_le_bytes());
    
    for sample in pcm {
        let sample_i16 = (sample * 32767.0).min(32767.0).max(-32768.0) as i16;
        wav.extend_from_slice(&sample_i16.to_le_bytes());
    }
    
    wav
}

#[wasm_bindgen]
pub fn create_wav_header(sample_rate: u32, num_samples: usize) -> Vec<u8> {
    let mut header = Vec::new();
    
    header.extend_from_slice(b"RIFF");
    let data_size = (num_samples * 2) as u32;
    let file_size = data_size + 36;
    header.extend_from_slice(&file_size.to_le_bytes());
    header.extend_from_slice(b"WAVE");
    header.extend_from_slice(b"fmt ");
    header.extend_from_slice(&16u32.to_le_bytes());
    header.extend_from_slice(&1u16.to_le_bytes());
    header.extend_from_slice(&1u16.to_le_bytes());
    header.extend_from_slice(&sample_rate.to_le_bytes());
    let byte_rate = sample_rate * 2;
    header.extend_from_slice(&byte_rate.to_le_bytes());
    header.extend_from_slice(&2u16.to_le_bytes());
    header.extend_from_slice(&16u16.to_le_bytes());
    header.extend_from_slice(b"data");
    header.extend_from_slice(&data_size.to_le_bytes());
    
    header
}

#[wasm_bindgen]
pub fn pcm_samples_to_wav_data(samples: &[f32]) -> Vec<u8> {
    const SAMPLE_RATE: u32 = 24000;
    let mut wav = Vec::with_capacity(44 + samples.len() * 2);
    
    wav.extend_from_slice(b"RIFF");
    let data_size = (samples.len() * 2) as u32;
    let file_size = data_size + 36;
    wav.extend_from_slice(&file_size.to_le_bytes());
    wav.extend_from_slice(b"WAVE");
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
    let byte_rate = SAMPLE_RATE * 2;
    wav.extend_from_slice(&byte_rate.to_le_bytes());
    wav.extend_from_slice(&2u16.to_le_bytes());
    wav.extend_from_slice(&16u16.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_size.to_le_bytes());
    
    for sample in samples {
        let sample_i16 = (sample * 32767.0).min(32767.0).max(-32768.0) as i16;
        wav.extend_from_slice(&sample_i16.to_le_bytes());
    }
    
    wav
}
