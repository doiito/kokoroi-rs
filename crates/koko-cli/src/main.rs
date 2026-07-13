mod streaming_reader;

use clap::Parser;
use kokoros::pipeline::{
    spawn_generator_workers, GeneratorConfig, PreprocessorConfig,
    run_preprocessor_with_total, SortedAudioQueue, TextChunk, TextChunkQueue, TokenQueue,
    TTSBackend,
};
use kokoros::tts::koko::TTSKokoParallel;
use streaming_reader::StreamingFileReader;
use std::{
    fs,
    io::Write,
    process::{Command, Stdio},
    sync::Arc,
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
    thread,
};
use tracing_subscriber::fmt::time::FormatTime;

struct UnixTimestampFormatter;

impl FormatTime for UnixTimestampFormatter {
    fn format_time(&self, w: &mut tracing_subscriber::fmt::format::Writer<'_>) -> std::fmt::Result {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap();
        let timestamp = format!("{}.{:06}", now.as_secs(), now.subsec_micros());
        write!(w, "{}", timestamp)
    }
}

#[derive(Parser, Debug)]
#[command(name = "koko")]
#[command(version = "0.1")]
#[command(about = "Kokoro TTS CLI - Text to Speech with pipeline architecture")]
struct Cli {
    #[arg(short = 'i', long = "input")]
    input_file: Option<String>,

    #[arg(short = 't', long = "text")]
    text: Option<String>,

    #[arg(short = 'o', long = "output", default_value = "output.wav")]
    output: String,

    #[arg(short = 'l', long = "lan", default_value = "zh")]
    lan: String,

    #[arg(short = 'm', long = "model", default_value = "models/tiandy_tts_v1.0-zh_s.onnx")]
    model_path: String,

    #[arg(short = 'd', long = "data", default_value = "data/voices-v1.0.bin")]
    data_path: String,

    #[arg(short = 's', long = "style", default_value = "zm_yunyang")]
    style: String,

    #[arg(short = 'p', long = "speed", default_value_t = 0.7, help = "Speech speed multiplier")]
    speed: f32,

    #[arg(short = 'n', long = "threads", default_value_t = 2, help = "Number of model inference threads")]
    threads: usize,

    #[arg(long = "max-chars", default_value_t = 150, help = "Max characters per chunk (affects generation speed)")]
    max_chars: usize,

    #[arg(long = "max-phonemes", default_value_t = 510, help = "Max phonemes per chunk")]
    max_phonemes: usize,

    #[arg(short = 'P', long = "play")]
    play: bool,

    #[arg(long = "buffer-chunks", default_value_t = 2)]
    buffer_chunks: usize,
}

fn write_wav_file(path: &str, samples: &[f32], sample_rate: u32) -> std::io::Result<()> {
    use std::fs::File;
    let channels: u16 = 1;
    let bits_per_sample: u16 = 32;
    let bytes_per_sample: u32 = (bits_per_sample as u32) / 8;
    let block_align: u16 = channels * bits_per_sample / 8;
    let byte_rate: u32 = sample_rate * (block_align as u32);
    let num_frames: usize = samples.len();
    let data_size: u32 = (num_frames as u32) * bytes_per_sample;
    let riff_chunk_size: u32 = 36 + data_size;
    let mut f = File::create(path)?;
    f.write_all(b"RIFF")?;
    f.write_all(&riff_chunk_size.to_le_bytes())?;
    f.write_all(b"WAVE")?;
    f.write_all(b"fmt ")?;
    f.write_all(&(16u32).to_le_bytes())?;
    f.write_all(&(3u16).to_le_bytes())?;
    f.write_all(&channels.to_le_bytes())?;
    f.write_all(&sample_rate.to_le_bytes())?;
    f.write_all(&byte_rate.to_le_bytes())?;
    f.write_all(&block_align.to_le_bytes())?;
    f.write_all(&bits_per_sample.to_le_bytes())?;
    f.write_all(b"data")?;
    f.write_all(&data_size.to_le_bytes())?;
    for &s in samples {
        f.write_all(&s.to_le_bytes())?;
    }
    Ok(())
}

