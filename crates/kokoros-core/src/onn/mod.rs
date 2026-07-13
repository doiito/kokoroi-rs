#[cfg(feature = "ort")]
pub mod ort_base;
#[cfg(feature = "ort")]
pub mod ort_koko;
#[cfg(feature = "ort")]
pub use ort_koko::{ModelStrategy, OrtKoko};

#[cfg(all(feature = "onnx", not(feature = "ort"), not(feature = "oxionnx")))]
mod onnx_inference;

#[cfg(all(feature = "onnx", not(feature = "ort"), not(feature = "oxionnx")))]
pub use onnx_inference::OnnxInference;

#[cfg(feature = "oxionnx")]
mod oxionnx_inference;

#[cfg(feature = "oxionnx")]
pub use oxionnx_inference::OnnxInference;
