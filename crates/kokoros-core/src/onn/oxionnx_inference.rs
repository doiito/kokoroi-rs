use std::collections::HashMap;
use oxionnx_core::{TensorStorage, TypedTensor};

pub struct OnnxInference {
    session: oxionnx::Session,
}

impl OnnxInference {
    pub fn from_bytes(model_bytes: &[u8]) -> Result<Self, String> {
        let session = oxionnx::Session::from_bytes(model_bytes)
            .map_err(|e| format!("Failed to load ONNX model: {}", e))?;

        Ok(Self { session })
    }

    pub fn run(&self, input_ids: Vec<i64>, style: Vec<f32>, speed: f32) -> Result<Vec<f32>, String> {
        let seq_len = input_ids.len();
        let input_ids_storage = TensorStorage::I64(input_ids);
        let input_ids_tensor = TypedTensor::new(input_ids_storage, vec![1, seq_len]);

        let style_len = style.len();
        let style_tensor = TypedTensor::new(
            TensorStorage::F32(style),
            vec![1, style_len],
        );

        let speed_tensor = TypedTensor::new(
            TensorStorage::F32(vec![speed]),
            vec![1],
        );

        let mut inputs: HashMap<&str, TypedTensor> = HashMap::new();
        inputs.insert("input_ids", input_ids_tensor);
        inputs.insert("style", style_tensor);
        inputs.insert("speed", speed_tensor);

        let outputs = self.session.run_typed(&inputs)
            .map_err(|e| format!("Inference failed: {}", e))?;

        let output = outputs.values().next()
            .ok_or_else(|| "No output from model".to_string())?;

        let audio_data = match &output.storage {
            TensorStorage::F32(data) => data.clone(),
            _ => output.storage.to_f32_vec(),
        };

        Ok(audio_data)
    }
}