fn apply_fade(samples: &mut [f32], fade_len: usize, fade_in: bool, fade_out: bool) {
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

#[cfg(target_os = "windows")]
fn spawn_playback_thread(
    audio_queue: Arc<SortedAudioQueue>,
    total_chunks: Arc<AtomicUsize>,
    generation_ended: Arc<AtomicBool>,
    min_ready: usize,
) -> thread::JoinHandle<Vec<f32>> {
    use rodio::{OutputStream, Sink, Source};
    
    thread::spawn(move || {
        let fade_samples = 720;
        
        let mut ready_count = 0;
        while ready_count < min_ready {
            let total = total_chunks.load(Ordering::SeqCst);
            if total > 0 && ready_count < total && audio_queue.has_chunk(ready_count) {
                ready_count += 1;
            } else if generation_ended.load(Ordering::SeqCst) {
                break;
            } else {
                thread::sleep(std::time::Duration::from_millis(50));
            }
        }
        
        eprintln!("Starting playback...");
        
        let (_stream, stream_handle) = match OutputStream::try_default() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Could not get audio output: {}. Saving to file only.", e);
                return collect_all_audio(audio_queue, total_chunks, generation_ended);
            }
        };
        
        let sink = match Sink::try_new(&stream_handle) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Could not create audio sink: {}", e);
                return collect_all_audio(audio_queue, total_chunks, generation_ended);
            }
        };

        let mut played = 0;
        let mut all_samples = Vec::new();
        
        loop {
            let total = total_chunks.load(Ordering::SeqCst);
            let ended = generation_ended.load(Ordering::SeqCst);
            
            if total > 0 && played >= total && ended {
                break;
            }
            
            if let Some(mut samples) = audio_queue.pop_next(100) {
                let is_first = played == 0;
                apply_fade(&mut samples, fade_samples, is_first, false);

                let i16_samples: Vec<i16> = samples
                    .iter()
                    .map(|&f| (f.clamp(-1.0, 1.0) * 32767.0) as i16)
                    .collect();

                all_samples.extend(samples.iter().copied());

                let source = rodio::buffer::SamplesBuffer::new(1, 24000, i16_samples);
                sink.append(source);
                sink.sleep_until_end();

                played += 1;
            } else if ended {
                let total = total_chunks.load(Ordering::SeqCst);
                if total > 0 && played >= total {
                    break;
                }
                if total == 0 {
                    break;
                }
                thread::sleep(std::time::Duration::from_millis(10));
            } else {
                thread::sleep(std::time::Duration::from_millis(10));
            }
        }

        sink.sleep_until_end();
        eprintln!("Playback complete. Played {} chunks.", played);
        all_samples
    })
}

#[cfg(not(target_os = "windows"))]
fn spawn_playback_thread(
    audio_queue: Arc<SortedAudioQueue>,
    total_chunks: Arc<AtomicUsize>,
    generation_ended: Arc<AtomicBool>,
    min_ready: usize,
) -> thread::JoinHandle<Vec<f32>> {
    thread::spawn(move || {
        let fade_samples = 720;
        
        let mut ready_count = 0;
        while ready_count < min_ready {
            let total = total_chunks.load(Ordering::SeqCst);
            if total > 0 && ready_count < total && audio_queue.has_chunk(ready_count) {
                ready_count += 1;
            } else if generation_ended.load(Ordering::SeqCst) {
                break;
            } else {
                thread::sleep(std::time::Duration::from_millis(50));
            }
        }
        
        eprintln!("Starting playback...");
        
        let mut aplay = Command::new("aplay")
            .arg("-r").arg("24000")
            .arg("-f").arg("S16_LE")
            .arg("-c").arg("1")
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();

        let mut ffplay = if aplay.is_err() {
            Command::new("ffplay")
                .arg("-f").arg("s16le")
                .arg("-ar").arg("24000")
                .arg("-ac").arg("1")
                .arg("-nodisp")
                .arg("-autoexit")
                .arg("-i").arg("pipe:0")
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn().ok()
        } else {
            None
        };

        let mut stdin: Option<std::process::ChildStdin> = if let Ok(ref mut child) = aplay {
            child.stdin.take()
        } else if let Some(ref mut child) = ffplay {
            child.stdin.take()
        } else {
            eprintln!("Could not find audio player. Saving to file only.");
            return collect_all_audio(audio_queue, total_chunks, generation_ended);
        };

        let mut played = 0;
        let mut all_samples = Vec::new();
        
        loop {
            let total = total_chunks.load(Ordering::SeqCst);
            let ended = generation_ended.load(Ordering::SeqCst);
            
            if total > 0 && played >= total && ended {
                break;
            }
            
            if let Some(mut samples) = audio_queue.pop_next(100) {
                let is_first = played == 0;
                apply_fade(&mut samples, fade_samples, is_first, false);
                
                if let Some(ref mut s) = stdin {
                    for f in &samples {
                        let s16 = (f.clamp(-1.0, 1.0) * 32767.0) as i16;
                        let _ = s.write_all(&s16.to_le_bytes());
                    }
                }
                
                all_samples.extend(samples);
                played += 1;
            } else if ended {
                let total = total_chunks.load(Ordering::SeqCst);
                if total > 0 && played >= total {
                    break;
                }
                if total == 0 {
                    break;
                }
                thread::sleep(std::time::Duration::from_millis(10));
            } else {
                thread::sleep(std::time::Duration::from_millis(10));
            }
        }

        drop(stdin);
        if let Ok(ref mut child) = aplay {
            let _ = child.wait();
        } else if let Some(ref mut child) = ffplay {
            let _ = child.wait();
        }
        
        eprintln!("Playback complete. Played {} chunks.", played);
        all_samples
    })
}

