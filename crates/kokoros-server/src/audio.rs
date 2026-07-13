pub const SAMPLE_RATE: u32 = 24000;

pub fn normalize_audio(samples: &mut [f32]) {
    let mut max_abs = 0.0f32;
    for &s in samples.iter() {
        let abs = s.abs();
        if abs > max_abs {
            max_abs = abs;
        }
    }
    
    if max_abs > 1.0 {
        let scale = 0.95 / max_abs;
        for s in samples.iter_mut() {
            *s *= scale;
        }
    }
}

pub fn apply_fade(samples: &mut [f32], fade_len: usize, fade_in: bool, fade_out: bool) {
    let fade_len = fade_len.min(samples.len() / 2);
    if fade_len == 0 {
        return;
    }
    if fade_in {
        for i in 0..fade_len {
            let t = i as f32 / fade_len as f32;
            samples[i] *= t;
        }
    }
    if fade_out {
        for i in 0..fade_len {
            let t = i as f32 / fade_len as f32;
            samples[samples.len() - 1 - i] *= t;
        }
    }
}

pub fn float_samples_to_wav(samples: &[f32]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut normalized = samples.to_vec();
    
    for s in normalized.iter_mut() {
        if s.is_nan() || s.is_infinite() {
            *s = 0.0;
        }
    }
    
    normalize_audio(&mut normalized);
    
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: SAMPLE_RATE,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    
    let mut cursor = std::io::Cursor::new(Vec::new());
    {
        let mut writer = hound::WavWriter::new(&mut cursor, spec)?;
        for &sample in &normalized {
            let clamped = sample.clamp(-1.0, 1.0);
            let sample_i16 = if clamped < 0.0 {
                (clamped * 32768.0) as i16
            } else {
                (clamped * 32767.0) as i16
            };
            writer.write_sample(sample_i16)?;
        }
        writer.finalize()?;
    }
    
    Ok(cursor.into_inner())
}

pub fn float_samples_to_pcm(samples: &[f32]) -> Vec<u8> {
    let mut normalized = samples.to_vec();
    
    for s in normalized.iter_mut() {
        if s.is_nan() || s.is_infinite() {
            *s = 0.0;
        }
    }
    
    normalize_audio(&mut normalized);
    
    let mut pcm = Vec::with_capacity(normalized.len() * 2);
    for &sample in &normalized {
        let clamped = sample.clamp(-1.0, 1.0);
        let sample_i16 = if clamped < 0.0 {
            (clamped * 32768.0) as i16
        } else {
            (clamped * 32767.0) as i16
        };
        pcm.extend_from_slice(&sample_i16.to_le_bytes());
    }
    pcm
}

pub fn merge_audio_chunks(chunks: Vec<Vec<f32>>) -> Vec<f32> {
    let total_len: usize = chunks.iter().map(|c| c.len()).sum();
    let mut merged = Vec::with_capacity(total_len);
    for chunk in chunks {
        merged.extend(chunk);
    }
    merged
}
