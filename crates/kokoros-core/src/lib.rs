pub mod pipeline;
pub mod tts;
pub mod utils;

#[cfg(any(feature = "onnx", feature = "oxionnx", feature = "ort"))]
pub mod onn;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

#[cfg(target_arch = "wasm32")]
pub use wasm::KokoroWASM;

#[cfg(target_arch = "wasm32")]
pub use wasm::pcm_samples_to_wav_data;