fn collect_all_audio(
    audio_queue: Arc<SortedAudioQueue>,
    total_chunks: Arc<AtomicUsize>,
    generation_ended: Arc<AtomicBool>,
) -> Vec<f32> {
    let mut all_samples = Vec::new();
    let mut collected = 0;
    
    loop {
        let total = total_chunks.load(Ordering::SeqCst);
        let ended = generation_ended.load(Ordering::SeqCst);
        
        if total > 0 && collected >= total && ended {
            break;
        }
        
        if let Some(samples) = audio_queue.pop_next(100) {
            all_samples.extend(samples);
            collected += 1;
        } else if ended {
            let total = total_chunks.load(Ordering::SeqCst);
            if total > 0 && collected >= total {
                break;
            }
            if total == 0 {
                break;
            }
            thread::sleep(std::time::Duration::from_millis(10));
        } else {
            thread::sleep(std::time::Duration::from_millis(10));
        }
    }
    
    all_samples
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
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_timer(UnixTimestampFormatter)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();

    let text = if let Some(input_file) = &cli.input_file {
        fs::read_to_string(input_file)?
    } else if let Some(text) = &cli.text {
        text.clone()
    } else {
        eprintln!("Error: Either --input or --text must be provided");
        std::process::exit(1);
    };

    if text.trim().is_empty() {
        eprintln!("Error: Input text is empty");
        std::process::exit(1);
    }

    eprintln!("Loading model from {}...", cli.model_path);
    let tts = Arc::new(TTSKokoParallel::new_with_instances(&cli.model_path, &cli.data_path, cli.threads).await);
    eprintln!("Model loaded. Using {} inference threads.", cli.threads);

    let start_time = std::time::Instant::now();

    let text_queue = TextChunkQueue::new(100);
    let token_queue = TokenQueue::new(100);
    let audio_queue = Arc::new(SortedAudioQueue::new());
    let total_chunks = Arc::new(AtomicUsize::new(0));
    let generation_ended = Arc::new(AtomicBool::new(false));

    let preprocessor_config = PreprocessorConfig {
        lan: cli.lan.clone(),
        min_phonemes: 3,
        max_phonemes: cli.max_phonemes,
        max_chars: cli.max_chars,
        num_workers: cli.threads,
    };
    
    let (preprocessor_handle, _stats) = run_preprocessor_with_total(
        preprocessor_config,
        text_queue.clone(),
        token_queue.clone(),
        Arc::clone(&total_chunks),
    );

    let backend = TTSBackendImpl {
        tts: Arc::clone(&tts),
    };
    
    let generator_config = GeneratorConfig {
        num_workers: cli.threads,
        style: cli.style.clone(),
        lan: cli.lan.clone(),
        speed: cli.speed,
    };
    
    let generator_handles = spawn_generator_workers(
        Arc::new(backend),
        generator_config,
        token_queue.clone(),
        Arc::clone(&audio_queue),
    );

    if let Some(input_file) = &cli.input_file {
        let reader = StreamingFileReader::new(0);
        let _ = reader.read_file(input_file, text_queue.clone())?;
    } else {
        let text = Arc::new(text);
        let _ = text_queue.send(TextChunk { index: 0, text });
        let _ = text_queue.send_end();
    }

    let generation_end_signal = Arc::clone(&generation_ended);
    let audio_queue_for_signal = Arc::clone(&audio_queue);
    thread::spawn(move || {
        let _ = preprocessor_handle.join();
        for handle in generator_handles {
            let _ = handle.join();
        }
        audio_queue_for_signal.mark_ended();
        generation_end_signal.store(true, Ordering::SeqCst);
        eprintln!("Generation complete.");
    });

    let min_ready = cli.buffer_chunks;
    
    let playback_handle = if cli.play {
        spawn_playback_thread(
            Arc::clone(&audio_queue),
            Arc::clone(&total_chunks),
            Arc::clone(&generation_ended),
            min_ready,
        )
    } else {
        thread::spawn(move || {
            collect_all_audio(audio_queue, total_chunks, generation_ended)
        })
    };

    let all_samples = playback_handle.join().unwrap();

    let audio_duration = all_samples.len() as f32 / 24000.0;
    write_wav_file(&cli.output, &all_samples, 24000)?;

    let elapsed = start_time.elapsed();
    eprintln!("========================================");
    eprintln!("Output: {}", cli.output);
    eprintln!("Audio duration: {:.2} seconds", audio_duration);
    eprintln!("Processing time: {:.2} seconds", elapsed.as_secs_f32());
    if audio_duration > 0.0 {
        eprintln!("Real-time factor: {:.2}x", audio_duration / elapsed.as_secs_f32());
    }
    eprintln!("========================================");

    Ok(())
}
